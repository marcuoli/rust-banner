//! Localized, caller-owned application banners. Rendering performs no Git, clock,
//! counter, command-line parsing, or application-exit discovery.
//!
//! ```
//! use rust_banner::{BuildDetails, Config, write};
//! let build = BuildDetails { version: "1.2.3+20261107.42", ..Default::default() };
//! let config = Config {
//!     name: "MyApp", author: "Application author", command: "myapp",
//!     build: Some(&build), no_color: true, ..Default::default()
//! };
//! let mut output = Vec::new();
//! write(&mut output, &config)?;
//! # Ok::<(), rust_banner::Error>(())
//! ```
mod catalog;
mod redact;

pub use catalog::{BuildTranslations, Translations, catalog, decode_translations};
pub use redact::{command_line, redact_args};
use std::{
    error, fmt,
    io::{self, Write},
};
pub use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

pub const ENGLISH: &str = "en";
pub const SPANISH: &str = "es";
pub const PORTUGUESE: &str = "pt-BR";
pub const COMPLETION: &str = "\nDone. Have a nice day !\n\n";
pub const WARRANTY: &str = concat!(
    "THE SOFTWARE IS PROVIDED \"AS IS\", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,\n",
    "FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,\n",
    "WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE."
);

/// Application metadata; this is independent of the dependency's Cargo version.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BuildDetails<'a> {
    pub version: &'a str,
    pub build_date: &'a str,
    pub overall_build_number: u64,
    pub commit: &'a str,
    pub rust_version: &'a str,
}

/// Presentation and metadata are borrowed immutably for each call.
/// `build` cannot be combined with legacy `version` or `build_time`.
#[derive(Clone, Debug, Default)]
pub struct Config<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub build: Option<&'a BuildDetails<'a>>,
    pub version: &'a str,
    pub build_time: &'a str,
    pub author: &'a str,
    pub command: &'a str,
    pub args: &'a [&'a str],
    pub started_at: Option<OffsetDateTime>,
    pub language: &'a str,
    pub translations: Option<&'a Translations>,
    pub sensitive_flags: &'a [&'a str],
    pub no_color: bool,
}

/// Configuration errors leave the writer untouched. I/O errors may follow partial
/// output; the original `io::Error` remains available through `source()`.
#[derive(Debug)]
pub enum Error {
    Configuration(String),
    Output {
        operation: &'static str,
        source: io::Error,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(message) => write!(f, "banner: {message}"),
            Self::Output { operation, source } => write!(f, "banner: {operation}: {source}"),
        }
    }
}
impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Self::Output { source, .. } => Some(source),
            _ => None,
        }
    }
}
fn invalid(message: impl Into<String>) -> Error {
    Error::Configuration(message.into())
}

/// Render and validate completely before the first write. Color is on even for
/// captured writers unless `no_color` is set or `NO_COLOR` is present (even empty).
pub fn write<W: Write + ?Sized>(writer: &mut W, config: &Config<'_>) -> Result<(), Error> {
    let output = render_with_color(config, std::env::var_os("NO_COLOR").is_none())?;
    writer
        .write_all(output.as_bytes())
        .map_err(|source| Error::Output {
            operation: "write output",
            source,
        })
}

/// Assemble a banner using the same environment color policy as [`write`].
pub fn render(config: &Config<'_>) -> Result<String, Error> {
    render_with_color(config, std::env::var_os("NO_COLOR").is_none())
}

fn render_with_color(config: &Config<'_>, color_available: bool) -> Result<String, Error> {
    if config.build.is_some() && (!config.version.is_empty() || !config.build_time.is_empty()) {
        return Err(invalid(
            "build cannot be combined with version or build_time",
        ));
    }
    let name = redact::sanitize(config.name);
    let author = redact::sanitize(config.author);
    let command = redact::sanitize(config.command);
    let version = redact::sanitize(config.build.map_or(config.version, |b| b.version));
    for (field, value) in [
        ("name", &name),
        ("version", &version),
        ("author", &author),
        ("command", &command),
    ] {
        if value.trim().is_empty() {
            return Err(invalid(format!("required metadata field {field} is empty")));
        }
    }
    let text = match config.translations {
        Some(custom) => {
            custom.validate(if config.language.is_empty() {
                ENGLISH
            } else {
                config.language
            })?;
            custom.clone()
        }
        None => catalog(config.language)?,
    };
    let mut description = redact::sanitize(config.description);
    let mut build_time = redact::sanitize(config.build_time);
    if let Some(build) = config.build {
        // Legacy custom catalogs retain the entire English group, never a mix of
        // individually missing labels in an otherwise localized build group.
        let fallback;
        let labels = if text.build == BuildTranslations::default() {
            fallback = catalog(ENGLISH)?;
            &fallback.build
        } else {
            &text.build
        };
        let overall = if build.overall_build_number == 0 {
            labels.unversioned.clone()
        } else {
            build.overall_build_number.to_string()
        };
        let details = format!(
            "{}: {} | {}: {} | {}: {}",
            labels.overall_build,
            overall,
            labels.commit,
            build_value(build.commit, &labels.unknown),
            labels.rust_version,
            build_value(build.rust_version, &labels.unknown)
        );
        description = if description.trim().is_empty() {
            details
        } else {
            format!("{} | {details}", description.trim())
        };
        build_time = build_value(build.build_date, &labels.unknown);
    }
    let mut header = name;
    if !description.is_empty() {
        header.push_str(&format!(" ({description})"));
    }
    header.push_str(&format!(" {version}"));
    if !build_time.is_empty() {
        header.push_str(&format!(" ({build_time})"));
    }
    let command = command_line(&command, config.args, config.sensitive_flags);
    let (author_separator, footer) = if let Some(started) = config.started_at {
        // Go's RFC3339 layout emits whole seconds while retaining the caller's
        // offset. Reject values RFC3339 cannot represent before writing anything.
        let started = started
            .replace_nanosecond(0)
            .map_err(|e| invalid(e.to_string()))?
            .format(&Rfc3339)
            .map_err(|e| invalid(format!("run-start timestamp: {e}")))?;
        let width = text
            .command_line
            .chars()
            .count()
            .max(text.run_started.chars().count());
        (
            " : ",
            format!(
                "{:<width$}: {command}\n{:<width$}: {started}",
                text.command_line, text.run_started
            ),
        )
    } else {
        (": ", format!("{}: {command}", text.command_line))
    };
    let warranty = if !config.no_color && color_available {
        format!("\x1b[31m{WARRANTY}\x1b[0m")
    } else {
        WARRANTY.into()
    };
    Ok(format!(
        "\n{header}\n\n{}{author_separator}{author}\n\n{}\n\n{warranty}\n\n{footer}\n\n",
        text.author, text.license_notice
    ))
}
fn build_value(value: &str, unknown: &str) -> String {
    let value = redact::sanitize(value);
    if value.trim().is_empty() {
        unknown.into()
    } else {
        value.trim().into()
    }
}

/// Append the invariant English completion line; never flush, close, or exit.
/// Rust references cannot be nil. Zero-progress writers return `WriteZero`;
/// legitimate short writes are retried by the standard `Write::write_all` API.
pub fn write_completion<W: Write + ?Sized>(writer: &mut W) -> Result<(), Error> {
    writer
        .write_all(COMPLETION.as_bytes())
        .map_err(|source| Error::Output {
            operation: "write completion",
            source,
        })
}

#[cfg(test)]
mod color_tests {
    use super::*;
    #[test]
    fn default_captured_and_explicit_color_policy() {
        let mut config = Config {
            name: "App",
            version: "1",
            author: "A",
            command: "app",
            ..Default::default()
        };
        assert!(
            render_with_color(&config, true)
                .unwrap()
                .contains(&format!("\x1b[31m{WARRANTY}\x1b[0m"))
        );
        assert!(!render_with_color(&config, false).unwrap().contains('\x1b'));
        config.no_color = true;
        assert!(!render_with_color(&config, true).unwrap().contains('\x1b'));
    }
}

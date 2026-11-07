use crate::{Error, invalid};
use serde::{Deserialize, Serialize};

/// Labels omit colons. A nonempty group must supply all five fields.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildTranslations {
    #[serde(default)]
    pub overall_build: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub rust_version: String,
    #[serde(default)]
    pub unknown: String,
    #[serde(default)]
    pub unversioned: String,
}

/// Complete per-call catalog. An empty build group is the explicit compatibility
/// path for catalogs predating structured build metadata.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Translations {
    pub language: String,
    pub author: String,
    pub command_line: String,
    pub run_started: String,
    pub license_notice: String,
    #[serde(default)]
    pub build: BuildTranslations,
}
impl Translations {
    pub fn validate(&self, expected: &str) -> Result<(), Error> {
        for (name, value, label) in [
            ("language", &self.language, false),
            ("author", &self.author, true),
            ("command_line", &self.command_line, true),
            ("run_started", &self.run_started, true),
            ("license_notice", &self.license_notice, false),
        ] {
            validate_field(name, value, label)?;
        }
        if self.language != expected {
            return Err(invalid(
                "translation language does not match selected language",
            ));
        }
        if self.build != BuildTranslations::default() {
            for (name, value, label) in [
                ("build.overall_build", &self.build.overall_build, true),
                ("build.commit", &self.build.commit, true),
                ("build.rust_version", &self.build.rust_version, true),
                ("build.unknown", &self.build.unknown, false),
                ("build.unversioned", &self.build.unversioned, false),
            ] {
                validate_field(name, value, label)?;
            }
        }
        Ok(())
    }
}
fn validate_field(name: &str, value: &str, label: bool) -> Result<(), Error> {
    if value.trim().is_empty()
        || value.chars().any(char::is_control)
        || (label && value.contains(':'))
    {
        return Err(invalid(format!("invalid translation field {name}")));
    }
    Ok(())
}

/// Decode strict UTF-8 JSON; reject unknown fields, incomplete catalogs, trailing
/// values, controls and language mismatches before rendering.
pub fn decode_translations(bytes: &[u8], expected: &str) -> Result<Translations, Error> {
    let data = std::str::from_utf8(bytes)
        .map_err(|_| invalid("translation catalog is not valid UTF-8"))?;
    let text: Translations = serde_json::from_str(data)
        .map_err(|e| invalid(format!("decode translation catalog: {e}")))?;
    text.validate(expected)?;
    Ok(text)
}

/// Select an exact built-in language; an empty identifier selects English.
/// Locale detection and application precedence belong to the caller.
pub fn catalog(language: &str) -> Result<Translations, Error> {
    let (language, author, command, run, notice, overall, unknown, unversioned) = match language {
        "" | "en" => (
            "en",
            "Author",
            "Command line",
            "Run started",
            "This software is distributed under the Oracle Universal Permissive License (UPL), Version 1.0.",
            "Overall build",
            "unknown",
            "unversioned",
        ),
        "es" => (
            "es",
            "Autor",
            "Línea de comandos",
            "Inicio",
            "Este software se distribuye bajo la Licencia Universal Permisiva de Oracle (UPL), versión 1.0.",
            "Compilación general",
            "desconocido",
            "sin versión",
        ),
        "pt-BR" => (
            "pt-BR",
            "Autor",
            "Linha de comando",
            "Início",
            "Este software é distribuído sob a Licença Universal Permissiva da Oracle (UPL), versão 1.0.",
            "Compilação geral",
            "desconhecido",
            "sem versão",
        ),
        _ => return Err(invalid("unsupported language")),
    };
    Ok(Translations {
        language: language.into(),
        author: author.into(),
        command_line: command.into(),
        run_started: run.into(),
        license_notice: notice.into(),
        build: BuildTranslations {
            overall_build: overall.into(),
            commit: "Commit".into(),
            rust_version: "Rust".into(),
            unknown: unknown.into(),
            unversioned: unversioned.into(),
        },
    })
}

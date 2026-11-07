# rust-banner

Native Rust application banners with English, Spanish and Brazilian Portuguese,
complete custom catalogs, safe command display, and the unchanged three-line
English UPL warranty. Licensed under [UPL 1.0](LICENSE).

The package coordinate is `rust-banner` 0.1.0. Application release identities,
including full date/PR versions and overall build numbers, are caller-owned and
independent of this coordinate. Source releases use immutable Git tags in the
[official public repository](https://github.com/marcuoli/rust-banner). This project
has no configured crates.io publication. Consumers select a real published tag:

```toml
[dependencies]
rust-banner = { git = "https://github.com/marcuoli/rust-banner", tag = "RELEASE_TAG" }
```

Replace `RELEASE_TAG` with an immutable tag listed by the repository. Cargo's
lockfile records its exact commit. Do not infer an application version from the
library package version.

```rust
use rust_banner::{BuildDetails, Config, write, write_completion};

fn main() -> Result<(), rust_banner::Error> {
let build = BuildDetails {
    version: "1.2.3+20261107.42", build_date: "2026-11-07T12:34:56-03:00",
    overall_build_number: 792, commit: "abcdef0", rust_version: "rustc 1.99.1",
};
let config = Config {
    name: "MyApp", description: "Database collector",
    author: "Application author", command: "myapp", build: Some(&build),
    args: &["--password", "hidden", "--file", "inventory.lst"],
    language: rust_banner::PORTUGUESE,
    no_color: true, // Map the application's parsed --no-color flag here.
    ..Default::default()
};
let mut output = std::io::stdout().lock();
write(&mut output, &config)?;
// Application output goes here, then append once before flushing/closing.
write_completion(&mut output)?;
Ok(())
}
```

The structured header is
`MyApp (Database collector | Overall build: N | Commit: SHA | Rust: rustc X.Y.Z) VERSION (BUILD_DATE)`.
No version truncation or Git, clock, counter or runtime compiler discovery occurs.
An absent date, commit or compiler uses the catalog's unknown marker. A zero
overall number uses its unversioned marker. The application must supply a version.
Borrowed configuration and build details are never mutated by rendering.

Legacy callers can supply `Config.version` and optional `Config.build_time`
instead of `Config.build`. Combining these sources is an error, including values
that only contain controls. Name, version, author and command must be nonempty
after removal of Unicode controls. Description and legacy build time are optional.
`started_at: None` omits the run-start line. A caller-supplied `OffsetDateTime`
is formatted as RFC3339 whole seconds with its UTC offset preserved; timestamp
values outside RFC3339's range return a configuration error. No clock is read.
The reexported `OffsetDateTime` type comes from the official `time` crate.

`write` assembles and validates the entire banner before touching the writer;
`render` returns that same text. Output always uses LF, including Windows.
`write_completion` appends exactly `"\nDone. Have a nice day !\n\n"`, in plain
English, and never flushes, closes, or exits. Use the application's human output
stream; omit human messages when its machine-readable mode requires that.

The complete legal warranty is red by default, even in captured output. Set
`no_color`, or set `NO_COLOR` to any value including empty, to disable ANSI styling.
The library does not parse flags or decide whether the app should show a banner.

Language identifiers are exact: `en`, `es`, `pt-BR`; empty means English. Other
identifiers, including `pt` and `en-US`, require a complete matching custom
`Translations` value in `Config.translations`. `catalog` returns an owned built-in
catalog for application customization. `decode_translations` accepts strict UTF-8
JSON and rejects unknown fields, trailing data, incomplete base fields, controls,
and language mismatches. Rust strings cannot contain invalid UTF-8.

Every base field is required, even a run-start label not currently displayed.
Labels omit colons. A nonempty `build` group must contain all five fields:
`overall_build`, `commit`, `rust_version`, `unknown`, `unversioned`. A completely
empty group preserves the old whole English group as a compatibility path;
strict localized applications supply a complete group. Partial groups never fall
back field by field. The legal warranty and completion line remain English.
Unicode label alignment counts code points, not terminal display cells.

`redact_args` returns a sanitized copy. `command_line` quotes it for display,
never execution by a shell. Quoted nonprintable characters use official Unicode 16
data; see the precise Unicode-version compatibility boundary in [qualification](docs/native-port.md). Matching uses Unicode simple lowercase on both names and custom flags, preserving
Go credential matching for dotted I and Greek sigma. Defaults match long flag substrings
`password`, `passphrase`, `passwd`, `secret`, `token`, `private-key`, and `wallet`.
Additional names such as `-p` and `--inventory-file` extend the defaults. Inline
and separate values are redacted; a separate value is consumed even if it looks
like another flag. Short clusters are not parsed. URL user information and
sensitive query values are redacted, query keys are sorted, and repeated safe
values retain order. Malformed recognized URLs or queries become
`[REDACTED-URL]`; ordinary protected values become `[REDACTED]`.

URL validation uses the official `url` crate, with raw presentation retained to
avoid WHATWG changes to paths, host case or explicit ports. Authorities that the
maintained parser rejects are hidden conservatively, even where Go's parser
accepts an unusual authority. Query byte escapes, including non-UTF-8 values,
remain byte preserving. Callers must mark app-specific secret flags. Unmarked
positional secrets, arbitrary prose, URL paths and fragments are not searched for
credentials. Terminal controls are removed before matching and display.

I/O errors retain the original `std::io::Error` through `Error::source`. Rust
writer references cannot be null. Idiomatic `Write::write_all` retries legitimate
short writes and interrupted writes; zero progress returns `WriteZero`. This is
the explicit Rust adaptation of Go's nil/short-write handling. Partial text may
remain after an output failure.

Development uses pinned Rust/Clippy 1.99.1 on Windows x86_64 MSVC. Edition 2024;
no nondefault features or alternate targets are claimed. Run
`./scripts/validate.ps1`; it scopes outputs, caches, temporary files, fixture roots
and full logs to the external project namespace and retains the managed Rustup
home. It runs formatting, locked all-target Clippy with warnings denied, locked
tests and package verification. `cargo package` is a validation step, not a
registry or release publication. See [qualification and design](docs/native-port.md).

use rust_banner::{
    BuildDetails, BuildTranslations, Config, Error, OffsetDateTime, Translations, catalog,
    command_line, decode_translations, redact_args, render, write, write_completion,
};
use serde::Deserialize;
use std::{
    error::Error as _,
    io::{self, Write},
};
use time::format_description::well_known::Rfc3339;

fn config() -> Config<'static> {
    Config {
        name: "MyApp",
        description: "Database collector",
        version: "1.2.3+20261107.42",
        build_time: "2026-11-07T12:34:56-03:00",
        author: "Application author",
        command: "myapp",
        args: &[
            "--password",
            "hidden",
            "two words",
            "--file",
            "inventory.lst",
        ],
        no_color: true,
        ..Default::default()
    }
}
#[derive(Deserialize)]
struct Vectors {
    render: Vec<RenderVector>,
    redact: Vec<RedactVector>,
    quote: Vec<QuoteVector>,
    completion: String,
}
#[derive(Deserialize)]
struct RenderVector {
    language: String,
    structured: bool,
    started: bool,
    want: String,
}
#[derive(Deserialize)]
struct RedactVector {
    args: Vec<String>,
    extra: Option<Vec<String>>,
    want: Vec<String>,
}
#[derive(Deserialize)]
struct QuoteVector {
    command: String,
    args: Vec<String>,
    want: String,
}
#[test]
fn go_produced_golden_vectors() {
    let vectors: Vectors = serde_json::from_str(include_str!("fixtures/go-parity.json")).unwrap();
    for vector in vectors.render {
        let mut c = config();
        c.language = &vector.language;
        if vector.started {
            c.started_at = Some(
                OffsetDateTime::parse("2026-11-07T13:14:15.123456789-03:00", &Rfc3339).unwrap(),
            );
        }
        let build = BuildDetails {
            version: c.version,
            build_date: c.build_time,
            overall_build_number: 792,
            commit: "abcdef0123456789abcdef0123456789abcdef01",
            rust_version: "rustc 1.99.1",
        };
        if vector.structured {
            c.build = Some(&build);
            c.version = "";
            c.build_time = "";
        }
        assert_eq!(
            render(&c).unwrap(),
            vector.want,
            "language={} structured={} started={}",
            vector.language,
            vector.structured,
            vector.started
        );
    }
    for vector in vectors.redact {
        let args: Vec<_> = vector.args.iter().map(String::as_str).collect();
        let extra: Vec<_> = vector
            .extra
            .as_deref()
            .unwrap_or(&[])
            .iter()
            .map(String::as_str)
            .collect();
        assert_eq!(redact_args(&args, &extra), vector.want, "args={args:?}");
    }
    for vector in vectors.quote {
        let args: Vec<_> = vector.args.iter().map(String::as_str).collect();
        assert_eq!(
            command_line(&vector.command, &args, &[]),
            vector.want,
            "args={args:?}"
        );
    }
    let mut output = Vec::new();
    write_completion(&mut output).unwrap();
    assert_eq!(output, vectors.completion.as_bytes());
}
#[test]
fn invalid_metadata_writes_nothing() {
    for field in 0..6 {
        let mut c = config();
        match field {
            0 => c.name = "\0\x1b",
            1 => c.version = "\n\t ",
            2 => c.author = "",
            3 => c.command = " \n",
            4 => c.language = "pt",
            _ => c.language = "en-US",
        }
        let mut output = Vec::new();
        assert!(write(&mut output, &c).is_err());
        assert!(output.is_empty());
    }
    let build = BuildDetails {
        version: "1",
        ..Default::default()
    };
    for (version, build_time) in [("1", ""), ("", "date"), ("\0", "")] {
        let c = Config {
            build: Some(&build),
            version,
            build_time,
            ..config()
        };
        let mut out = Vec::new();
        assert!(write(&mut out, &c).is_err());
        assert!(out.is_empty());
    }
    let empty = BuildDetails::default();
    let c = Config {
        build: Some(&empty),
        version: "",
        build_time: "",
        ..config()
    };
    assert!(render(&c).is_err());
}
#[test]
fn unknown_build_values_and_control_sanitizing() {
    for (language, overall, unknown) in [
        ("en", "unversioned", "unknown"),
        ("es", "sin versión", "desconocido"),
        ("pt-BR", "sem versão", "desconhecido"),
    ] {
        let build = BuildDetails {
            version: "0.0.0+unknown\0",
            commit: "\x1b\n",
            rust_version: "\t\n",
            ..Default::default()
        };
        let c = Config {
            build: Some(&build),
            version: "",
            build_time: "",
            description: " \n\t",
            language,
            ..config()
        };
        let got = render(&c).unwrap();
        assert!(got.contains(&format!(
            ": {overall} | Commit: {unknown} | Rust: {unknown}) 0.0.0+unknown ({unknown})"
        )));
        assert_eq!(build.version, "0.0.0+unknown\0");
        assert_eq!(c.description, " \n\t");
    }
    let build = BuildDetails {
        version: "1\n",
        build_date: " date\n ",
        overall_build_number: u64::MAX,
        commit: " x\0 ",
        rust_version: " rustc\t ",
    };
    let c = Config {
        build: Some(&build),
        version: "",
        build_time: "",
        description: "Collect\nor",
        ..config()
    };
    assert!(render(&c).unwrap().starts_with("\nMyApp (Collector | Overall build: 18446744073709551615 | Commit: x | Rust: rustc) 1 (date)\n"));
}
fn french() -> Translations {
    Translations {
        language: "fr".into(),
        author: "Auteur".into(),
        command_line: "É".into(),
        run_started: "Début".into(),
        license_notice: "Avis privé.".into(),
        build: BuildTranslations::default(),
    }
}
#[test]
fn custom_catalog_legacy_complete_and_unicode_alignment() {
    let mut text = french();
    let build = BuildDetails {
        version: "1",
        ..Default::default()
    };
    let c = Config {
        language: "fr",
        translations: Some(&text),
        version: "",
        build_time: "",
        build: Some(&build),
        description: "",
        started_at: Some(OffsetDateTime::parse("2026-11-07T00:00:00+05:45", &Rfc3339).unwrap()),
        ..config()
    };
    let rendered = render(&c).unwrap();
    assert!(rendered.starts_with("\nMyApp (Overall build: unversioned | Commit: unknown | Rust: unknown) 1 (unknown)\n\nAuteur : "));
    assert!(rendered.contains("\n\nÉ    : myapp"));
    assert!(rendered.contains("\nDébut: 2026-11-07T00:00:00+05:45\n\n"));
    assert!(rendered.contains("Avis privé."));
    text.build = BuildTranslations {
        overall_build: "Construction".into(),
        commit: "Révision".into(),
        rust_version: "Compilateur".into(),
        unknown: "inconnu".into(),
        unversioned: "sans version".into(),
    };
    let snapshot = text.clone();
    let c = Config {
        language: "fr",
        translations: Some(&text),
        version: "",
        build_time: "",
        build: Some(&build),
        description: "Invariant",
        ..config()
    };
    assert!(render(&c).unwrap().contains(
        "Invariant | Construction: sans version | Révision: inconnu | Compilateur: inconnu"
    ));
    assert_eq!(text, snapshot);
}
#[test]
fn invalid_catalog_fields_and_groups_are_atomic() {
    for group in [false, true] {
        for field in 0..5 {
            for invalid in ["", "  ", "\n", "\x1b[31m", "\u{0085}"] {
                let mut text = catalog("en").unwrap();
                let selected = if group {
                    match field {
                        0 => &mut text.build.overall_build,
                        1 => &mut text.build.commit,
                        2 => &mut text.build.rust_version,
                        3 => &mut text.build.unknown,
                        _ => &mut text.build.unversioned,
                    }
                } else {
                    match field {
                        0 => &mut text.language,
                        1 => &mut text.author,
                        2 => &mut text.command_line,
                        3 => &mut text.run_started,
                        _ => &mut text.license_notice,
                    }
                };
                *selected = invalid.into();
                let c = Config {
                    translations: Some(&text),
                    ..config()
                };
                let mut out = Vec::new();
                assert!(
                    write(&mut out, &c).is_err(),
                    "group={group},field={field},invalid={invalid:?}"
                );
                assert!(out.is_empty());
            }
        }
    }
    for field in 0..6 {
        let mut text = catalog("en").unwrap();
        let label = match field {
            0 => &mut text.author,
            1 => &mut text.command_line,
            2 => &mut text.run_started,
            3 => &mut text.build.overall_build,
            4 => &mut text.build.commit,
            _ => &mut text.build.rust_version,
        };
        label.push(':');
        assert!(text.validate("en").is_err());
    }
    assert!(french().validate("en").is_err());
    for alias in ["pt", "en-US", "de", "../en", "EN", " en"] {
        assert!(catalog(alias).is_err());
    }
    assert_eq!(catalog("").unwrap(), catalog("en").unwrap());
}
#[test]
fn strict_json_decoding() {
    let valid = serde_json::to_vec(&french()).unwrap();
    assert_eq!(decode_translations(&valid, "fr").unwrap(), french());
    for suffix in [" {}", " junk"] {
        let mut broken = valid.clone();
        broken.extend_from_slice(suffix.as_bytes());
        assert!(decode_translations(&broken, "fr").is_err());
    }
    let mut object = serde_json::to_value(french()).unwrap();
    object["unexpected"] = true.into();
    assert!(decode_translations(&serde_json::to_vec(&object).unwrap(), "fr").is_err());
    for field in [
        "language",
        "author",
        "command_line",
        "run_started",
        "license_notice",
    ] {
        let mut object = serde_json::to_value(french()).unwrap();
        object.as_object_mut().unwrap().remove(field);
        assert!(decode_translations(&serde_json::to_vec(&object).unwrap(), "fr").is_err());
    }
    for broken in [
        b"null".as_slice(),
        b"\xff",
        br#"{"language":"fr","author":"\ud800"}"#,
    ] {
        assert!(decode_translations(broken, "fr").is_err());
    }
    assert!(decode_translations(&valid, "en").is_err());
}
struct Failure;
impl Write for Failure {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::PermissionDenied, "sentinel"))
    }
    fn flush(&mut self) -> io::Result<()> {
        panic!("library must not flush")
    }
}
struct Zero;
impl Write for Zero {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Ok(0)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
struct Short(Vec<u8>);
impl Write for Short {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        let len = input.len().min(3);
        self.0.extend_from_slice(&input[..len]);
        Ok(len)
    }
    fn flush(&mut self) -> io::Result<()> {
        panic!("library must not flush")
    }
}
#[test]
fn writer_failures_short_writes_and_completion() {
    for completion in [false, true] {
        let mut failed = Failure;
        let error = if completion {
            write_completion(&mut failed)
        } else {
            write(&mut failed, &config())
        }
        .unwrap_err();
        assert_eq!(
            error
                .source()
                .unwrap()
                .downcast_ref::<io::Error>()
                .unwrap()
                .kind(),
            io::ErrorKind::PermissionDenied
        );
        assert!(error.to_string().contains("sentinel"));
        let mut zero = Zero;
        let error = if completion {
            write_completion(&mut zero)
        } else {
            write(&mut zero, &config())
        }
        .unwrap_err();
        assert!(
            matches!(error,Error::Output{source,..} if source.kind()==io::ErrorKind::WriteZero)
        );
        let mut short = Short(Vec::new());
        if completion {
            write_completion(&mut short).unwrap();
            assert_eq!(short.0, rust_banner::COMPLETION.as_bytes());
        } else {
            write(&mut short, &config()).unwrap();
            assert_eq!(short.0, render(&config()).unwrap().as_bytes());
        }
    }
}
#[test]
fn independent_concurrent_locales() {
    std::thread::scope(|scope| {
        for i in 0..24 {
            scope.spawn(move || {
                let language = ["en", "es", "pt-BR"][i % 3];
                let c = Config {
                    language,
                    ..config()
                };
                assert!(
                    render(&c)
                        .unwrap()
                        .contains(&catalog(language).unwrap().command_line)
                );
            });
        }
    });
}
#[test]
fn hostile_secrets_are_hidden_without_mutating_inputs() {
    let args = [
        "--pass\0word=secret-one",
        "https://u:p@host/?ToKeN=secret-two",
        "--endpoint=https://u:p@[bad",
        "https://u:p@host/?token=secret-three;foo=bar",
        "--api-key",
        "secret-four",
    ];
    let before = args;
    let output = redact_args(&args, &["--api-key"]);
    assert_eq!(args, before);
    let output = output.join(" ");
    for secret in [
        "secret-one",
        "secret-two",
        "secret-three",
        "secret-four",
        "u:p",
    ] {
        assert!(!output.contains(secret));
    }
    assert!(!output.chars().any(char::is_control));
}

// Environment is changed only on a child Command, avoiding process-global mutation
// and preserving independent concurrent locale/color tests.
#[test]
fn no_color_environment_presence_even_when_empty() {
    for value in [None, Some(""), Some("0")] {
        let mut command = std::process::Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", "environment_color_child", "--ignored"])
            .env_remove("NO_COLOR")
            .env(
                "RUST_BANNER_EXPECT_COLOR",
                if value.is_none() { "yes" } else { "no" },
            );
        if let Some(value) = value {
            command.env("NO_COLOR", value);
        }
        let output = command.output().unwrap();
        assert!(
            output.status.success(),
            "child failed: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
#[test]
#[ignore = "subprocess-only environment isolation"]
fn environment_color_child() {
    let c = Config {
        no_color: false,
        ..config()
    };
    let got = render(&c).unwrap();
    assert_eq!(
        got.contains("\x1b[31m"),
        std::env::var("RUST_BANNER_EXPECT_COLOR").unwrap() == "yes"
    );
}

#[test]
fn unicode_simple_case_redacts_default_and_custom_credentials() {
    let args = [
        "--PRİVATE-KEY=hidden-inline",
        "--PRİVATE-KEY",
        "hidden-separate",
        "https://host/?PR%C4%B0VATE-KEY=hidden-query",
    ];
    let expected = [
        "--PRİVATE-KEY=[REDACTED]",
        "--PRİVATE-KEY",
        "[REDACTED]",
        "https://host/?PR%C4%B0VATE-KEY=%5BREDACTED%5D",
    ];
    assert_eq!(redact_args(&args, &[]), expected);
    assert_eq!(
        redact_args(
            &["--İD=x", "--İD", "y", "https://host/?%C4%B0D=z"],
            &["--id"]
        ),
        [
            "--İD=[REDACTED]",
            "--İD",
            "[REDACTED]",
            "https://host/?%C4%B0D=%5BREDACTED%5D"
        ]
    );
    assert_eq!(redact_args(&["--id=x"], &["--İD"]), ["--id=[REDACTED]"]);
    assert_eq!(
        redact_args(
            &[
                "--ΟΣ=x",
                "--ΟΣ",
                "y",
                "https://host/?%CE%9F%CE%A3=z",
                "--ος=safe-final-sigma"
            ],
            &["--οσ"]
        ),
        [
            "--ΟΣ=[REDACTED]",
            "--ΟΣ",
            "[REDACTED]",
            "https://host/?%CE%9F%CE%A3=%5BREDACTED%5D",
            "--ος=safe-final-sigma"
        ]
    );
}

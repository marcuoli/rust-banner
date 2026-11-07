# Native port and qualification

Issue: #1. Scope: public reusable library only; no private application source,
inputs, configuration or artifacts. Parent delivery owns review, integration,
immutable source tags, mirror verification and removal of the issue worktree.

The reference is the owned Go banner's renderer, build details, catalogs,
completion and redaction sources, tests and README. The Rust modules follow the
same contracts with focused native APIs: borrowed Config/BuildDetails, owned
per-call translations, strict serde JSON, time::OffsetDateTime and std::io::Write.
The native build compiler label is Rust. Missing fields never trigger clock,
Git, compiler, counter or environment metadata discovery. Only NO_COLOR presence
is read for presentation policy. The dependencies are official upstream crates;
no copied third-party code, local patches or cache rewriting are used.

Implementation plan and checks:

1. Read the complete owned Go contracts and tests; identify exact layout,
   localization, validation-before-write, unknown markers and display behavior.
2. Implement native focused renderer, catalog and redaction modules, preserving
   compatibility while expressing ownership and writer errors in Rust.
3. Generate public-safe fixed goldens with the installed Go reference compiler.
   Retain 12 language/build/timestamp renderer vectors, 20 credential/URL vectors,
   4 quoting vectors and the exact completion bytes. Only Rust label/compiler
   substitution is deliberate in the render fixtures.
4. Test invalid metadata and complete/partial/legacy catalogs, UTF-8/strict JSON,
   Unicode code-point alignment, max counters, controls, immutable inputs,
   concurrent locales, writer error identity/short writes and actual NO_COLOR
   presence in isolated child processes. Test documentation examples.
5. Run pinned formatting, locked all-target Clippy, locked tests and package
   verification; retain compiler, graph and complete logs externally.
6. Review public content and dependency licenses/maintenance, then hand the exact
   result to the parent delivery owner. Never represent a passing local port as
   already integrated, mirrored or released.

Reproduce goldens with `scripts/generate-golden.ps1 -ReferenceRoot <owned-go-banner>`.
The exporter stages only owned Go library sources in external issue scratch,
uses external Go caches, generates external evidence first and deliberately
promotes the immutable JSON snapshot into tests. It neither modifies the Go
reference nor uses a local replacement of third-party dependencies.

Compatibility boundaries are explicit: Rust references are non-null and
write_all retries short/interrupted writes, returning WriteZero on no progress;
RFC3339 timestamps are caller-provided typed values; URL authorities rejected by
the official WHATWG parser are hidden conservatively. Golden vectors verify the
common Go presentation rather than claiming both parsers accept identical URI
languages. Alignment is code points rather than terminal cells. No Linux, macOS,
alternate features, registry publication, database or application qualification
is claimed by this library's Windows checks.

Reference qualification: owned Go source commit
`d5d39178069584bc107243d820b2d3e67da08d05`, generated with Go 1.27.3
windows/amd64. Display quoting uses the official unicode-general-category 1.1.0
Unicode 16 data. The Go reference uses Unicode 17; newly assigned characters may
be conservatively escaped inside quoted values until an official compatible
category-data release is available. Do not copy or patch Unicode dependency
source/tables to bypass that version boundary. Existing shared Unicode characters,
including format/private-use/unassigned characters, are verified by Go goldens.
Credential-name matching uses official ICU simple scalar lowercase mappings,
matching Go's policy rather than full-string/contextual lowercase. Regression
vectors cover dotted capital I in default inline/separate flags and percent-
decoded URL keys, plus dotted-I and Greek sigma custom flags in both directions.
Full lowercase would expand İ or select a final sigma and fail to hide credentials.
# Rust library development

Use idiomatic Rust and the simplest clear implementation. Preserve the documented
Go behavior contracts and use compatible official dependencies. Do not patch,
fork or rewrite third-party source or caches. Keep public content free of private
application source, inputs and credentials.

Run Rustfmt, locked Clippy with warnings denied, locked behavior/compatibility
tests and package verification. Keep generated artifacts, caches, fixtures and
logs in the owning external project namespace. Keep permanent main source in
its canonical backed-up location. Runtime metadata/rendering performs no Git,
clock or counter discovery.

Track implementation in the owning issue. Use a feature worktree, reviewed PR,
verified mirror publication and canonical-main synchronization. Before closing,
preserve unique evidence and remove issue worktrees through Git, verifying their
actual administrative directory and registration are gone. Published tags and
assets are immutable. Do not claim untested platform or Oracle compatibility.
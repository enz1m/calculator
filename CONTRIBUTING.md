# Contributing

Use the Rust toolchain selected by rust-toolchain.toml and commit the app's
lockfile. The framework is a pinned public Git dependency; there is no vendored
SDK to edit here. Fix shared control defects in enzUI and update its dependency
only after a compatible release exists.

```sh
cargo fmt --all -- --check
cargo test --locked
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo build --locked
```

`tools/build.sh` forwards Cargo subcommands unchanged. Personal compiler job,
resource and cache settings belong in your environment or user configuration.
Keep temporary QA fixtures and captures under ignored AGENTS/.

Tests should protect expression semantics, conversion accuracy, user workflows
and history safety. Prefer table-driven cases over separate tests for each
operator or setter. For UI changes, exercise Basic, Scientific and Convert in
both themes, at the minimum size and with keyboard input. Use disposable XDG
config/data directories for review, and capture the actual app window.

Contributions use the MIT license. Preserve third-party notices.

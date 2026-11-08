# Calculator contributor notes

Read README.md for setup and storage. Keep app
logic in Rust and reuse enzUI controls, theme tokens and native text transfers.

Use the pinned public SDK tag and foundation revisions in Cargo.toml. Update
Cargo.lock deliberately; never edit Cargo's cached dependency sources.

Use ordinary Cargo commands or tools/build.sh, which forwards to Cargo without
resource limits. Keep personal build policies outside the tracked repository.

Retain tests for observable expression semantics, conversion, history safety and
keyboard routing. Avoid tests that just assert setter values or source text.
Comments explain constraints; prefer descriptive names over narration.

Use an isolated Wayland session and disposable XDG data/config paths for visual
checks. Capture the exact app window in both themes, all modes and minimum size.
Keep local QA scripts, captures and notes under ignored AGENTS/. Only curated
README screenshots belong in docs/screenshots/.

Before finishing code changes, run formatting, app tests and strict Clippy.
Stage explicit paths and keep build products, credentials and QA state out of
commits.

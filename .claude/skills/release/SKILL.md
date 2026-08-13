---
name: release
description: Cut a new release of the tinyset crate. Use when the user asks to "create a release", "issue a new release", "cut a release" or "publish a release".
---

# Issuing a new release

Only do this when the user asks for a release, and at each step below show what you will do and ask for the user's confirmation.

1. **Decide the bump.** Look at what landed since the commit that set the current
   version, ignoring CI/tooling/docs-only commits. Patch for bug fixes, minor for
   new public features. An MSRV bump is *not* breaking here — the crate uses
   `rust-version` to signal it (see the 0.5.2 changelog entry) — but do mention it
   in the changelog. Ask if a change might be breaking.

2. **Add a `CHANGELOG.md` entry at the top**, matching the surrounding style:
   `* X.Y.Z - Mon. DD, YYYY`, then one `-` bullet per user-facing change.

3. **Bump `version` in `Cargo.toml`** — the `tinyset` package only, not `bench`.

4. **`cargo build && cargo test`** (the build refreshes `Cargo.lock`). Both must pass.

5. **Commit and tag.** Tag is the bare version, no `v` prefix: `git tag X.Y.Z`.

6. **Run `cargo publish`**

7. **After publish succeeds**, `git push && git push --tags`.

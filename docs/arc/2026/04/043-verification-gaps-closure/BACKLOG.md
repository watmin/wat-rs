# Arc 043 — BACKLOG

Four slices.

---

## Slice 1 — USER-GUIDE.md §12 + sigma formula

**Status: ready.**

§12 Error handling drift (missed in arc 038):
- Line 1215: Bundle return type → `:wat::holon::BundleResult`
  typealias (arc 032).
- Line 1217: capacity-mode list `:silent / :warn / :error / :abort`
  → `:error / :abort` (arc 037).
- Line 1219: `floor(sqrt(dims))` framing → reference `d` per-AST
  per arc 037.
- Lines 1224: explicit Result type → BundleResult typealias.
- Line 1521: `floor(sqrt(dims))` capacity-overflow gotcha — same
  framing update.

§Config setters (optional) — line 137-138:
- `presence_sigma(d) = floor(√d/2) - 1` → add `max(1)` clamp.

## Slice 2 — README test surface + wat_cache reference

**Status: ready.**

§Status block:
- Replace "zero regressions across every shipped arc" prose with
  concrete numbers: 725 Rust tests across 25 binaries; ~58 wat
  tests across 11 wat-tests files; zero clippy.
- Drop `wat_cache` from the integration-suite enumeration (file
  doesn't exist in wat-rs/tests/; cache tests live in
  `crates/wat-lru/tests/test.rs`).

## Slice 3 — `src/*.rs` doc-comment sweep

**Status: ready.**

Seven `wat::test_suite!` → `wat::test!` updates across three
files:
- `src/test_runner.rs:20, 41, 145, 361` (4 sites)
- `src/bin/wat.rs:254, 277` (2 sites)
- `src/panic_hook.rs:47` (1 site)

Plus one set-dims! example in a runnable doc comment:
- `src/harness.rs:26` — drop or replace with current minimal form.

Plus context check on:
- `src/test_runner.rs:421` — "arc 037 contract, set-dims! is a
  no-op" comment. Read context first; if it's documenting parser
  rejection it's accurate; if it's runtime-behavior prose it's
  stale.

## Slice 4 — INSCRIPTION + cross-references

**Status: obvious in shape.**

- `INSCRIPTION.md` summarizing all four verification findings,
  the verification methodology (`cargo test --release`,
  `src/` grep, cross-doc spot-check), and the honest disclosure
  that arc 042's "current through arc 037" claim was too
  confident — §12 of USER-GUIDE had three stale spots.
- `docs/README.md` arc index extended.
- 058 FOUNDATION-CHANGELOG row in lab repo.

---

## Cross-cutting

- Verify after each slice: grep + spot-read.
- Commit per slice. Push per commit.
- This arc's INSCRIPTION explicitly names what arc 042 missed.
  Honest record beats the appearance of completeness.

# Arc 043 — Verification gaps closure

**Opened:** 2026-04-24.
**Scope:** the four "Not verified" items I named in arc 042's
post-review honest assessment, plus the additional drift the
verification surfaced.

## Why this arc exists

After arcs 038-042 closed the wat-rs user-facing doc audit, the
builder asked: "your assessment is that wat-rs is honest again?"

My honest answer was *substantially more honest, not provably
honest*. Four follow-on checks would close the gaps:

1. Run `cargo test --workspace` and update README's test surface
   line.
2. Verify sigma formulas in USER-GUIDE / CONVENTIONS against
   `src/dim_router.rs`.
3. Sweep `src/*.rs` module-level doc comments for retired forms
   (commit `5b5fad8` touched 7 source files; subsequent arcs
   touched some but not all).
4. Cross-doc consistency spot-check.

Builder confirmed: do all four. Historical records (arc
INSCRIPTIONs, arc 005 INVENTORY) stay frozen.

## What the verifications surfaced

### 1. Test counts

`cargo test --release` against `wat-rs/Cargo.toml`:
- **725 Rust tests** across 25 binaries (lib unit tests + integration suites).
- **~58 wat tests** across 11 wat-tests files (`wat-tests/holon/*`,
  `wat-tests/std/*`, `wat-tests/std/service/*`).
- Zero failures, zero warnings.

README currently says "zero regressions across every shipped arc"
without numbers (deliberately de-specified in arc 039). Concrete
numbers are now verifiable; including them is more honest than
omitting them.

Plus: README enumerates `wat_cache` as one of the integration
suites — that file does **not exist** in `wat-rs/tests/`. Per
arc 013, the cache surface externalized to `wat-lru` whose tests
live at `crates/wat-lru/tests/test.rs`. Reference is stale.

### 2. Sigma formulas

`src/dim_router.rs:185-205`:

```rust
impl SigmaFn for DefaultPresenceSigma {
    fn sigma_at(&self, d: usize, _sym: &SymbolTable) -> i64 {
        let sqrt_d = (d as f64).sqrt();
        let s = (sqrt_d.floor() / 2.0).floor() as i64 - 1;
        s.max(1)                                              // ← clamp
    }
}

impl SigmaFn for DefaultCoincidentSigma {
    fn sigma_at(&self, _d: usize, _sym: &SymbolTable) -> i64 {
        1
    }
}
```

USER-GUIDE.md cites `presence_sigma(d) = floor(√d/2) - 1` —
correct on the formula but **missing the `max(1)` clamp**. At low
`d` (e.g. `d=4`, `floor(√4)/2 - 1 = 0`) the bare formula yields
0 or negative; the clamp keeps presence-floor meaningful at small
dims. The clamp is real substrate behavior; doc should reflect it.

CONVENTIONS.md mentions sigma functions abstractly ("functions of
`d`") — no specific formula, so nothing to fix there.

### 3. `src/*.rs` doc-comment audit

Grep for retired forms in `src/`:

**Drift found:**
- `src/test_runner.rs:20, 41, 145, 361` — four `wat::test_suite!`
  references in module-level doc comments and example blocks
  (arc 018 renamed to `wat::test!`).
- `src/bin/wat.rs:254, 277` — two more `wat::test_suite!` references
  in CLI entry-point doc comments.
- `src/panic_hook.rs:47` — one `wat::test_suite!` reference in
  module-level "// !" comment.
- `src/harness.rs:26` — example showing `(:wat::config::set-dims!
  1024)` in a doc-comment runnable example (arc 037 retired the
  setter).
- `src/test_runner.rs:421` — comment says "arc 037 contract,
  set-dims! is a no-op" — needs context check (might be accurate
  if it's documenting parser rejection).

**Confirmed correct (NOT drift):**
- Test fixtures in `src/load.rs:1455`, `src/resolve.rs:321`,
  `src/config.rs:619-622`, `src/freeze.rs:929,1268` — string
  literals testing that retired forms are *rejected*. These are
  tests; they need the retired strings.

### 4. Cross-doc spot-check — surfaced more drift

While verifying sigma formulas in USER-GUIDE, I grepped for
remaining capacity-mode references and **found three drift spots
in §12 Error Handling that arc 038's slice 5 missed**:

- **Line 1217**: "The four `capacity-mode` values (`:silent` /
  `:warn` / `:error` / `:abort`)" — arc 037 retired silent + warn;
  only two remain. Arc 038's slice 5 updated §6 Capacity guard but
  didn't reach §12.
- **Lines 1215, 1224**: explicit
  `:Result<:wat::holon::HolonAST, :wat::holon::CapacityExceeded>`
  return type — should use `:wat::holon::BundleResult` typealias
  (arc 032). My slice 5 updated §6 but missed §12.
- **Lines 1219, 1521**: `floor(sqrt(dims))` references — under arc
  037 the formula uses `d` (per-AST), not a single `dims`. The
  prose can stand ("dims" still conceptually refers to the encoded
  vector dim) but should at least drop the `()` accessor framing.

This is honest drift I introduced by missing §12 in arc 038.
Catching it now is the verification pass paying out.

## The discipline

Same as arcs 038-042. Targeted edits per topic; commit per slice;
push per commit; verify after each slice.

## Out of scope

- **Historical records.** Arc INSCRIPTIONs (DESIGN, BACKLOG,
  INSCRIPTION) stay frozen — including arc 042's INSCRIPTION
  whose claim that wat-rs was "current through arc 037" turned
  out to overstate the §12 state. The honest correction lives
  in arc 043's INSCRIPTION; the prior arc's record stays as-is.
- **Lab `CLAUDE.md`** — different repo; ships as a lab arc.
- **arc 005 INVENTORY.md** — preserved per builder's earlier
  call.

## What this arc proves

**Verification finds drift assertion can't.** Arc 042's confidence
("wat-rs honest through arc 037") was based on grep + per-arc
sweep without cross-doc verification. Catching §12 required
running `cargo test`, grepping `src/`, and cross-referencing
formula citations against source. That work surfaced one stale
section in USER-GUIDE I'd missed and confirmed several other
claims.

The audit shape is *iterate* — each round catches what the prior
round missed. Arc 043 is the verification round.

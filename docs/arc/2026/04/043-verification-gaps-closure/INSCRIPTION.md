# Arc 043 — INSCRIPTION

**Closed:** 2026-04-24.
**Commits:**
- `5b98f28` — DESIGN + BACKLOG opened
- `77d1573` — Slice 1: USER-GUIDE §12 + sigma formula
- `1ee1ec3` — Slice 2: README test surface (concrete counts)
- `29cd609` — Slice 3: src/*.rs doc-comment sweep
- `<this commit>` — Slice 4: INSCRIPTION + cross-references

## Why this arc existed

Builder asked after arc 042: *"your assessment is that wat-rs is
honest again?"*

Honest answer was *substantially more honest, not provably
honest*. Four follow-on checks were proposed; this arc executed
them:

1. Run `cargo test --workspace` and update README test counts.
2. Verify sigma formulas in USER-GUIDE / CONVENTIONS against
   `src/dim_router.rs`.
3. Sweep `src/*.rs` module-level doc comments.
4. Cross-doc consistency spot-check.

## What the verifications surfaced

### Test counts

`cargo test --release` from `wat-rs/Cargo.toml`:
- **725 Rust tests** across 25 binaries (lib units + integration
  suites).
- **~58 wat tests** across 11 wat-tests files (`wat-tests/holon/*`,
  `wat-tests/std/*`, `wat-tests/std/service/*`).
- Zero failures. Zero clippy warnings.

`wat_cache` was cited in README as one of the integration suites
— **the file does not exist** in `wat-rs/tests/`. Cache tests
externalized to `crates/wat-lru/tests/test.rs` per arc 013. Stale
reference removed in Slice 2.

### Sigma formulas

`src/dim_router.rs:188-205`:
```
DefaultPresenceSigma::sigma_at(d)
  = max(1, floor(floor(√d) / 2) - 1)
DefaultCoincidentSigma::sigma_at(_d) = 1
```

USER-GUIDE.md cited `floor(√d/2) - 1` correctly on the formula but
**missed the `max(1)` clamp**. Fixed in Slice 1.

CONVENTIONS.md mentions sigma functions abstractly ("functions of
`d`") with no specific formula — nothing to fix there.

### `src/*.rs` doc-comment audit

Eight doc-comment updates across four source files (Slice 3):
- 7 `wat::test_suite!` → `wat::test!` references in module-level
  doc comments and example blocks (`test_runner.rs` ×4,
  `bin/wat.rs` ×2, `panic_hook.rs` ×1). Arc 018 renamed the macro;
  these doc comments hadn't caught up.
- 1 `set-dims!` example in `harness.rs:26` runnable doc-comment
  example — dropped.
- 1 stale framing in `test_runner.rs:421`: comment said "set-dims!
  is a no-op" — actual arc 037 contract is **retired** (rejected
  at config collection time), not no-op. Fixed.

`cargo check --workspace --tests` confirmed the doc-comment edits
don't break any compilable code.

### Cross-doc consistency — surfaced more drift

The verification pass surfaced three drift spots in USER-GUIDE
§12 Error handling that arc 038's slice 5 had missed:
- Bundle return type still cited explicitly as
  `:Result<HolonAST, CapacityExceeded>` (should use the
  `:wat::holon::BundleResult` typealias from arc 032).
- Capacity-mode list showed all four (`:silent / :warn / :error
  / :abort`) — arc 037 retired `:silent` and `:warn`.
- `floor(sqrt(dims))` framing — under arc 037 the formula
  references the per-AST `d` picked by the active DimRouter.

All three fixed in Slice 1.

Plus a pre-commit drift catch: I had reached for a non-existent
primitive `:wat::core::i64::sqrt-floor` in Slice 1's gotcha
prose. Verified via grep — primitive does not exist; replaced
with description of the existing `CapacityExceeded /cost +
/budget` accessors that the Err path already uses.

## Honest disclosure

Arc 042's INSCRIPTION claimed: *"all wat-rs user-facing docs are
current through arc 037."*

That was **too optimistic**. USER-GUIDE §12 had three stale spots
arc 038's slice 5 didn't reach (capacity-mode count, BundleResult
typealias propagation, dims-framing). The verification pass
caught what the per-arc sweeps had missed.

The honest correction lives here, not in a retroactive edit to
arc 042's INSCRIPTION. **Historical records stay frozen.** Arc
042 was honest about what it knew at slice-close; arc 043
records what verification surfaced after.

## What this arc proves

**Verification finds drift assertion can't.** The audit shape is
*iterate*. Each pass catches what the prior pass missed. Per-arc
sweeps caught most; cross-doc verification caught §12; running
`cargo test` caught the test count and `wat_cache` straggler;
grepping `src/` caught the seven doc-comment macro names.

**Pre-commit drift checks pay out.** Two self-introduced errors
caught before push during this arc alone:
- The invented `:wat::core::i64::sqrt-floor` primitive (Slice 1).
- An originally-misframed Slice 3 commit message that would have
  said "set-dims! is a no-op" carrying forward the drift it was
  supposed to fix (caught while reading test_runner.rs:421's
  context).

Both verified via grep before any commit landed.

## Out of scope (still)

- **Lab `holon-lab-trading/CLAUDE.md`** — different repo,
  different domain. Lab-side arc.
- **arc 005 INVENTORY.md** — preserved per builder.

## Files touched

- `docs/USER-GUIDE.md` — Slice 1: §Config setters formula clamp,
  §12 BundleResult adoption, §12 capacity-mode count, §12 + §14
  `floor(sqrt(d))` framing.
- `README.md` — Slice 2: §Status test surface block.
- `src/test_runner.rs`, `src/bin/wat.rs`, `src/panic_hook.rs`,
  `src/harness.rs` — Slice 3: doc-comment sweep.
- `docs/arc/2026/04/043-verification-gaps-closure/{DESIGN,BACKLOG,INSCRIPTION}.md`
  — the arc record.
- `docs/README.md` — arc index extended.
- `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  — cross-repo audit trail row.

## The doc-audit set, finished honestly

After arcs 038-043 plus the `cargo test` verification:
- USER-GUIDE.md: synced through arc 037 (verified, including §12).
- README.md: synced through arc 037 (verified counts and suite list).
- CONVENTIONS.md: synced through arc 037.
- wat-tests/README.md: synced through arc 037.
- ZERO-MUTEX.md: synced through arc 037.
- src/*.rs doc comments: synced through arc 037.
- arc 005 INVENTORY.md: preserved as-is per builder.

**What the substrate ships is what the user-facing surface
describes.** Verified, not asserted.

The lab `CLAUDE.md` is the one remaining audit-set item. Lab-side
arc when builder calls.

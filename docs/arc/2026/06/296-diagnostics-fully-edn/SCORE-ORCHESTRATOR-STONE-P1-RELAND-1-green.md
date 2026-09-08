# SCORE (orchestrator) — STONE P-1 RELAND-1: GREEN

**Independent re-run.** Floor and clippy central, per FM 18/19.

```
FLOOR EXIT=0    Summary [ 190.089s] 5255 tests run: 5255 passed, 18 skipped
```

## The union, verified by reading — not by report

`src/declare/typevar.rs`, `first_unknown_named_type`:

```rust
if env.contains(p)
    || crate::runtime::is_builtin_primitive(stripped)
    || use_decls.covers(p)
    || env.is_subtype_parent(p)
{ return; }
```

```
grep is_reserved_prefix src/check.rs      ->  NOTHING.  Both blankets gone. STOP-1 held.
UseDeclarations::covers                       resolve/walk.rs:112 now CALLS it — one rule, not two.
both loops                                    intact; the functions half is `for (_name, func)`
                                              now that the skip that used `name` is gone.
```

## The 2 reds it arrived with, and their one root

```
intrinsic::tests::doc_arg_ret_types_match_checker_scheme
intrinsic::tests::probe_can_doc_types_reconstruct_the_checker_scheme

panicked at src/intrinsic/mod.rs:2429:13:
doc ret type for `:wat::edn::ForeignVariant/variant` says `:wat::core::Keyword`,
checker scheme says `:wat::core::keyword`
  left: ":wat::core::Keyword"   right: ":wat::core::keyword"
```

⛔ **This is the rider's own honest delta 1, as a floor red.** It wrote: *"The `@ret` comment in
`src/intrinsic/edn.rs` still says Keyword — not freeze-walked, not patched."* A GATE tests exactly
that consistency, and the sibling's failure text states the only two dispositions it accepts:

> *"either fix the doc, or record why the divergence is a real limit and add the name to
> `FROZEN_SPELLING_MISMATCHES` with its measured reason."*

★ **"Not patched" is not among them.** An honest delta that names an inconsistency a gate measures
is a red wearing a disclosure. Naming a hazard is not handling it.
`[[feedback_naming_a_hazard_is_not_handling_it]]`

## The fix, and why it was four lines and not one

The gate named ONE line. `:wat::core::Keyword` is the same phantom the wall caught five times in
fixtures, so I swept for every copy rather than silencing the gate:

```
src/intrinsic/edn.rs:350   prose   `→ :wat::core::Keyword`         -> keyword
src/intrinsic/edn.rs:361   @ret    :wat::core::Keyword             -> keyword   ← the gate's line
src/edn/render.rs:487      prose   `→ :wat::core::Keyword`         -> keyword
src/edn/render.rs:456      TypeMismatch { expected: ":wat::core::Keyword" }  -> keyword
```

★ `render.rs:456` is the one that mattered beyond tidiness: a **user-facing diagnostic naming a
type that does not exist.** No test asserted it, so no gate could ever have found it. Fixing only
the gate's line would have left it lying.
`[[feedback_a_patch_fixes_one_copy_of_a_claim]]`

Survivors after the sweep: zero, excluding P-1's own explanatory comment in `check.rs`.

## Rows

| # | expected | actual |
|---|---|---|
| 1–7 | the original seven `--check` EXITs | ✓ unchanged |
| 8 | `test(p1_annotation)` 10 passed, 0 skipped | ✓ |
| new | `use_rust_annotation` EXIT=0 | ✓ store 3 proven |
| new | `derive_marker_bound` EXIT=0 | ✓ store 4 proven |
| new | `rust_without_use` EXIT=1, `:rust::test::Greeting` | ✓ no `:rust::` blanket — STOP-1's real detector |
| STOP-4 | corpus refuses 0 after stores 3+4 | ✓ predicted 0, measured 0 |
| FLOOR | — | ✓ **5255/5255 after the Keyword sweep** |

## ⚠ CLIPPY — the number in the record was never floor.sh's

```
cargo clippy --release --all-targets -- -D warnings     EXIT=101     5 dead_code items
```

```
src/check.rs:6526      variant `Wildcard` is never constructed
src/check.rs:6923      fn pattern_coverage is never used
src/match_arm.rs:37    field `ident_span` is never read
src/runtime.rs:13538   fn try_match_pattern_ast is never used
src/runtime.rs:13724   fn substitute_many is never used
```

**NOT this stone's.** My first `nextest` run this session — at `53e3f9449`, which adds only test
files on top of the pushed `8022e21b7` — already printed *"`wat` (lib) generated 5 warnings"*,
naming `substitute_many`. Test files cannot create dead code in `src/`, so all five are already on
`origin/main`.

★★★ **And the seam's "clippy 0" was never measured by `scripts/floor.sh` — that script does not run
clippy at all.** It runs nextest and doctests. Every "clippy N" in this session's record came from
an ad-hoc `cargo clippy | grep -c` in the orchestrator's own command line, and across three
invocations that grep returned **0, then 7, then 8** for a tree whose real answer never changed.
A number with no owning instrument is not a measurement.
`[[feedback_an_instrument_must_outlive_the_number_it_produced]]` ·
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

Five dead-code items are a known flaw and are not left silently: they are the subject of their own
stone, drawn from `purgare`'s domain, not absorbed into P-1.

## Disposition

**LANDED.** Floor green, clippy state identical to the pushed baseline. Pushing.

## Named, carried forward — NOT absorbed here

- `is-type?` answers **false** for `:rust::sqlite::Connection` (use!'d in the same file) and for
  `:wat::spawn::Spawned` (a live derive marker). Measured. **Two contradictory answers to "is this
  a type?" now ship in one binary** — the wall's four stores, the verb's two. P-2 rests on this
  verb; it is P-2's prerequisite.
  `NOTE-is-type-shares-the-blindness-the-P1-floor-exposed.md`
- `derive` does not validate its MARKER (`types.rs:3410-3421`), so store 4 admits any name anyone
  derived from. Untested-as-written; a fixture is owed.
- The 5 dead-code items, and the clippy gate that has no owning instrument.

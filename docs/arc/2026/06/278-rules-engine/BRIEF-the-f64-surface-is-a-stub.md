# BRIEF — the f64 surface is a stub, and the totality column has two holes

Anchor at `/home/watmin/work/holon/wat-rs/`; verify with `pwd`; `git -C …` for git reads.
Tree clean at HEAD `c59b2dca`. Floor **`4348 / 4348 / 0 / 262`**, clippy clean,
`check-where-shapes.sh` → `9 pair(s), 98 rows`.

## The work in one paragraph

A rule cannot do anything with a float. The rete surface exposes **14 i64 rows and 2 f64 rows** —
and both f64 rows landed this morning. There are no f64 comparators at all, so
`(where (:wat::rete::f64::> score 0.8))` — the canonical rule of this engine's entire target use
case, rules over streaming anomaly scores — is inexpressible. This mints the four f64 comparator
rows, fixes two holes in the totality column that block them, corrects a casing bug the orchestrator
introduced this morning, and re-points four rows at doors that now exist.

## Read in order

1. `src/rete/vocabulary.rs` — `RETE_OPS` (45 rows). The i64 comparator rows
   (`:wat::rete::i64::{> < >= <=}`) are your exemplar; copy their shape exactly. Note the two
   distinct classes: comparators/equality/conversions are `OpClass::Alias`, arithmetic is
   `OpClass::Fallback`. **You are minting Aliases only.**
2. `src/rete/purity.rs` — the `let total = matches!(` at **line 515** (NOT the one at 186, which is
   scoped to `:wat::core::string::`/`regex::` and returns immediately; reading the wrong one cost the
   orchestrator three wrong claims this morning).
3. `src/rete/purity.rs` — the comment block immediately above line 515, from
   `BRIEF-total-column-honest.md` (#52). It states the builder's stricter-than-IEEE totality rule and
   explains why `f64::>` is total: *"it is a comparison whose OUTPUT is a bool, never itself the
   undefined value."* That sentence is the whole warrant for hole 1.
4. `src/runtime.rs:~5188-5220` — the generic and per-type comparator dispatch, for hole 2's grounding.

## Part A — hole 1: three f64 comparators are missing from `total`

The `total` list at `purity.rs:515` contains `i64::>`, `i64::<`, `i64::>=`, `i64::<=` — all four —
but of the f64 family only **`f64::>`**. `f64::<`, `f64::<=`, `f64::>=` are absent.

They are pure ∧ det already (they sit in that list). They are total by the file's own stated reason:
each is a comparison whose output is a bool, and `eval_f64_compare` is NaN-correct — `NaN > 1.0` is
`false`, not a raise. There is no input on which they fail to produce an ordinary bool.

**Add the three.** Put them beside `f64::>` with a comment citing this brief.

Why they were missed: #52's own STOP-3 read *"do not widen the audit past entries already `true`"*,
so it swept false-trues and never revisited entries already `false`. A correct scope that left its
mirror image untouched.

## Part B — hole 2: generic `<` is marked total and is NOT

`":wat::core::<"` is in the `total` list. Its three siblings `>`, `>=`, `<=` are not. All four
dispatch to the same `eval_compare` (`runtime.rs:5191-5194`), which returns
`RuntimeErrorKind::TypeMismatch` when `values_compare` yields `None` — the incomparable-operands
domain hole that `DESIGN-STONE-where-admits-only-rete-ops.md` cites as the whole reason per-type
comparison exists:

> *"Generic `>` is PARTIAL. Its domain hole is 'these two operands are not comparable.'
> Monomorphising … deletes the domain hole."*

Generic `<` has exactly that hole and is marked total. **Remove it.** This is a false-true — the
dangerous direction — and the odd one out of four.

**⚠ GROUND THE BLAST RADIUS BEFORE YOU CUT.** `total` is **user-visible** (`freeze.rs:728` registers
the axis so a wat program can query it) but is **NOT** in the fence's conjunction — `classify_fn`'s
pure/det helpers are `purity.rs:1129/1134`; `Axis::Total`'s helper at `:1141` is separate. So this
should be inert to `compile-condition`. **Verify that by running, not by trusting this paragraph.**
If removing it turns anything red, STOP and report — a red here is a finding, not an obstacle.

## Part C — mint the four f64 comparator rows

All `OpClass::Alias`, `params: &[F64, F64]`, `ret: ParamType::Bool`, `type_params: &[]`,
`meta: OpMeta { pure: true, deterministic: true, total: true }`.

| rete_name | core_name |
|---|---|
| `:wat::rete::f64::>`  | `:wat::core::f64::>`  |
| `:wat::rete::f64::<`  | `:wat::core::f64::<`  |
| `:wat::rete::f64::>=` | `:wat::core::f64::>=` |
| `:wat::rete::f64::<=` | `:wat::core::f64::<=` |

All four core targets are registered and dispatched — verified 2026-08-05 at `check.rs:~15878` and
`runtime.rs:5214-5217`. Unlike this morning's round-1c brief, this table has been checked against the
disk; if any row is wrong, the disk is the arbiter and you should say so.

## Part D — the casing bug (the orchestrator's, shipped in `6d5af2c8`)

Round 1c minted `:wat::rete::String::=` and `:wat::rete::String::not=` with a **capital S**. Every
other string row in both surfaces is **lowercase**: `:wat::core::string::{length,concat,trim,…}` and
`:wat::rete::string::{length,to-lowercase,trim}`. The name was derived from the *type* (`String`)
instead of the *module* (`string`).

**Rename both rows to `:wat::rete::string::=` / `:wat::rete::string::not=`.** No consumers exist yet
— they are hours old and the fence is unarmed — so this is a rename, not a migration. Confirm zero
call sites before you rename (`grep -rn 'rete::String::' --include=*.wat --include=*.rs .`) and say
what you found.

## Part E — re-point the four equality rows

`c59b2dca` restored `:wat::core::{i64,f64}::{=,not=}`. The round-1c rows currently route to the
generic `:wat::core::=` / `not=` because the per-type doors did not exist. **Re-point them:**

| rete_name | core_name was | core_name becomes |
|---|---|---|
| `:wat::rete::i64::=`    | `:wat::core::=`    | `:wat::core::i64::=`    |
| `:wat::rete::i64::not=` | `:wat::core::not=` | `:wat::core::i64::not=` |
| `:wat::rete::f64::=`    | `:wat::core::=`    | `:wat::core::f64::=`    |
| `:wat::rete::f64::not=` | `:wat::core::not=` | `:wat::core::f64::not=` |

`string::`, `bool::` and `keyword::` rows keep the generic — core has no per-type equality for those
and this brief does not mint any (that is STOP-2).

Update the block comment above the round-1c rows: its explanation that the per-type targets "do not
exist" is now false, and a stale comment is a lie the next reader inherits.

## ⛔ STOPs — rejection criteria

- **⛔ STOP-1 — Aliases only.** No `OpClass::Fallback` rows. f64 arithmetic (`+ - * /`) reaches ±Inf
  and needs the `:undefined` carrier; it is a separate, larger stone. Do not mint it here.
- **⛔ STOP-2 — mint no new `:wat::core::` verb.** Every core target in this brief already exists.
  If you conclude one is missing, STOP and report — that contradicts a grounded table and the
  orchestrator owns the re-scope.
- **⛔ STOP-3 — do not touch `RETE_MODULES`.** Admission is a separate, unarmed axis.
- **⛔ STOP-4 — do not arm anything.** The fence stays as it is.
- **⛔ STOP-5 — if removing generic `<` from `total` turns any test red, STOP and report** with the
  failing test's name and its assertion. Do not "fix" the test to accommodate the cut; a red there
  means something depends on the false-true and that is the orchestrator's call.
- **⛔** Do not add a `_` wildcard arm on an enum scrutinee.
- **⛔** Do not commit, stash, push, or touch git.

## Verify — FOREGROUND, block, run the suite SOLO

```
cargo build --release
cargo nextest run --release          # no other cargo process alive
cargo clippy --release --all-targets
./wat-scripts/perf/grid/check-where-shapes.sh
```

Floor **`4348 / 4348 / 0 / 262`** — vocabulary and classification only; no test may move.
Gate **`9 pair(s), 98 rows`**. Read the **Summary line**, never a piped exit code.

## EXPECTATIONS — written before the strike

| # | what | expected |
|---|---|---|
| 1 | row count | **49** (45 + 4) |
| 2 | ★ **a float rule is now expressible** | `(:wat::rete::f64::> 0.9 0.8)` → `true`; `(:wat::rete::f64::<= 0.1 0.2)` → `true` |
| 3 | ★ **the domain hole stays deleted** | `(:wat::rete::f64::> 1.0 1)` is a **type error at `--check`**, exit 1 |
| 4 | ★ **NaN is total, not a hole** | `(:wat::rete::f64::> nan 1.0)` → `false`, no raise. Build the NaN by computation (e.g. `0.0 / 0.0` via a core f64 op), NOT by a literal |
| 5 | ★ **non-vacuity** | a bogus `:wat::rete::f64::>X` raises a located `UnknownFunction` **at runtime** — `--check` does not validate `:wat::*` heads |
| 6 | the casing rename | `grep -c 'rete::String::'` → **0**; `:wat::rete::string::=` resolves and runs |
| 7 | the re-point | the four equality rows name `:wat::core::{i64,f64}::{=,not=}`; each still returns the right boolean |
| 8 | ★ hole 1 closed | `f64::<`, `f64::<=`, `f64::>=` are in the `total` list at `purity.rs:515` |
| 9 | ★ hole 2 closed | `:wat::core::<` is NOT in it, and **nothing went red** |
| 10 | ★ floor | `4348 / 4348 / 0 / 262` exactly |
| 11 | ★ gate | `9 pair(s), 98 rows — wat == Clara on every shape` |
| 12 | clippy | clean |

Rows 2, 3, 4, 5, 9, 10, 11 re-run by the orchestrator by hand.

**Runtime prediction: 30–45 minutes.** Time-box 90.

**Trap doors:**
1. **Reading the `total` list at line 186** instead of 515. 186 is the string/regex branch and
   returns immediately. Cost the orchestrator three wrong claims this morning.
2. **Trusting a line number across an edit.** `purity.rs` moved +17 lines this morning and a citation
   taken before that shift was quoted after it. Re-grep; do not carry a number.
3. **Trusting `--check` for row 5** — it does not validate `:wat::*` heads at all.
4. **A NaN literal.** Build NaN by computation; a literal may not parse and proves nothing about the
   runtime path.
5. **Assuming Part B is inert.** It is argued to be inert from `freeze.rs:728` and
   `purity.rs:1129/1134/1141`. Argued is not measured. Run it.

## Scratch

Any scratch `.wat` goes in `wat-scripts/scratch-pad/` — never a tmp dir. That directory is parsed and
type-checked by the `every_wat_scripts_file_loads` gate on every build, which is the point: a scratch
program that rots goes RED instead of becoming a graveyard that reads like live code. Write a real
program with a `:user::main`; a probe without one fails before resolving anything and proves nothing.

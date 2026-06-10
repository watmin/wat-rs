# DESIGN — Stone 258.1: `if` inference (the keystone)

**Status: STRIKE-READY target. Bare `(if cond then else)` infers; the conditional is the only branch point.**

## Contract (pinned)

`(:wat::core::if cond then else)` — 3 args (cond, then, else) — type-checks and evaluates with NO
return annotation:
- `cond` must infer to `:wat::core::bool` (unchanged).
- `then`/`else` are inferred; **`unify(then_ty, else_ty)`** gives the form's type. They must unify
  (strict — same discipline the declared type enforced today); a mismatch is a `TypeMismatch`
  naming both branch types, NOT an arity error.
- the consuming site does the rest via recipient `assignable` (the `do` model).

**Dual-read:** the 5-arg `(:wat::core::if cond -> :T then else)` keeps its current behaviour
(branches `unify` against the declared `:T`) so the corpus stays green until the 258.3 sweep.

## Sites

- **`src/check.rs` `infer_if`** (~7120): branch on `args.len()` — `4`/`5` → existing annotated path
  (keep); `3` → NEW inference path (infer both branches, unify, return the join; cond must be bool).
  Replace the `len()==3 → "now requires -> :T"` error with the inference path.
- **`src/runtime.rs` `eval_if`** (~5874) **+ `eval_if_tail`** (~2773) **+ `step_if`** (~21138, macro
  engine): accept 4 args `[cond, then, else]` alongside 5. Drop the `len()==3 → MalformedForm`
  rejection; dispatch on length.

## Out of scope = rejected

- `cond` (258.2 — reborn as a macro, not patched here).
- The corpus sweep (258.3) and the 5-arg hard-cut (258.4).
- LUB / subtype-join for divergent branches — branches `unify` (strict); divergent-subtype joins are
  not a supported class (they never were — the annotation used `unify` too).

## Probe (RED at HEAD)

`tests/probe_arc258_stone1_if_inference.rs`:
- C01: `(if true 1 2)` evals to `1` — bare if works. (RED at HEAD: 3-arg → "now requires `-> :T`".)
- C02: `(if false -> :i64 1 2)` evals to `2` — annotated dual-read still works. (GREEN at HEAD.)
- C03: `(if true 1 "s")` fails to check, and the error is a **branch mismatch, not arity** (asserts
  the message does not contain "now requires"). (RED at HEAD: it's rejected for arity.)

## Gate

- `cargo test --release --test probe_arc258_stone1_if_inference` → 3/3 (RED at HEAD).
- `cargo build --release` clean; full suite: only the 4 known nursery deadlock-reds.

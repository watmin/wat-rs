# DESIGN — STONE: `defclause` dispatch joins `src/function/` — the home was minted for it

> **Builder, 2026-09-01:** *"next module."*
>
> numeric 892 · declare 3,506 · reflect 2,173. `runtime.rs` is **27,584**. This takes **~1,637**.

## ★ THE HOME ALREADY EXISTS, AND IT WAS MINTED FOR EXACTLY THIS

`src/function/mod.rs`, its own opening line:

> *"Stone 241.18a — Mints `src/function/` as the dedicated namespaced home for the **fn-form**"*

```
src/function/parse.rs     410   parse_fn_signature · parse_fn_signature_with_rest · …
src/function/eval.rs      102   eval_fn
src/function/infer.rs     251   infer_fn
src/function/metadata.rs   78   peel_type_binder
```

A `defclause` **is** the fn-form's multi-clause shape, and its parser/selector have been squatting in
the megafile while the home for them stood half-empty. ★★ Unlike `numeric`/`declare`/`reflect`, this
stone mints **no new home** — it fills one out, and the moved items land in files that already exist
by the same role split.

## What moves — 12 items, `3627–5263`

```
parse   3627 parse_defclause_clause · 3908 mod arc109_two_iii_defclause_return_slot (#[cfg(test)])
        3980 parse_defclause_form · 4228 parse_extend_type_form · 4566 parse_derive_form
        4632 is_defclause_form
eval    4645 eval_call_to_defclause · 4675 select_defclause_clause
        4888 eval_call_to_defclause_with_vals
subsume 4998 declared_type_subsumes · 5031 value_matches_type_by_name · 5170 val_type_path
```

⛔ **`eval_let` (5264) is the boundary and is NOT defclause.** The block ends at `val_type_path`.
Fourth stone running, same hazard: **move by the list, never by span.**

★ `arc109_two_iii_defclause_return_slot` is a `#[cfg(test)] mod` whose `use super::` names
`parse_defclause_clause` — it is that function's own probe and travels with it.

## ★★ THE BLAST RADIUS IS 9 SITES — AND 4 OF THEM ARE A CYCLE EDGE THE LAST STONE CREATED

```
src/check.rs             5
src/declare/register.rs  4    <- `crate::runtime::{parse_defclause_form, parse_extend_type_form}`
```

The declare stone had to import those two from `crate::runtime` because they had no home. Moving
them to `src/function/` **dissolves that edge as a side effect** — `declare → runtime` loses two of
its genuine (non-facade) references. ⚠ Not the stone's purpose, but worth measuring after: the crate
campaign's blocker is genuine edges, and this stone removes some without trying.

## THE ONE CONTRACT DECISION — pinned

**Moved items land in `src/function/`'s EXISTING files by role, not in a new `defclause.rs`.**
Parsing goes with parsing, evaluation with evaluation. A `defclause.rs` would re-create inside the
home the very "grouped by the form it came from" split the home exists to avoid — `src/function/` is
organized by ACT (parse / eval / infer / metadata), and this stone honours that.

The one exception is `subsume`: `declared_type_subsumes` · `value_matches_type_by_name` ·
`val_type_path` are **runtime** type-matching for clause selection, not check-time inference. They
are not `infer.rs`'s concern and get their own file. ⚠ **The rider verifies this against the bodies
and reports if it does not hold** — `reflect`'s `verbs.rs` shipped 12 of 13 because exactly this kind
of assignment turned out wrong.

## ★ THE PREDICTION — falsifiable

```
runtime.rs        27,584  ->  ~26,000   (-1,600)
src/function/     904  ->  ~2,500 lines, still 5-6 files
declare -> runtime   loses 2 genuine references
9 call sites      crate::runtime::X -> crate::function::<role>::X
eval_let          UNTOUCHED, still in runtime.rs
behaviour         every defclause call selects the same clause
```

## Out of scope = REJECTED (not deferred)

- **`peer_protocol`** (~3,000, three non-contiguous ranges). ★ Its home also exists — `src/kernel/`
  already holds `peer.rs`, `listener.rs`, `address.rs`, `spawn.rs`. It is the bigger prize and the
  next stone; three ranges against one block is not the shape to take second.
- **`stepper` · `pattern_matching` · `quasiquote` · `kernel_signal`.** ⚠ Spans stale again after
  this stone; each re-derives.
- **`eval_let` / `bind_let_binding` / `eval_do`** — the eval spine, adjacent by line only.
- **The facade sweep**, numeric stone 2's lattice, and `src/macros/` → `src/expand/`
  (`[[NOTE-RULED-src-macros-becomes-src-expand]]`, deliberately timed for just before the crate move).

## THE FOUR QUESTIONS — flat YES/NO

| option | Obvious? | Simple? | Honest? | Good UX? | verdict |
|---|:---:|:---:|:---:|:---:|---|
| **into `src/function/`'s existing files, by role** | YES | YES | YES | YES | ✅ **ADMITTED** |
| a new `src/function/defclause.rs` holding all 12 | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| a new top-level `src/defclause/` | **NO** | YES | **NO** | — | ⛔ **DISQUALIFIED** |
| take `peer_protocol` first because it is bigger | YES | **NO** | YES | — | ⛔ **DISQUALIFIED** |
| take the whole `3627–5600` span | YES | YES | **NO** | — | ⛔ **DISQUALIFIED** |

- **one-`defclause.rs` Honest? NO** — it groups by the FORM the code came from inside a home
  organized by ACT, re-creating the split the home exists to prevent.
- **top-level `src/defclause/` Obvious? NO / Honest? NO** — a defclause is the fn-form's multi-clause
  shape; giving it a sibling home to `src/function/` asserts they are different domains, and the
  parser proves otherwise (`parse_defclause_form` and `parse_fn_signature` read the same shapes).
- **`peer_protocol`-first Simple? NO** — three non-contiguous ranges; one clean block first.
- **whole-span Honest? NO** — it takes `eval_let`, `bind_let_binding`, `eval_do`: the eval spine,
  adjacent by line only. This is the trap that has appeared in every stone of this campaign.

## Acceptance

| what | command | expected |
|---|---|---|
| ★ `eval_let` did not move | `grep -c "fn eval_let" src/runtime.rs` | **1** |
| the megafile sheds it | `wc -l src/runtime.rs` | ~26,000, from 27,584 |
| no new home minted | `ls src/function/` | existing files + at most one new (`subsume`) |
| ★ the cycle edge shrinks | `grep -c "crate::runtime::" src/declare/register.rs` | **fewer than before** |
| no facade imports | new/edited `use` blocks | `crate::value::` direct |
| the impl does not know its edge | `grep -c "crate::intrinsic" src/function/*.rs` | 0 |
| behaviour unchanged | every `defclause` call | selects the same clause |
| floor | `scripts/floor.sh`, exit read UNPIPED | 5114/5114, 0 failed |
| clippy | `cargo clippy --release --all-targets -- -D warnings` | 0 |

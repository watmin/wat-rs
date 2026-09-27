# WEIGH — STONE 255.56: STOP-2 — ACCEPTED AS A STOP (nothing switched)

**Executor: grok via pulsare, commit `0197212c8`** (the SCORE only; `src/` and `wat/` untouched; pre-census taken).
Weighed by the orchestrator on 2026-09-27.

## The stop, weighed

- **`wat/doctest.wat:118` `(= got want)` is a genuine STOP-2.** Both operands are `:Ok` payloads of `:wat::eval-ast!`,
  whose `T` is fresh per call (`src/check.rs:20540-20562`). A doc example evaluates to whatever it returns, so no one
  type can be pinned. After `unify` both sides are the same unresolved variable, and Refuse refuses it. **Needs a
  ruling.**
- **`wat/rete/acc.wat:68`/`:87` were my brief's error, not a Refuse site.** They are `(< v cur)` / `(> v cur)`, and
  `infer_ordering` unifies the operands first, so `v` becomes `i64` from `cur`. 255.54 recorded the operand before that
  unify. The latent issue is real but belongs to rete, not this stone: `Element.bindings` is a bare `PersistentMap`
  (`wat/rete.wat:40`), so `v` is `i64` only by that unification, and a non-`i64` accum result would reach
  `values_compare` and get `None`. Carried to the rete raise-fence work.
- **The three `tests/resolve/…fix_source_local_rules__contract-0{6a,6b,7}` files are golden output text**
  (`tests/resolve/probe_arc251_fix_source_local_rules.rs:88-107`: `assert_eq!` against `include_str!`). Only the
  census type-checks them. After Refuse their census rc flips to 1. STOP-1 named them for the builder.

Grok's reading of the other sites (`list.wat:46`, the record round-trips, uuid, char) is right: each unifies with a
concrete operand and stays admitted.

## Rulings owed

- The doctest comparison.
- The golden fixtures.

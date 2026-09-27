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

## Resume weighed (2026-09-27): STOP-1 accepted; `6225697a9` held local (the floor is red)

Grok built the switch (the gates ask the classes; Refuse; E-a; Z1; the predicates deleted, ledger 198 → 195; the
bounds on `assert-eq`/`dedupe-walk`/`dedupe`; the `ord_result_*` pins; `Pt`/`HPt` moved to a `.wat.bad`; the
doctest's `matches?`). Then it stopped on 13 floor reds outside step 5. Read by the orchestrator:

1. **`nil` is refused as `:()`.** `wat-tests/bracket.wat:40`/`:65` `(assert-eq (each …) nil)`. The `nil` edge is registered
   on the **alias path**. By the bound check the type is its expansion `Tuple([])`, and Z1's one-or-more `:..` refuses
   that. **There is no separate "empty tuple" type:** `(Tuple)` cannot be built, so `:()` *is* `nil`. Z1's intent
   (`nil` Equatable, not Orderable) is right. The edge must be alias-transparent, registering on what the alias
   names. That is an implementation fix, not a new ruling.
2. **`:test::Wrapper` is a `defstruct` round-tripped through EDN** (`wat-tests/edn/roundtrip.wat:20`, `:75`, `:86`)
   and compared with `assert-eq`. Q1 makes structs non-`Equatable`. **Needs the builder:** the test's type is data,
   so should it be a record?
3. Three scratch files compare `_` operands (`eval-ast!` results, `struct-field`). Each is pinned by a typed consumer,
   as the doctest was. `probe-eq-generic-instantiation.wat` now **correctly** refuses `eq-generic` on two functions;
   that call is the hole this stone closes. It moves to a `.wat.bad` asserting the refusal.
4. The census's nine 0→1 flips are the three goldens (expected) plus the six files above.

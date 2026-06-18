# DESIGN — Stone 0d: transform-op check-side parity (the vector disparity)

> Arc 278 stone 0, part d — close the LAST persistent-collection gap. 0c gave `PersistentVector` the
> transform/sequence ops at **runtime** (the `eval_vec_*` arms). But the **checker** never followed: the
> transform ops are still monomorphic `Vector`-only static `TypeScheme`s, so `(foldl f init pv)` in a typed
> body fails to type-check. Surfaced building 1b `compile`, where the worker had to route around it with
> `(foldl f init (range 0 n))` + `PersistentVector/get` indexing. Builder: *"we gotta fix the vector
> disparity."* This stone makes the checker agree with the runtime 0c already shipped.

## The gap (grounded)

- **Runtime: DONE.** Every collection op routes a single generic head to one internal-dispatching eval fn —
  `:wat::core::foldl → eval_vec_foldl` (runtime.rs:4341), `map → eval_vec_map` (4340), etc. 0c taught each
  `eval_vec_*` to handle `Value::wat__core__PersistentVector` (commit `8f671452`). No per-Type runtime heads
  do the work; the generic head dispatches by value-tag. So `(foldl f init pv)` **runs** correctly today.
- **Checker: monomorphic Vec-only.** `map`/`filter`/`foldl`/`foldr`/`reverse`/`take`/`drop` are registered as
  static `TypeScheme`s with `vec_of(t_var())` = `wat::core::Vector` ONLY (check.rs:17963-18073). `concat` checks
  via the `:wat::core::Vector/concat` scheme (alias from core.wat:44). None accept `PersistentVector`. So a
  typed body that folds/maps a `PersistentVector` is rejected at check time — the disparity.

## The decision (four-questions — the instrument, not a menu)

**How the 8 transform ops type-check on `Vector<T> | PersistentVector<T>`.**

*Hard constraint, weighed first:* honest polymorphism — both collections first-class at the checker,
type-preserving, checker agrees with the shipped runtime, no union-hack. Both candidates clear it → go to the four.

**A — custom infer arms** (`infer_map`/`infer_foldl`/… in `collection/infer.rs`, dispatched from the
keyword-head match, static Vec-only schemes retired):
- **Obvious? YES** — the exact pattern `infer_conj`/`infer_get`/`infer_assoc` already use (check.rs:5136-5205).
- **Simple? YES** — one infer fn per op, each knowing its own collection-arg position; no new machinery; mirrors
  the runtime's one-eval-fn-per-op.
- **Honest? YES** — genuine `Vector|PersistentVector` acceptance, type-preserving; closes the exact gap 0c left.
- **Good UX? YES** — `foldl`/`map`/etc. type-check on every collection; the 1b range-get workaround dies.

**B — extend the Dispatch machinery with a per-op dispatch-arg index** (so fn-first ops join the formal
receiver-first Dispatch family):
- **Obvious? NO** — introduces a configurable dispatch-arg position that does not exist today.
- **Simple? NO** — net-new substrate for ops the runtime dispatches without it.
- Disqualified on Obvious + Simple before UX is weighed.

→ **A, flat.** The transform ops have *mixed* collection-arg positions (`reverse`/`take`/`drop` receiver-first;
`map`/`filter`/`foldl`/`foldr` fn-first; `concat` two-collection) — every existing Dispatch entity is
receiver-first, so B would have to grow new machinery for ops a per-op custom arm handles directly. A is the
honest fit because each arm knows its own op's shape. This matches the doctrine partition (check.rs:5144-5167):
these are PROJECTIVE intrinsics (`C<T> × fn → C<U>`), and the partition explicitly routes projective ops to
`infer_<op>` and says **"DO NOT make these clauses."**

## The work — 8 projective custom infer arms in `src/collection/infer.rs`

Mirror the existing `infer_conj`/`infer_get`/`infer_assoc` shape (collection/infer.rs:118-485): extract the
collection arg's parametric type, accept `Vector<T>` OR `PersistentVector<T>`, project the result
type-preservingly, emit a teaching `TypeMismatch` for any other shape. Dispatch each from the keyword-head
match in check.rs by a `return`-early arm (mirror 5172-5205). **Retire the static Vec-only `TypeScheme`** for
each (check.rs:17963-18073) — like conj/get/assoc, the arm becomes the single source of truth (no scheme +
arm to drift).

| op | call shape | coll arg | check-side signature (C ∈ {Vector, PersistentVector}) |
|---|---|---|---|
| `map`     | `(map f xs)`       | arg[1] (fn-first) | `C<T> × fn(T)->U → C<U>` (elem T→U, container preserved) |
| `filter`  | `(filter pred xs)` | arg[1] (fn-first) | `C<T> × fn(T)->bool → C<T>` |
| `foldl`   | `(foldl f init xs)`| arg[2] (fn-first) | `fn(Acc,T)->Acc × Acc × C<T> → Acc` |
| `foldr`   | `(foldr f init xs)`| arg[2] (fn-first) | `fn(T,Acc)->Acc × Acc × C<T> → Acc` |
| `reverse` | `(reverse xs)`     | arg[0]            | `C<T> → C<T>` |
| `take`    | `(take xs n)`      | arg[0]            | `C<T> × i64 → C<T>` |
| `drop`    | `(drop xs n)`      | arg[0]            | `C<T> × i64 → C<T>` |
| `concat`  | `(concat a b)`     | arg[0]+arg[1]     | `C<T> × C<T> → C<T>` (SAME kind both sides; mixed → TypeMismatch) |

`concat` is the one wrinkle: it currently checks through the `concat → :wat::core::Vector/concat` alias
(core.wat:44) + that impl's scheme, not a surface arm. **Grounding sub-step (in the BRIEF):** confirm whether
`concat`/`Vector/concat` can take a surface custom arm like the others, or whether the parity belongs on a
`:wat::core::PersistentVector/concat` impl scheme paralleling `Vector/concat`. Implement whichever the alias
path actually supports; if it is structurally different from the other 7 → STOP and report (do not invent).
Either way the end state is identical: `(concat pv pv) → PersistentVector`, same-kind-only, mirroring the
runtime 0c shipped (eval.rs `vector_concat_inner` rejects mixed kinds).

## The ONE contract decision

**Type-preserving polymorphism over exactly `{Vector, PersistentVector}`; each arm is total (no static scheme
behind it); `concat` is same-kind-only.** A `PersistentVector` in → a `PersistentVector` out (map/filter/
reverse/take/drop/concat); `foldl`/`foldr` return the accumulator. The checker exactly mirrors the runtime
0c shipped — no new behavior, only the type-level agreement that was missing.

## Out of scope (affirmative cuts)

- **`sort'`** — 0c did NOT give it a PersistentVector runtime arm, so checker parity would outrun the runtime.
  Stays Vec-only; a separate stone if ever needed.
- **`range`** — monomorphic `i64 × i64 → Vec<i64>` by construction; nothing to make polymorphic.
- **`List<T>`** — not part of the 0c persistent-collection set; this stone is `Vector | PersistentVector` only.
- **The formal `:Seq`/`:Map` defprotocol** — arc 285 (Layer 2), still deferred. This is Layer-1 op-parity at
  the checker, which IS the "consistent access" the engine needs.

## Proof (FM-2-bis — RED at HEAD)

`tests/probe_arc278_0d_transform_dispatch_parity.rs` (RED, un-ignore on green): a `fn check(src) -> Result<(),
String>` harness (as in `probe_arc256_generic_defclause.rs`) feeds a typed `defn` whose body runs a
`PersistentVector` through each of the 8 ops (`foldl`/`map`/`filter`/`foldr`/`reverse`/`take`/`drop`/`concat`)
with the correct return-type annotations, and asserts the program **checks clean**. RED at HEAD: at least the
first `(foldl f init pv)` raises a `TypeMismatch` (scheme expects `Vector`, got `PersistentVector`). GREEN when
the 8 arms land. (A second case asserts a genuine wrong-element call is still REJECTED — parity must not become
permissiveness.)

## Four questions (the stone)

- **Obvious?** YES — "the transform ops type-check on PersistentVector the same way they already run on it."
- **Simple?** YES — 8 arms mirroring 4 existing siblings; retire 8 schemes; no new machinery.
- **Honest?** YES — genuine polymorphism (the probe runs each op AND keeps wrong-element rejection); checker
  finally matches the runtime; no crutch.
- **Good UX?** YES — every collection op works uniformly at check + runtime; the range-get workaround is gone.

## Blast radius

`src/collection/infer.rs` (8 new projective infer fns) · `src/check.rs` (8 `return`-early dispatch arms in the
keyword-head match; retire the 8 static Vec-only schemes at 17963-18073) · maybe `wat/core.wat` /
`src/check.rs` for the `concat` alias path (per the grounding sub-step) · the probe. **NO runtime change**
(0c already did it). NO new dispatch machinery. No git in the worker.

## Realized (post-build, 2026-06-18) — deviations + residual

- **SHIPPED**: 8 projective infer arms in `collection/infer.rs` + 8 `return`-early dispatch arms in `check.rs`;
  `concat` enforces same-kind-only in code (mixed Vec+PV → TypeMismatch); all type-preserving. Probe 2/2;
  lib 931/36, deftest 264/1, load-order 1/0 — all unchanged (own re-run). Build: 25 warnings, unchanged.
- **DEVIATION (STOP-2 resolved, not retired-as-planned):** the DESIGN said retire all 8 static schemes.
  Retiring `map`/`foldl`/`foldr`/`filter` broke `:wat::list::reduce`/`fold` — those are `defalias` whose
  Function derivation (Case 2 in `register_defalias`) reads the TARGET's static scheme. So those 4 schemes are
  **RETAINED** (only `reverse`/`take`/`drop` retired). No double-truth for call sites: the keyword-head arm
  `return`s before `env.get`, so direct `(foldl …)` always goes through the arm; the retained scheme serves
  ONLY alias derivation. Documented with WHY-comments in `check.rs`.
- **RESIDUAL (banked, out of scope):** because aliases derive from the Vec-only scheme, `:wat::list::reduce`
  and `:wat::list::fold` over a `PersistentVector` stay REJECTED — the alias-of-a-projective-intrinsic seam:
  an alias cannot inherit the arm's polymorphism through static-scheme derivation. The rete engine uses
  `foldl` directly, so this does not block 1b. **Follow-on (build only if a real call forces it):** give
  projective-op aliases PV parity — either route the alias surface name through the same custom arm, or teach
  the defalias Case-2 derivation to defer to the arm. Not the formal Seq/Map defprotocol (arc 285); a narrower
  alias-derivation seam.
- **IDE diagnostics were line-shift artifacts:** the 71-line insertion at check.rs:5203 shifted every line
  below, so rust-analyzer reported pre-existing warnings (`head_span` at 11421/13711) as "new" and hadn't
  reconciled the new `infer_*` call sites (flagged them dead). cargo is authoritative: the arms are used; 25
  warnings, unchanged. ([[feedback_ide_diagnostics_can_lie]])

## Sequencing

1. Revert the incomplete 1b from the tree (its child-edges were deferred — it re-strikes after this anyway,
   and its range-get foldl is obsoleted by this stone).
2. This stone (0d): DESIGN (here) → RED probe → BRIEF + EXPECTATIONS → single-hop sonnet → weigh → commit.
3. Re-strike 1b clean: direct `foldl` over the `PersistentVector`, child edges wired, probe strengthened to
   prove the chain (alpha→root-join→production), not just the alpha count.

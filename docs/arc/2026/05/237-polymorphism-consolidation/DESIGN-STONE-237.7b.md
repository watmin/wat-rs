# Stone 237.7b — collection-op define-dispatch → ∀T intrinsics (the rest)

**Follows 237.7a** (`length` = ∀T intrinsic, `8100d9d2`). Same doctrine
(`DESIGN-STONE-237.7-intrinsic-kill.md`): the collection ops dispatch on generic
container heads, which a defclause (closed type universe, no `:Any`) can't host —
so they become Rust `∀T` **intrinsics** (the `:wat::core::type` / `length` shape).
`define-dispatch` evacuates one op at a time; the `DispatchRegistry` itself is
deleted in 237.7c (after arithmetic 237.8).

## Crawl (ground truth, 2026-05-27)

`wat/core.wat` — FOUR `define-dispatch` collection ops remain (length already gone):

| op | clauses (coll → leaf) | shape |
|---|---|---|
| `empty?` | Vector/HashMap/HashSet → `*/empty?` | `∀T. coll -> bool` |
| `contains?` | Vector(T)/HashMap(K)/HashSet(T) → `*/contains?` | `(coll, elem) -> bool` |
| `get` | Vector(i64)/HashMap(K) → `*/get` | `(coll, key) -> Option<element>` |
| `conj` | Vector(T)/HashSet(T) → `*/conj` | `(coll, elem) -> coll` (type-preserving) |

`assoc` is **NOT** a define-dispatch — it is a single-impl **alias** (HashMap only,
`core.wat` aliases block, arc 146 slice 4). The 237.7-kill DESIGN wants it
promoted to **multi-impl (HashMap + Record)** = a behavior change tied to the
records/typed-entities doctrine. Different shape → its own slice.

The **exemplar** (length, proven): `check.rs:19610` registers
`TypeScheme { type_params:["T"], params:[t_var()], ret:i64_ty(), rest:None }`;
`runtime.rs:eval_length` (16155) arity-checks → evals arg → `match` raw `Value`
(Vec/HashMap/HashSet) → returns, else teaching `TypeMismatch`; dispatch arm at
`runtime.rs:5323`. Runtime per-type leaves already exist for all four ops (arc
146 + arc 240 List work).

## The typing tension (why this is NOT a uniform length-mirror)

`length` returns concrete `i64` — no element type in the return, so a plain
`∀T. T -> i64` scheme is exact. The four ops split:

- **TIER A — concrete return, plain ∀ scheme works:**
  - `empty?` : `∀T. T -> :bool` — pure length-mirror.
  - `contains?` : `∀T,E. (T, E) -> :bool` — two type-params, concrete bool out.
- **TIER B — return depends on the collection's element type (rank-1 ∀ can't
  express it precisely):**
  - `get` : `(coll, key) -> Option<element-of-coll>`. The old define-dispatch
    knew `Vector<T>/get -> Option<T>`, `HashMap<K,V>/get -> Option<V>` per-clause.
    A plain `∀T,K. (T,K) -> Option<?>` loses that. → likely a **custom inference
    arm** (mirror `infer_positional_accessor`, which already does `first` →
    `Option<element>` by reducing the collection type + extracting the element),
    NOT a plain scheme.
  - `conj` : `(coll, elem) -> coll` — type-preserving. `∀T. (T, E) -> T` may
    express it IF the checker unifies the return with arg0's type AND constrains
    `E` to the element type (so `conj(Vector<i64>, "x")` errors). Needs
    verification — may need a custom arm too.

**This is the load-bearing open question. The FM-2-bis probe settles it before
any BRIEF: does a plain ∀ scheme type-check `contains?`/`conj` correctly, and
does `get` need a custom inference arm?**

## List coverage (opportunity, not just migration)

`conj` is Vector/HashSet only (no List); `get`/`contains?`/`empty?` likewise
predate arc 220 List. The runtime leaves for List exist (arc 240 added
first/rest/List inference; eval already handles List in several arms). The
intrinsic eval arms SHOULD add the `wat__core__List` case where the runtime
supports it — closing the same gap 240.1 closed for first/rest. Verify per op in
the probe; do not assume.

## Slicing (proactive stepping-stones)

- **237.7b-i** — `empty?` (Tier A, the recipe-confirmer; pure length-mirror,
  `∀T. T -> bool`). Smallest, proves the bool-return intrinsic shape + the
  define-dispatch-delete cadence for this op family.
- **237.7b-ii** — `contains?` (Tier A) + `conj` (Tier B type-preserving), with
  List arms. Element-typed; rides 7b-i's settled recipe.
- **237.7b-iii** — `get` (Tier B, custom inference arm for element extraction).
  The hardest; isolated so its element-typing gets full attention.
- **237.7c (or 237.8-adjacent)** — `assoc` multi-impl (HashMap + Record) +
  `DispatchRegistry` deletion (after arithmetic 237.8 evacuates `+'2` etc.).

Rationale: 7b-i de-risks the cadence; Tier-A-before-Tier-B means the element-type
machinery lands on a proven foundation; `get`'s custom inference is the one piece
that genuinely diverges from the length recipe, so it gets its own stone.

## Immediate next action

**FM-2-bis probe** (`tests/probe_arc237_7b_intrinsic_typing.rs`): empirically
settle, for each op, whether a plain ∀ scheme suffices or a custom inference arm
is required — BEFORE briefing 7b-i. Specifically prove/disprove:
1. `∀T. T -> bool` type-checks `empty?` over Vector/HashMap/HashSet/List.
2. `∀T,E. (T,E) -> bool` type-checks `contains?` AND rejects a wrong-element call.
3. `conj` type-preservation: `(Vector<i64>, i64) -> Vector<i64>`; wrong elem errors.
4. `get` returns `Option<element>` precisely (or confirm it needs the custom arm).

Then 7b-i BRIEF cites the probe as the proven recipe.

## Constraints

Edits in `src/check.rs` + `src/runtime.rs` + `wat/core.wat` (delete decls) only.
NO holon-rs. NO `DispatchRegistry` deletion yet (237.7c). Per-op: register scheme
/ custom-arm → add eval arm (match raw Value, route to existing leaf) → delete the
`define-dispatch` decl → regression-guard probe green + lib 834/0 + build gate 0.
Green-gate = `./scripts/green-gate.sh` (`--lib` + `--tests` build; leak-free).

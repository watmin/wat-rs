# DESIGN — sequence-container registry: make checker↔runtime container drift UNREPRESENTABLE

## What + why (the root cure; rung 3 of the ladder)
The `seq-container-drift` stone (`75356ecc`) was the *symptom* cure: it hand-added the missing checker arms and
a probe that goes RED *if* drift recurs (rung 2 — a check that fires). This stone is the *root* cure (rung 3 — a
shape the mistake cannot be expressed in): the accepted-container set for a sequence op becomes a value
**derived from a single table that both the checker and the runtime consult**, so a one-sided arm — handling a
container on one side and not the other — has nowhere to be written.

Root mechanism of the bug class (confirmed across arcs 220/249/278-0b): "which containers does op X accept" is
duplicated — hand-rolled arms in `check.rs`/`collection/infer.rs` AND separate hand-rolled arms in
`runtime.rs`/`collection/eval.rs` — kept in sync by hand. Nothing forced the sync, so adding a runtime arm and
forgetting the checker twin produced a false-reject (and the lying error message → sonnet thrash). Eliminate the
duplication and the class is gone.

## Current state (grounded this session)
- **Checker, partially centralized.** `extract_seq_elem` (`collection/infer.rs:505`) maps a `TypeExpr` → its
  container + element type — but only `{Vector, PersistentVector}` (parametric and bare Path). The HOF ops
  (`map`/`filter`/`foldl`/`foldr`/`reverse`/`take`/`drop`/`concat`) route through it (infer.rs:570…1047) and so
  are drift-free. `first`/`second`/`third`/`rest` (in `check.rs`), `conj`/`get`/`contains`/`assoc`
  (infer.rs:31/129/231/343) hand-roll their own arms.
- **Runtime, not centralized.** Each op hand-rolls `Value::wat__core__X(_) => …` arms (length:12280,
  empty?:12323, contains?:12368, conj:12408, get:12452, positional:10944, rest in collection/eval). The only
  Value→container map is `val_type_path` (runtime.rs:6420), used for *naming*, not op dispatch.

## The registry (the single source of truth)
A new home `src/collection/seq_container.rs` owns:

1. **`enum SeqContainer`** — the closed set of positional/linear containers:
   `Vector, List, PersistentVector, Tuple, WatAstList, HashSet`. (Keyed collections — HashMap, PersistentMap,
   Record — are a *separate* family for `get`/`assoc`/`contains`-on-maps; see Out of scope.)

2. **Two total classifiers (the ONLY container-recognition sites):**
   - `fn of_type(&TypeExpr) -> Option<SeqContainer>` (checker side) — supersedes/absorbs `extract_seq_elem`.
   - `fn of_value(&Value) -> Option<SeqContainer>` (runtime side).
   Both `match` exhaustively on the representation; adding a `Value` variant or `TypeExpr` head a container can
   take forces a compile error here until handled.

3. **A capability table — THREE states per cell** (the `✗`-conflation was a flaw; a cell is one of):
   - **✓ Supported** — built, both sides.
   - **○ Gap** — the container's nature PERMITS it, the runtime arm was never built. *Incompleteness, fillable* —
     after the registry, flipping `○→✓` is one capability flag + one element-iterate impl, both sides, drift-free.
   - **∅ N/A** — the container's NATURE forbids it (not a gap; never to be filled).

   GROUNDED in the runtime's real arms (the strike verifies each against `runtime.rs`/`collection/eval.rs`):

   | container | Indexable | Tail (rest) | Append (conj) | Mappable |
   |---|---|---|---|---|
   | Vector / PersistentVector | ✓ | ✓ | ✓ | ✓ |
   | List | ✓ | ✓ | ✓ | ○ gap (runtime maps only Vec/PV) |
   | Tuple | ✓ | ∅ N/A (heterogeneous: arity-type change) | ∅ N/A | ∅ N/A (one `f` can't map mixed types) |
   | WatAstList | ✓ | ✓ | ○ gap | ○ gap |
   | HashSet | ∅ N/A (unordered: no "first") | ∅ N/A | ✓ | ○ gap (set→set sensible) |

   THIS stone is behavior-preserving: it encodes the table AS-IS (drift dead). The `○ gap` cells are the
   self-documenting worklist for the *incompleteness* annihilation (the immediate follow-on the registry makes
   trivial — e.g. "List mappable" becomes a one-liner). The `∅ N/A` cells stay forever.

4. **Per-capability element + reconstruction helpers**, both representations:
   - `elem_type(&self, &TypeExpr) -> TypeExpr` / `elem_values(&self, &Value) -> Vec<Value>` (extraction).
   - `reconstruct_type(&self, elem: TypeExpr) -> TypeExpr` / `reconstruct_value(&self, items) -> Value`
     (identity-preserving rebuild — for `rest`/`conj`/`map` returning the same container kind).

## The parity guarantee (why drift becomes unrepresentable)
An op no longer hand-lists containers. It declares the **capability** it needs, and the accepted set is
*computed from the one table*:
- `first`/`second`/`third` require `Indexable` → accepted set = `{c | c.indexable()}`, identical on both sides
  because both read the same table.
- `rest` requires `Tail`; `conj` requires `Append`; etc.

There is no per-op, per-side container list to drift, because the set is **derived, not written twice**. And a
new container = one new `enum` variant → exhaustiveness errors in `of_type`, `of_value`, and the capability
table until all are filled → it is impossible to teach the runtime a container without the checker (and vice
versa). That is the rung-3 guarantee.

## The narrow waist — why this is about EVOLUTION, not just drift
PV and PersistentMap were never planned; they were **shimmed in at O(ops)** — one hand-written arm bolted onto
every collection op (~16), on BOTH sides, by hand. That O(ops)-per-new-type cost IS the process that drifted and
bit us (someone bolted the runtime arm, forgot the checker twin). The registry's real purpose is to make adding
the *next* primitive type **O(1)**: the registry is the **narrow waist** of an hourglass —

```
  first second third rest conj nth get map filter fold reverse take drop ...   ← OPS (many; written ONCE)
                \            \         |        /          /
                 ╲─────────── Container protocol (the waist) ──────────╱        ← ONE thin interface
                /          /        |         \            \
          Vector   List   PersistentVector   Tuple   WatAstList   HashSet   <FutureType>   ← TYPES (each impl'd ONCE)
```

Ops are generic over the waist; a new primitive type implements the waist **once** and every op lights up — both
checker and runtime, drift-free by construction. (This is the same hourglass as wat's `defprotocol` Seq/Map —
arc 285, STUBBED; this Rust-internal waist is the impl-level substrate that wat-level protocol can later ground
on. We own the source, so the waist is a closed enum edited in one home — no open-plugin machinery needed; "add a
primitive type" = "edit one file".)

## The contract decision (pinned)
> **Ops never `match` on which container it is.** Each op is written generically against the container protocol —
> `capabilities()`, `elements(value)`, `reconstruct(items)`, `elem_type(typeexpr)`, `reconstruct_type(elem)` —
> and asks only "do you have capability C?". The accepted-container set is therefore DERIVED from the one
> capability table, identical on both sides (checker `of_type`, runtime `of_value`). A new container = one impl
> at the waist (one home file); the op layer never moves. Matching on a specific container *inside an op* is the
> anti-pattern this stone exists to delete — it reintroduces O(ops) coupling and the drift class.

## Scope — ANNIHILATE the whole positional/sequence family (one stone, no staging)
A staged "migrate the drift-5 now, the rest later" leaves the class ALIVE in the unmigrated ops — that is
containment, not annihilation. This stone migrates **every** positional/sequence op onto the registry on BOTH
sides, deleting all their hand-rolled container arms, until **no hand-rolled container classification remains in
the sequence family anywhere**:
`first`/`second`/`third`/`rest`/`nth`/`last`/`get`(seq)/`conj`/`contains`(seq)/`length`/`empty?` + the HOF
family `map`/`filter`/`foldl`/`foldr`/`reverse`/`take`/`drop`/`concat`. `extract_seq_elem` is absorbed into
`of_type` (the HOFs stop carrying a hard-coded `{Vector,PV}`).

Execution may proceed op-by-op *within* the stone (migrate one, keep the suite green, next) — that is incremental
execution toward a complete kill, NOT staging that ships a half-dead class. The DELIVERABLE is the whole family
annihilated.

**Honest decomposition (not timidity):** keyed-collection ops (`assoc`, map-`get`, map-`contains`) are a
genuinely DIFFERENT shape (K→V, not indexed-element) and get their own `MapContainer` registry — a sibling
stone, same pattern. Splitting positional-vs-keyed is decomposition by real type-family difference, not a staged
retreat.

## Two layers of annihilation — what each one kills (be honest about the rung)
- **Layer 1 — the capability registry (THIS stone).** Annihilates the class we actually suffered:
  *container-added-to-one-side-only* becomes a **compile error** (a new `enum` variant → exhaustiveness failures
  in `of_type`, `of_value`, and the table until all are filled), and *per-op set divergence* becomes impossible
  (the set is DERIVED from one capability table both sides read, never written twice). The bug class = dead.
- **Layer 2 — generated arms (NOT this stone; named, decide after Layer 1).** A macro that emits BOTH the checker
  arm and the runtime arm from one capability declaration would additionally annihilate the only residual Layer 1
  leaves: *a future dev writing a brand-new op with bespoke arms that ignore the registry* — a failure we have
  NOT suffered. It is reachable but is metaprogramming across two megafiles; building it now is gold-plating a
  hypothetical. Recommend: land Layer 1, then decide Layer 2 against real need. (extirpare: climb until the
  material runs out OR the next rung guards only a hypothetical — hold the rung that kills the real class.)

## The probe (parity matrix, the green net)
`tests/probe_seq_container_registry.rs`: for each migrated op × each `SeqContainer`, assert BOTH (a) the
runtime result and (b) that the checker accepts/rejects in lockstep with the capability table — i.e. an op
type-checks a container IFF the table grants the capability, AND runs to the same value. Plus a NEGATIVE case
(a non-container arg rejected on both sides). The existing `probe_seq_container_parity.rs` (7) and the full
collection op test suite are the regression net (behavior-preserving: this is a refactor, observable results
unchanged).

## Out of scope (= rejected for this stone, named)
- **Keyed-collection registry** (`assoc`/map-`get`/map-`contains`) — sibling stone (`MapContainer`), distinct
  type-family.
- **Layer 2 codegen** — named above; decided after Layer 1 against real need, not pre-built.
- **Filling matrix gaps is NOT in this stone.** The capability table encodes CURRENT runtime truth (e.g. List is
  `✗ Mappable` because the runtime maps only Vec/PV today). Making List mappable etc. is FEATURE work, a separate
  decision — sneaking it into a refactor would be dishonest and inflate differential risk. The registry makes
  each such gap a one-line future annihilation; that's its dividend, not this stone's job.
- Hand-decomposing the megafiles beyond necessary call-site rewrites (corrected doctrine: registry is new code in
  a HOME; `check.rs`/`runtime.rs` call sites become thin classify→dispatch).

## Done = green (whole-family annihilation)
`tests/probe_seq_container_registry.rs` green (the full op × container parity matrix + a non-container negative);
`probe_seq_container_parity` still 7/7; the full collection op suite unchanged (behavior-preserving refactor);
rete differentials (8a/8b/8custom/7exists) unchanged; lib floor 941/36; `cargo build --release` clean; warning
count ≤ 26. **The kill is proven by grep:** no hand-rolled per-container `match` arm remains in ANY sequence op
on either side — every one classifies through `SeqContainer::of_type`/`of_value`. A new container is a single
enum variant whose exhaustiveness ripple forces both sides (demonstrate in the SCORE: add a dummy variant, show
it fails to compile until both classifiers + the table handle it, then remove it).

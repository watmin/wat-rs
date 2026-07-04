# 118.2-Z — the comprehensive clojure-core seq/collection/aggregate family completion

> **Status: STRIKE — DESIGN. Scope ratified 2026-07-03.** After the 118.2a flip landed (`map`/`filter`/`take`/
> `drop` lazy over `Stream`; `mapv`/`filterv`/`into`/`reduce`/`count`/`run!`), the builder ruled the close is not a
> thin tail but a **massive core upgrade to comprehensive clojure-core parity**: *"all the things we expect to find in
> clojure should be here with very few limitations."* This design is the map all six strikes brief from.

## The bar (ratified)

**Comprehensive clojure-core parity for the seq / collection / aggregate family — very few limitations.** A clojure
dev reaches for a form and it is there, in `:wat::core::`, named as they expect (`NOMINA NOTA, MACHINA TACITA`).
Divergence only where the *dialect* demands it (`VIRTVTE PARES, NON LITTERA` — dialect, not impl), and every such
divergence is named, not silent.

### Out of scope — named, not deferred-in-costume (each derived, not chosen)

- **`reduced` / `reduced?`** — OBVIATED, not deferred (`118 TVA RECVRSIO, TVVS REDITVS`). `loop`/`recur`/`reduced`
  are one JVM-workaround cluster wat sheds: TCO-on-Rust means you invoke yourself directly (no `recur`) and return to
  stop (no `reduced`). There is no `reduced` stone. The one true dependency under deep folds is **arc 261**
  (stack-safe eval / CEK — a STUB), a separate arc.
- **Memoization / rewind** — the single-pass law (`118 R1, NON BIS IN IDEM FLVMEN`). A rewind buffer is the user's to
  build; core ships the honest O(1)-memory primitive.
- **`seq` / `next` (clojure's nil-punning)** — clojure's `seq` returns `nil` on empty and `next` returns `nil` at the
  end, and idiom leans on that nil-pun (`(when (seq xs) …)`). wat **sheds nil-punning**: `nil` = unit (no-useful-
  return), absence = `#wat.core.Option/None` (explicit). The wat idiom is `empty?` + `rest` (both already
  polymorphic), never a nil-returning `seq`/`next`. So `seq`/`next` are OUT as nil-punning forms — their *capability*
  is present under `empty?`/`rest`/`first`. (If a `first`-or-None accessor is wanted, that is an `Option`-returning
  form, a separate small decision — not clojure's nil-punning `next`.)
- **`lazy-seq` the macro** — wat has `:wat::stream::lazy` (the primitive) + the lazy defclause family; `lazy-seq` the
  clojure macro-name is not resurfaced (`118 R1` settled the `stream/lazy` naming).
- **transducers** — clojure's `(map inc)`-arity-1 transducer protocol is a separate paradigm; OUT of this arc (the
  eager+lazy surfaces cover the expressiveness; a transducer arc is a future, named elsewhere if demand surfaces).

## The roster — grounded HAVE vs NEED (2026-07-03)

**HAVE** (checker intrinsics + `wat/` defclauses/defns, grounded): `map` `filter` `take` `drop` `reduce` `foldl`
`foldr` `mapv` `filterv` `into` `vec`(→`into []`) `doall` `dorun` `run!` `count` `first` `rest` `second` `third`
`last` `nth` `concat` `conj` `reverse` `sort` `sort-by` `keys` `values` `assoc` `dissoc` `get` `contains?` `apply`
`range` `empty?` · primitives `:wat::stream::{cons,lazy,empty}`.

**NEED** — by SHAPE (the shape is the strike boundary):

| shape | forms to build |
|---|---|
| **① lazy transformers** (1-seq, `filter` defclause shape → `Stream`) | `remove` · `keep` · `keep-indexed` · `map-indexed` · `mapcat` · `take-while` · `drop-while` · `take-nth` · `distinct` · `dedupe` · `interpose` · `reductions` |
| **② partition / split family** (chunking; over `take`/`drop`) | `partition` · `partition-all` · `partition-by` · `split-at` · `split-with` |
| **③ multi-seq** (needs variadic `map`) | **`map` → variadic** (N-seq, stop-at-shortest, clojure-faithful) · `interleave` · `interleave`-kin. **`zip` REMOVED** (→ `(map vector …)`) |
| **④ eager terminals** (seq → value / map / bool) | `min` · `max` · `min-key` · `max-key` · `group-by` · `frequencies` · `some` · `every?` · `not-any?` · `not-every?` |
| **⑤ generators** (lazy producers → `Stream`, ∞-capable) | `iterate` · `repeat` · `repeatedly` · `cycle` |

Deliberately weighed and **included** despite subtlety: `flatten` — clojure ships it (deep-flatten); included in ①
as a lazy transformer, documented as the deep variant. `dedupe` (consecutive-only, no seen-set) **and** `distinct`
(all, seen-set) both ship — they are different clojure-core forms.

## The two layers (dependency-ordered — Layer 1 before Layer 2, always)

**Layer 1 — complete the core family.** Everything above. Nothing composes what does not yet exist, so Layer 1 lands
first (you cannot `(reduce min …)` before `min` exists — grounded: `min`/`max`/`group-by` were all ABSENT, which is
exactly why rete hand-rolled them).

**Layer 2 — consumers stop reimplementing; compose Layer 1.** Two clean-ups, both derived:

- **`:wat::std::list::` orphan re-home (FORCED — arc 109 std-kill).** Four eager intrinsics hang off the DEAD `std`
  namespace (grounded: `check.rs:19473/19487/19496`). All live/used:
  - `map-with-index` (8 sites) → **`:wat::core::map-indexed`** (upgraded into ① lazy; it was the eager ancestor).
  - `remove-at` (29 sites) → **`:wat::core::remove-at`** (kept — a genuine positional vector op clojure lacks; a
    superset like enums/match/Result; re-homed, stays eager).
  - `window` (9 sites) → **subsumed into `:wat::core::partition`** (`(partition n 1 coll)` is a sliding window; not a
    clojure name — don't ship the wat-ism `partition` expresses).
  - `zip` (21 sites) → **removed** → `(map vector …)` (rests on ③ variadic `map`).
- **rete accumulator decouple (RULED IN — "any tooling that isn't rete-specific gets moved out; rete uses the core
  tooling with rete logic on top").** rete's 8 `acc::*` (`rete.wat:2011+`) **fuse** a rete-domain *projection*
  (`Element` → `bindings[var]`) with a pure *aggregate*. The aggregate moves to Layer 1; the projection stays; the
  accumulator becomes a thin wrapper (the core op is the oracle — `replicate-is-a-smell`):
  - `acc::distinct` → `(into [] (:wat::core::distinct (:wat::core::map <project> els)))`
  - `acc::min`/`max` → `(:wat::core::reduce :wat::core::min (:wat::core::map <project> els))` (needs Layer-1 `min`/`max`)
  - `acc::sum` → `(:wat::core::reduce :wat::core::+ 0 (:wat::core::map <project> els))` (no new form — `+` exists)
  - `acc::mean` → `(:wat::core::/ sum count)` (composition, no new form)
  - `acc::group-by` → `(:wat::core::group-by <project> els)`
  - `acc::count` → `(:wat::core::count els)` (already trivial)

## The strike decomposition (6 strikes + close — depth-first; the gate is the judge)

1. **A — lazy transformers ①.** ~12 defclauses in `wat/seq.wat`, the `filter` shape (one clause per seqable,
   `stream/lazy` + recursion; a `<form>-stream` helper `defn` where the Stream-walk needs it, à la `reduce-stream`).
   RED probe: laziness of `take-while`. `secare`-clean (one file, one pattern).
2. **B — partition / split family ②.** Over `take`/`drop` (already lazy). `partition`'s `step` arity is forced into
   scope here (it subsumes `window`). RED probe: `(partition 2 (range))` terminates lazily.
3. **C — variadic `map` + `interleave` ③.** The one strike touching **Rust** (`eval_vec_map` + `infer_map`,
   `transform.rs`/`infer.rs`) — N-seq lockstep, stop-at-shortest, variadic type inference. `interleave` a defclause
   over it. `zip` retired (21-site migration → `(map vector …)`). RED probe: `(map + [1 2] [10 20])` → `(11 22)`.
4. **D — eager terminals ④.** `min`/`max`/`min-key`/`max-key` (variadic arithmetic), `group-by`/`frequencies` (seq →
   map), `some`/`every?`/`not-any?`/`not-every?` (seq → bool). RED probe: `(min 3 1 2)` → `1`; `(group-by odd? …)`.
5. **F — generators ⑤.** `iterate`/`repeat`/`repeatedly`/`cycle` — lazy producers (`stream/lazy` + `stream/cons`, no
   input seq; ∞-by-construction). Leans on C's stop-at-shortest to terminate `(map f finite (iterate …))`. RED probe:
   `(take 3 (iterate inc 0))` → `[0 1 2]` without diverging.
6. **G — Layer 2 consumer migration.** The `:wat::std::list::` re-home (67 sites) + the rete accumulator decouple (8
   `acc::*`). Rides the red of the retired orphan intrinsics. `secare`-splittable by crate.

**Close (Z).** `intueri` the whole family (coherence); fix the stale `src/collection/mod.rs:124` comment ("DO NOT
defclause collection ops" — the flip already broke it with `filter`); turn `118 R1` (`NON BIS IN IDEM FLVMEN`) and
`R2` (`STRICTVM ARDET FLVMEN SVRGIT`) from PROBANDVM → PROBATVM (the flip + family landed); whole-workspace gate-0;
**INSCRIPTION** (no deferrals — `reduced`/memoization/`seq`-nil-pun affirmatively out-of-scope, arc 261 named).

## Ordering (the dependency IS the order)

A, B, D, F build Layer-1 forms with **no interdependence** → can pipeline/parallelize. **C (variadic map) gates
`interleave` and `zip`-removal**; **F (generators) wants C's stop-at-shortest** to be truly useful (finite∩infinite).
**G (Layer 2) needs all of Layer 1** (rete composes `min`/`max`/`group-by`/`distinct`; the orphan re-home needs
`map-indexed`/`partition`). So: **A‖B‖D first, C, then F, then G, then Close.** Each strike: DESIGN-mirrors-this +
RED probe committed first + BRIEF + EXPECTATIONS + orchestrator-weighed kill.

## Out (rejected — named): `reduced` · memoization/rewind · `seq`/`next` nil-punning · `lazy-seq` macro-name · transducers. Each derived above, not deferred.

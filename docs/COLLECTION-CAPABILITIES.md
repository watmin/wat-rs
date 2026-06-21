# Collection capabilities — the authoritative grid

**This is the source of truth for which container supports which operation, and the state of each cell.**
Arc 278's narrow-waist registries (`SeqContainer`, `MapContainer`) exist to make this grid *enforced by the
compiler*: a cell is `done`, `BUILD` (a gap we fill now), or `N/A` (the container's nature forbids it, grounded).
A few are `DECIDE` — a genuine ✓-vs-N/A call awaiting a four-questions ruling. **No cell ships `○` (undecided).**

Operating principle (builder, 2026-06-20): *"the substrate must force our hands relentlessly towards the
idealized state."* The exhaustive enums + capability tables turn every gap into a compile error or a dead_code
signal — you cannot add a container, or leave a cell unbuilt, without the build telling you. This doc is the
human-readable face of that machine.

## Legend
- **`done`** — delivered (routed through the registry, runtime + checker, with tests).
- **`BUILD`** — a real gap; we implement it now, before moving on. (Was `○ gap` in the registry tables.)
- **`N/A`** — the container's *nature* forbids it (grounded reason given). Never to be filled.
- **`DECIDE`** — a genuine ✓-vs-`N/A` design call; needs a four-questions ruling before it becomes `done`/`BUILD`/`N/A`.

## The containers (what exists today)
- **Seq family** (`SeqContainer`): Vector, PersistentVector, List, Tuple, WatAstList, HashSet
- **Map family** (`MapContainer`): HashMap, PersistentMap, Record (base `wat__Record` + holon `wat__holon__Record`)

> **Not in either grid: lazy/infinite sequences (`lazy-seq`).** That *type* does not exist yet and is the one
> thing explicitly deferred (builder's call). It is **not a gap in this grid** — it's a future primitive that,
> when built, becomes a new `SeqContainer` variant the exhaustive matches will *force* into every cell below.

---

## Grid 1 — Seq family

Op groups: **pos** = first/second/third · **rest** · **conj** · **map** = map/filter/foldl/foldr · **ord** =
reverse/take/drop (order-dependent) · **concat** · **get** · **has?** = contains? · **len** = length/empty?

| container        | pos  | rest | conj  | map   | ord   | concat | get        | has?       | len  | assoc (by-index)      |
|------------------|------|------|-------|-------|-------|--------|------------|------------|------|-----------------------|
| Vector           | done | done | done  | done  | done  | done   | done       | done       | done | DECIDE (Clojure ✓)    |
| PersistentVector | done | done | done  | done  | done  | done   | done       | done       | done | DECIDE (Clojure ✓)    |
| List             | done | done | done  | BUILD | BUILD | BUILD  | BUILD¹     | BUILD¹     | done | N/A (not associative) |
| Tuple            | done | N/A² | N/A³  | N/A⁴  | N/A⁴  | N/A³   | DECIDE     | DECIDE     | BUILD| DECIDE                |
| WatAstList       | done | done | BUILD | BUILD | BUILD | BUILD  | DECIDE     | DECIDE     | BUILD| DECIDE                |
| HashSet          | N/A⁵ | N/A⁵ | done  | BUILD⁶| N/A⁷  | DECIDE⁸| DECIDE⁹    | done       | done | N/A (not k→v; use conj)|

1. **List get/has?** — the `list_get_inner` / `list_contains_q_inner` helpers **already exist** but aren't wired
   into the polymorphic dispatch. `BUILD` = wire them through the registry (cheap).
2. Tuple/rest — tail changes arity *and* element types → not representable.
3. Tuple/conj, Tuple/concat — fixed arity; appending changes the type.
4. Tuple/map, Tuple/ord — heterogeneous elements; one `fn` can't map mixed types.
5. HashSet/pos, HashSet/rest — unordered; no canonical "first" or "tail".
6. **HashSet/map** = ✓ set→set (deduped, unordered) — DECIDED this session (best option; identity-preserving).
   `foldl`/`foldr` over a set fold in **unspecified order** (set is unordered) — flag at build.
7. HashSet/ord (reverse/take/drop) — order-dependent ops on an unordered container → N/A.
8. **HashSet/concat** — sequence-concat on a set = set *union*, a different op. DECIDE: `N/A` ("use a union op")
   or ✓-as-union.
9. **HashSet/get** — Clojure `(get #{x} x)` returns the element if present (membership-as-lookup). DECIDE:
   ✓ (Clojure-faithful membership-get) or `N/A` (use `contains?`).

---

## Grid 2 — Map family

Seq ops (pos/rest/conj/map/ord/concat) on maps: a map is **not a sequence** in wat's model (unordered, no
`lazy-seq`). Clojure makes maps seqable (→ map-entries); wat does not, today.

| container     | get        | has? (key) | len  | empty? | assoc | seq ops (first/rest/map/…) |
|---------------|------------|------------|------|--------|-------|----------------------------|
| HashMap       | done       | done       | done | done   | done  | N/A (not a sequence)¹⁰      |
| PersistentMap | done       | done       | done | done   | done  | N/A¹⁰                       |
| Record        | BUILD¹¹    | BUILD¹¹    | BUILD| BUILD  | done* | N/A¹⁰                       |

\* Record/assoc = field update (flavor-preserving) — `done` once strike 5 lands (the pub-leak fix is in the weigh).

10. **map/fold over map-entries** — Clojure allows `(map f a-map)` over `[k v]` entries (unordered). DECIDE
    (whole row): do maps become iterable over entries? Tied to the `lazy-seq` question — likely **deferred with
    lazy-seq**, not built now. Marked `N/A (today)`, revisited when seq/iteration lands.
11. **Record get/has?/len** — get-by-keyword, contains-field, field-count/empty. `BUILD` (Clojure records are
    associative + counted). Record's `keyed_lookup`/`has_key`/`measurable` capability cells flip `false → true`
    when built (strike 6).

---

## The BUILD queue (gaps we fill now, no deferral)

In dependency order; each lands green before the next, continuous (no parking):

1. **Strike 6 — route the mixed ops both waists + fill Record cells.** Route `get`/`contains?`/`length`/`empty?`
   through `SeqContainer` (seq arms) and `MapContainer` (map arms); fill **Record/get, Record/has?, Record/len**;
   wire **List/get, List/has?** (helpers exist). Consumes `MapContainer::keyed_lookup`/`has_key`/`measurable`
   (kills the dead_code by *use*, not `#[allow]`). Fills Tuple/len, WatAstList/len.
2. **Strike 7 — seq HOF fills.** **List/map, WatAstList/map+conj, HashSet/map (set→set)**, and List/WatAstList
   for reverse/take/drop/concat. Each new `*_inner` + capability cell `false→true`.
3. **The DECIDE cells** (below) — ruled, then built or cut, interleaved as each op's strike reaches them.

## The DECIDE list (four-questions rulings owed — your call)

| cell | the call | lean |
|------|----------|------|
| Vector/assoc, PV/assoc | assoc-by-index (`(assoc v 0 :x)`) | Clojure ✓ — build |
| Tuple/get, Tuple/has?, Tuple/assoc | tuple by-index get / element-membership / by-index update | get ✓, has? ✓, assoc ✓ (tuples are indexed, fixed-arity) — lean build, grounded per cell |
| WatAstList/get, has?, assoc | child-by-index get / contains-child / child update | ✓ for AST manipulation — lean build |
| HashSet/get | membership-as-lookup (Clojure `(get set x)`) | ✓ Clojure-faithful, OR N/A "use contains?" |
| HashSet/concat | set union vs N/A | lean N/A (union is a distinct op) |
| Map family / seq ops (map-entries) | iterate maps as `[k v]` | **defer with lazy-seq** (not now) |

## The N/A registry (grounded cuts — never filled)
- Tuple: rest, conj, map, ord, concat — heterogeneous + fixed arity.
- HashSet: pos, rest, ord — unordered.
- List: assoc — not associative (sequential).
- HashSet: assoc — not key→value (use conj).
- Map family: positional seq ops — not a sequence (today).

---

## How this doc stays true
Update a cell the moment its strike lands (`BUILD → done`) or its ruling is made (`DECIDE → BUILD/N/A`). The
registries' capability tables (`seq_container.rs`, `map_container.rs`) are the *machine* version of this grid;
this doc is the *map*. When they disagree, the code wins and this doc is stale — fix it. A new container primitive
adds a row here AND a variant there; the exhaustive matches make sure neither is forgotten.

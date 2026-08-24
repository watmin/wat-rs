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
- **`DECIDE`** — ~~a genuine ✓-vs-`N/A` design call~~ **RESOLVED 2026-06-20** (see **Rulings** below; bucket now empty). Grid cells still marked `DECIDE` are superseded by the Rulings table and flip to `done`/`N/A` as each strike lands.

## The containers (what exists today)
- **Seq family** (`SeqContainer`): Vector, PersistentVector, List, Tuple, WatAstList, HashSet
- **Map family** (`MapContainer`): HashMap, PersistentMap, Record (base `wat__Record` + holon `wat__holon__Record`)

> **Not in either grid: lazy/infinite sequences (`lazy-seq`).** That *type* does not exist yet and is the one
> thing explicitly deferred (builder's call). It is **not a gap in this grid** — it's a future primitive that,
> when built, becomes a new `SeqContainer` variant the exhaustive matches will *force* into every cell below.

---

## Grid 1 — Seq family   *(all cells resolved; `BUILD` = the active campaign)*

Op groups: **pos** = first/second/third · **rest** · **conj** · **map** = map/filter/foldl/foldr · **ord** =
reverse/take/drop (order-dependent) · **concat** · **get** · **has?** = contains? · **len** = length/empty? ·
**assoc** = assoc-by-index

*(lookup/size — get/has?/len — DONE for all containers, both families, as of HEAD `7550310f`. Remaining `BUILD`:
the HOF cells, index-`assoc`, and set algebra.)*

| container        | pos  | rest | conj  | map   | ord   | concat | get   | has?  | len   | assoc |
|------------------|------|------|-------|-------|-------|--------|-------|-------|-------|-------|
| Vector           | done | done | done  | done  | done  | done   | done  | done  | done  | BUILD |
| PersistentVector | done | done | done  | done  | done  | done   | done  | done  | done  | BUILD |
| List             | done | done | done  | done  | done  | done   | done  | done  | done  | N/A ᵃ |
| Tuple            | done | N/A ᵇ| N/A ᶜ | N/A ᵈ | N/A ᵈ | N/A ᶜ  | N/A ᵉ | done  | done  | N/A ᵉ |
| WatAstList       | done | done | BUILD | BUILD | BUILD | BUILD  | done  | done  | done  | BUILD |
| HashSet          | N/A ᶠ| N/A ᶠ| done  | BUILD²| N/A ᵍ | N/A ʰ  | done³ | done  | done  | N/A ⁱ |

² HashSet/map = set→set (deduped, unordered); `foldl/foldr` fold in unspecified order (flag at build).
³ HashSet/get = membership-as-lookup (`Some(x)` if present) — uniform with `get` across keyed containers; under
  value-semantics returns no *new* info vs `has?`, kept for uniformity (Clojure's canonicalization needs ref identity).

N/A reasons: ᵃ List not associative (sequential) · ᵇ tail changes arity+types · ᶜ fixed arity · ᵈ heterogeneous
(one `fn` can't map mixed types) · ᵉ heterogeneous product: runtime-index can't be typed; use static
first/second/third or destructure · ᶠ unordered (no first/tail) · ᵍ order-dependent on unordered · ʰ `concat` is
seq-join; set-combine is `union` (see set algebra) · ⁱ not key→value (use conj).

---

## Grid 2 — Map family   *(all cells resolved)*

| container     | get   | has? (key) | len/empty? | assoc  | map/filter/fold (→Vec) | first/rest/positional |
|---------------|-------|------------|------------|--------|------------------------|-----------------------|
| HashMap       | done  | done       | done       | done   | BUILD ⁴                | N/A ⁵                 |
| PersistentMap | done  | done       | done       | done   | BUILD ⁴                | N/A ⁵                 |
| Record        | done  | done       | done       | done   | BUILD ⁴                | N/A ⁵                 |

Record/assoc = field update (flavor-preserving) — done (A2 `361788a1`).
⁴ map/filter/fold over a *finite* map iterate `[k v]` entries → **Vec** (eager; no lazy-seq needed). Return swaps
  Vec→lazy-seq when lazy-seqs land. (Still `BUILD` — part of the HOF/map-iteration work.)
⁵ first/rest/positional — maps are unordered; no positions. (Clojure's seqable-as-entries waits on lazy-seq.)

## Set algebra — NEW verbs (BUILD)

Sets need their combine ops; `concat` is not it. All `BUILD`:

| op             | HashSet | grounding |
|----------------|---------|-----------|
| `union`        | BUILD   | Ruby `Set#merge` / Clojure `clojure.set/union` — unordered, dedupes |
| `intersection` | BUILD   | elements in both |
| `difference`   | BUILD   | elements in a not in b |

---

## STANDING ORDER (2026-06-20)
**Collections are the blocking campaign.** Rete feature work (custom accumulators, returns-the-fact, …) is
BLOCKED until this grid is all `done`/`N/A`. We're in flow; the collection surface gets sane first, then we loop
back to the rete items. DECIDE bucket is **empty** — all rulings made below.

## The BUILD queue (in dependency order; each green before the next, continuous — no parking)

- ✅ **DONE — lookup/size, both families** (commits `f4beda7d`→`7550310f`): `assoc` + get/contains?/length/empty?
  route through MapContainer (HashMap/PersistentMap/Record) AND SeqContainer (all six seq types) via genuine
  capability gates; Record get/has?/len via schema; List/get+has? wired; Tuple/len+has?; WatAstList/len+get+has?;
  HashSet/get (membership). Floor lib 953/36/1, warnings 26.

REMAINING:
1. **Seq HOF fills (NEXT)** — flip `mappable` for List/WatAstList/HashSet; build/route map+filter+foldl+foldr for
   List, WatAstList, HashSet(set→set, unspecified fold order); reverse/take/drop/concat for List + WatAstList;
   **WatAstList/conj**. (Tuple HOFs = ∅N/A.)
2. **map/filter/fold over maps → Vec** — eager `[k v]` entry-iteration (HashMap/PersistentMap/Record); no lazy-seq.
3. **Index-assoc** — `assoc`-by-index on Vector/PV/WatAstList (homogeneous, bounds-checked). (Tuple/List = N/A;
   WatAstList/get-by-index already DONE in seq-1b.)
4. **Set algebra (new verbs)** — `union`, `intersection`, `difference` on HashSet (Ruby `Set#merge` / Clojure
   `clojure.set/*`). A set without its algebra is itself a gap.

## Rulings (DECIDE resolved 2026-06-20, four-questioned against the ADT + wat's choices, not stale rosetta)

| cell | verdict | grounding |
|------|---------|-----------|
| Vector/assoc, PV/assoc, WatAstList/assoc | **✓ BUILD** (assoc-by-index) | homogeneous → type-preserving, bounds-checked; the immutable element-update verb |
| WatAstList/get | **✓ BUILD** (child by index → `(Option :- [WatAST])`) | homogeneous, precise |
| Tuple/get, Tuple/assoc | **N/A** | heterogeneous *product*; runtime-index can't be typed (→`(Option :- [Value])`, lossy) and precise static access exists (first/second/third, destructure) |
| Tuple/has?, WatAstList/has? | **✓ BUILD** (element membership) | membership needs no static typing; matches wat's element-`contains?` |
| HashSet/get | **✓ BUILD** (membership-as-lookup) | uniform `ILookup` — a set is an element→element map; `get` works on every keyed container. (Footnote: under value-semantics it returns no *new* info vs `contains?` — Clojure's canonicalization payoff needs reference identity, which we lack. Kept for uniformity, not canonicalization.) |
| HashSet/concat | **N/A** | `concat` = ordered seq-join (keeps dupes); set-combine is `union` (unordered, dedupes) — a distinct verb. Clojure keeps them separate too. |
| Map/set-combine | **✓ BUILD `union`/`intersection`/`difference`** | sets need their algebra; `concat` is not it |
| Map family / map+filter+fold (entries) | **✓ BUILD → Vec** | eager iteration of a *finite* map; no lazy-seq needed; return swaps Vec→seq when lazy-seqs land |
| Map family / first/rest/positional | **N/A** | unordered; no positions (Clojure makes maps seqable via lazy-seq — deferred) |
| Seq *interface* (abstraction) | **with lazy-seqs, not now** | minting an interface over one eager impl is premature abstraction; introduce when lazy-seqs are its 2nd implementor |

## The N/A registry (grounded cuts — never filled)
- Tuple: rest, conj, map, ord, concat, **get, assoc** — heterogeneous product / fixed arity; static positional access only.
- HashSet: pos, rest, ord — unordered. **concat** — use `union`.
- List: assoc — not associative (sequential).
- HashSet: assoc — not key→value (use conj).
- Map family: positional/first/rest seq ops — not a sequence (today; revisited with lazy-seqs).

## The one bounded future-type
**lazy/infinite `lazy-seq`** — does not exist; the single deferred thing (builder's call). NOT a gap in this grid —
a future primitive the exhaustive matches will *force* into every cell when it lands. It also flips the
map+filter+fold *return* (Vec→lazy-seq) uniformly, and is when the `Seq` interface gets minted.

---

## How this doc stays true
Update a cell the moment its strike lands (`BUILD → done`) or its ruling is made (`DECIDE → BUILD/N/A`). The
registries' capability tables (`seq_container.rs`, `map_container.rs`) are the *machine* version of this grid;
this doc is the *map*. When they disagree, the code wins and this doc is stale — fix it. A new container primitive
adds a row here AND a variant there; the exhaustive matches make sure neither is forgotten.

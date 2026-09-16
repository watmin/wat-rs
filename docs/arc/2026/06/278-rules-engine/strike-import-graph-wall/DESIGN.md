# DESIGN-STONE — the fourth wall: a graph the engine can legally walk

> **Origin (2026-08-30).** Vigilia Class A1. `circumspicere`, cast last against the surround the
> eighteen inward wards turned their backs on: *"the imported node graph gets no structural
> validation, and the file's own 'three walls' enumerate none."*

## Why

`export.rs:15-17` states the file's own law: *"it consumes bytes some other process wrote, and
**every one of them can be a lie**."* `export.rs:2015` calls `import_export` *"the file's one place
where untrusted bytes become a runnable network."*

The header counts **three walls** — range refusal at the read (`expect_u16`/`expect_op`/
`expect_idx`), slot bounds as a post-pass (`check_program_slots` and friends), and three compat
gates (format version, ABI fingerprint, host `TypeEnv` field order). Every one is about a **value**.
**None is about the SHAPE OF THE GRAPH.**

Meanwhile two files state the graph invariant as a requirement, not a preference:

- `src/rete/kernel/node.rs:193` — *"The alpha/root-join/hash-join passes **require** ascending id
  order (topological)."*
- `src/rete/kernel/arm.rs:592` — *"ascending node id **is** the topological order every pass
  depends on."*

On the compile path that holds because ids are **minted** increasing. On the wire path nobody
checks. **This is the vigilia's Class A in one sentence: an invariant proven at one door and
assumed at all of them.**

## The measurement we already have — the probe, run before this stone was written

`probe.rs.txt` truncates a 7-node export to 3 and imports it. Verbatim, at HEAD `d024afb2e`:

```
IMPORT ACCEPTED A BROKEN GRAPH: kept 3 of 7 nodes, so every surviving parent's downstream
child id names nothing, and import returned Aggregate(... class: "wat::rete::Session" ...)

  AlphaNode    id=0  children=[1]   -> 1 exists
  RootJoinNode id=1  children=[2]   -> 2 exists
  TestNode     id=2  children=[3]   -> 3 DOES NOT EXIST
  next-id: 3
```

It passed all three existing walls and produced a runnable `Session`. Not a hypothetical.

## The algorithm

One pass over `network_pairs`, after phase 3 builds it and **before** phase 4 reads the side
tables. Collect the id set; then for every node prove: every child id resolves to a node; every
`node_ref_alpha_id` resolves to a node **whose kind is `Alpha`**; and every child id **exceeds its
parent's**. Refuse with `malformed(span, IMPORT_OP, …)` — the shape the other three walls already
use — naming the offending parent, the offending edge, and which of the three rules broke.

## ★ THE ONE CONTRACT DECISION

**The wall REFUSES; it never repairs, reorders, or prunes.** An import that cannot be walked is
not a network to be salvaged into a smaller one — it is a lie about a network, and the honest
answer is the same `MalformedForm` the read-level walls give. Specifically: no dropping of
dangling edges, no topological re-sort, no synthesising of a missing node. A repair would make the
importer's output depend on the damage rather than on the input, which is the property that makes
a wall a wall.

## Blast radius

`src/rete/export.rs` only — one new private fn plus one call site inside `import_export`, between
the `network` binding and the `compiled_conds` loop. Plus the probe into
`tests/rete/probe_arc278_export.rs`. **No new types. No change to any pack side. No change to the
wire format** — this wall reads what is already unpacked, so it costs no bytes and no version bump.

## Out of scope — AFFIRMATIVELY CUT, not deferred

- **A2 (`acc.rs`'s wire-reachable `panic!` → `Result`).** Cut from THIS strike and sequenced
  after it *on purpose*: the wall changes which malformed shapes can reach those panics, so
  measuring A2's reachability before the wall exists would measure the wrong surface.
- **A6 (unbounded `unpack_expr` recursion) and A7 (O(N²) import, missing ceiling calls).** Same
  door, different failure — depth and cost, not shape. Separate strikes.
- **Validating the five side tables' keys against the node set.** Considered and rejected: a
  `conds`/`drivers`/`progs`/`folds` entry for an id that names no node is *inert* — nothing reads
  it — whereas a graph edge to a missing node is walked. Different severity, different wall; do
  not smuggle it in here.

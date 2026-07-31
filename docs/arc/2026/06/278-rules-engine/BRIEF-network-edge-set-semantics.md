# BRIEF — make `network-add-child` idempotent (rete network edges are a SET)

Design: `DESIGN-STONE-network-edge-set-semantics.md`. Read it first; it carries the measurement and
the rejected alternatives.

## The work, in one paragraph

`:wat::rete::network-add-child` appends a child-id to a node's `children` unconditionally. When two
rules share a compiled node — which the `find-or-mint-*` dedup makes routine — the same edge is
wired once per rule, so one shared node ends up with N identical out-edges. The fire loop iterates
edges, so it does the same work N times over an already-inflated input, and the shared hash-join
materialises `M·N³` tokens. Make the helper idempotent: if the child-id is already present, return
the network unchanged.

## Read in order (the rooms)

1. **`wat/rete.wat:405-419`** — `network-add-child`, the function you are changing. Note `old-ch`
   is already bound; the guard goes between that binding and the `conj`.
2. **`wat/rete.wat:1478-1483`** — **copy this shape.** It is the identical idempotent-conj pattern
   (`if contains? acc f → acc, else conj`) already in this file, and it is what your edit should
   read like.
3. **`wat/rete.wat:997-1002`** — proof the primitive applies to exactly this data: a children
   `PersistentVector` probed with an `i64` node-id via `PersistentVector/contains?`. Argument order
   is `(contains? <collection> <item>)`.
4. **`wat/rete.wat:730-762`** — `compile-condition`'s alpha+join branch, steps 3 and 4. This is
   where the duplicate edges come from. **Do not change it** — it is reading room, so you can see
   that the two call sites need no guard once the helper has one.

## Implementation sketch

Fill this in; do not invent a different shape.

```clojure
(:wat::core::defn :wat::rete::network-add-child
  [network  <- :wat::core::PersistentMap
   node-id  <- :wat::core::i64
   child-id <- :wat::core::i64]
  -> :wat::core::PersistentMap
  (:wat::core::let [node   (:wat::core::Option/expect
                              (:wat::core::PersistentMap/get network node-id)
                              "network-add-child: node not found")
                    old-ch (:wat::rete::node-children-ids node)]
    (:wat::core::if (:wat::core::PersistentVector/contains? old-ch child-id)
      network
      (:wat::core::let [new-ch   (:wat::core::PersistentVector/conj old-ch child-id)
                        new-node (:wat::core::Record/assoc node :children new-ch)]
        (:wat::core::PersistentMap/assoc network node-id new-node)))))
```

Update the doc comment above it (`:405-408`) to state the set semantics — that a child already
present is a no-op, and why (a rete edge means "propagate to this child"; a second identical edge
would mean "propagate twice", which no caller wants).

## Blast radius

**`wat/rete.wat` only, this one function body plus its doc comment.** No `src/` Rust. No other
`.wat`. The `children` field's shape does not change, so no reader is affected.

## Your gates

Run these in the FOREGROUND and wait for each to finish. Do not background a command and return.

1. `cargo build --release` — must exit 0.
2. `cargo test --release --lib -- a8_node_share_fire_census --nocapture` — the RED gate. It fails
   today. It must PASS after your change, and its printed table must show `RootJoin` and `HashJoin`
   token counts **flat in N** with `derived` still 50 at every N. Paste the table in your report.
3. The stdlib load-order gate, since you touched `wat/`: a two-line `:user::main` that prints
   `(:wat::deporder::verify-stdlib)` must print `[]`.

Do **not** run the full `cargo nextest run` — the orchestrator weighs the floor centrally.

## STOP triggers

Each of these means: **ship nothing, report what you found.** None is a licence to adapt.

- **STOP-1** — `PersistentVector/contains?` does not type-check at this site (e.g. the checker
  rejects it against `node-children-ids`' return type). Report the exact located error. Do not reach
  for a hand-rolled fold or a different primitive.
- **STOP-2** — gate 2 still fails after the change, or its table shows `RootJoin`/`HashJoin` still
  growing with N. That means a second edge-duplication source exists that this design did not find.
  Report the new table verbatim.
- **STOP-3** — `derived` is no longer 50 at every N, or the load-order gate prints anything but
  `[]`. Report immediately; the change altered results, which it must not.
- **STOP-4** — you find yourself wanting to edit any test's expected value to make something pass.
  An assertion that only held because of the duplication is a finding, not a chore. Report the test
  name and its assertion verbatim and stop.

## Report back

The three gate results, the census table from gate 2, the diff of what you changed, and anything
that surprised you. Do not commit; the orchestrator weighs and commits.

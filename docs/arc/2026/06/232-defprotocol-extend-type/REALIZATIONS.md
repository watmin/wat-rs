# REALIZATIONS — arc 232 (defprotocol / extend-type)

## 2026-06-14 — The seats were already carved: reach-and-find, the dividend of honest foundations

Designing defprotocol from scratch should have been a paradigm-sized build. It wasn't — because
almost every tool it needed had already been laid down, in prior arcs, *before anyone knew
defprotocol would need them*. Each crawl for an integration point returned the same answer: it's
already there, already shaped for this, and the comment anticipated you. The whole design fell into
place in single reads.

The evidence, each found in one crawl:

- **The registration mold was waiting.** `defclause` (arc 237) is the exact pattern defprotocol +
  extend-type mirror: a `parse_*_form` → a `Value` in `runtime_def_values` → mirrored into `CheckEnv`
  by `from_symbols`. 232.1 was "do what defclause does, twice." No invention — imitation of a
  proven mold.
- **The satisfaction edge was pre-authorized, with a comment naming this use.** `register_subtype`
  (types.rs:446-449) carries, verbatim: *"Edges from unregistered names are allowed: the hierarchy
  is orthogonal to the TypeDef registry — a tag can derive regardless of whether it has a TypeDef
  entry. This mirrors Clojure's hierarchy being independent of what the tags ARE."* That comment was
  written for the record-subtype work, before defprotocol existed — and it is *precisely* the
  property that collapsed 232.2 from a feared `TypeDef::Protocol` churn to a single `register_subtype`
  call. The foundation foresaw the use and said so out loud.
- **The graph was already multi-parent.** `is_subtype` (types.rs:3076) walks a transitive DAG via
  `subtype_parents` (a *list*). So a record extending both `:wat::Record` (from recordtype) and a
  protocol (from extend-type) just works — multi-protocol extension, free, no design for it.
- **`assignable` already consults the edge.** check.rs:13566 returns true on `is_subtype(ap, ep)`
  for Path→Path. The satisfaction edge flows through with ZERO change to `assignable` or `is_subtype`.
- **The connection layer was already host-blind.** The C0b.2e unification (Peer'/Listener'/Address'
  as `Box<dyn>` Comm* traits) is *why* the host-agnostic-service problem localized to `start` alone
  — the surface was already transport-blind before we asked it to be.

This is the **complement of the reach-stumble.** Reach-stumble: you reach for a tool, find it
ABSENT or wrong, and that friction is the signal to build it ([[feedback_reach_stumble_is_the_signal]]).
Reach-and-find: you reach for a tool and find it *already carved for the use* — the dividend an
honestly-built foundation pays its own future. When a crawl returns "it's there, and the comment
anticipated you," that is the substrate's prior coherence paying interest.

The nuance that makes it honest rather than self-congratulatory: the **one** tool that was NOT
pre-laid was the open type-bound itself — defprotocol — and we found *that* gap the hard way, by a
disconfirming probe (`NoMatchingClauseAtCallSite` on an abstract forwarded arg). That's the tell
that distinguishes the two: a pre-laid seat is found in one read with an anticipatory comment; a real
gap fails the probe. The honest move on the gap was to stop and build the dep
([[feedback_deferred_dep_becomes_necessary_block_and_build]]); the gift on everything else was that
the dep, once built, had almost nothing left to invent — the seats around it were carved.

The lesson for how we build: foundations laid for honesty (not for a specific future) are the ones
that carve the most seats in advance. `register_subtype` wasn't made flexible *for protocols* — it
was made orthogonal because that was the honest shape of a hierarchy, and the honesty is what let it
serve a use its author hadn't seen. Build the floor true; the rooms you don't yet know you need will
find their footings already poured.

> To fold into the 170 chronicle later (builder's call) — sibling to the recent 200-arc local
> realizations; a song-worthy beat (REACH-AND-FIND / THE-SEATS-WERE-ALREADY-CARVED).

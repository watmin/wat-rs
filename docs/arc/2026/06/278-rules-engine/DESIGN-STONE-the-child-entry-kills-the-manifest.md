# DESIGN STONE — the CHILD ENTRY kills the manifest

**Arc 278. Drawn 2026-08-12, after the peer edge landed (`310f8050`) and the composition was
proven by run.** Status: DRAWN, unbuilt. Blast radius: every `defservice` in the corpus.

## Why

`defservice` ships its forked child a **hand-enumerated manifest** (`<fqdn>::service-forms`), while
its sibling `wat/bracket.wat` ships a `fn-forms` closure plus a one-liner main. The manifest is not
a design choice — it is a **workaround for the extractor's reach**, and pulling on it is what
exposed the whole arc's form-transmission work.

The manifest cannot be replaced by a closure walk today, for one reason: the generated child main
reaches its own internals **dynamically**.

```
wat/service.wat:2101   (apply (keyword/from-string ~dispatch-admin-name-str) ship [])
wat/service.wat:2120   (apply (keyword/from-string ~serve-name-str) self …)
```

A closure walk cannot follow that. **You cannot root a walk at a call that exists *because* it does
not resolve statically.** So the manifest has to enumerate by hand what the walk cannot reach — and
whatever the hand forgets, a user's service loses across the fork.

## The one contract decision

> **`defservice` emits a per-service `<fqdn>::child-entry` — a REAL parent `defn` that names
> `serve` and `dispatch-admin` STATICALLY — and the shipped `:user::main` becomes a one-liner that
> calls it with the rendezvous locus.**

Everything else follows: one `fn-forms` over `child-entry` replaces the manifest, and
`service-forms-def` dies.

The locus is a **parameter**, not a free name in the entry. MEASURED (`probe-arc278-free-user-name-
in-parent-defn.wat`): a free `:user::` name in a parent `defn` types as `:wat::core::keyword` and
refuses any typed use — *that* is why today's `child-main-form` is quasiquoted data rather than a
defn. The free name appears only in the shipped one-liner, where it is checked in the child, in
which it IS defined (the `ProcessOpts` launch arm prepends `(def :user::spawn::service-locus …)`).
Bracket already works exactly this way.

## Why it is possible NOW and was not before

Two facts, both established this session, both by run:

1. **The peer edge exists.** `serve`'s `self` is `ThreadSelfPeer'<Status,Admin>`; the process
   tier's child holds a `Peer'<Status,Admin>` from `self-peer`. Those are distinct heads with no
   relation until `310f8050` stated the safe one (`Peer' derives ThreadSelfPeer'`,
   `wat/spawn.wat`). Before that edge a static call was a **located TypeMismatch** — the strike was
   *impossible*, not merely unwritten.
2. **The static-call mechanism is already in the file, 20 times.** `serve` recurses on itself via
   `(~serve-name self l selectables next-id new-state)` — `serve-name` (`service.wat:731`) is a
   keyword node spliced into a call head. `dispatch-admin-name` (`:854`) is the same shape. The
   `apply` was never needed for the *generated* main; it is needed for the **generic `Locus/launch`
   impl**, which serves all services and genuinely cannot name a per-service fn. That one dynamic
   hop stays, at the boundary where genericity actually lives.

## PROVEN BY RUN — the disconfirming probe

`wat-scripts/scratch-pad/probe-arc278-child-entry-static-call.wat` (committed with this stone,
loader-gated). It builds a real `defservice` and a `child-entry`-shaped defn, and settles both
load-bearing claims:

| claim | result |
|---|---|
| **A** — a `Peer'<Status,Admin>` reaches `serve`'s `ThreadSelfPeer'` slot in a STATIC call | `--check` **exit 0** |
| **B** — `fn-forms` rooted there reaches the service internals | **`CLAIM-B PASS`**, closure = **30 forms** |

The 30 declared names include `serve`, `dispatch-admin`, `init`, `stop-project`,
`hibernate-project`, `State`/`Record`/`Admin`/`Status`/`Op`, both `Kwargs` structs, the surface and
its `$core-record`/`$holon-record`, the protocol `Op`/`Reply`, the per-op budget const. **That is
the manifest — derived rather than remembered.** The probe prints the whole name set beside the
count, so an empty walk cannot masquerade as a pass.

Two form-corrections the checker taught while writing it, both one-shot, both worth copying: the
selectables element is ONE tuple type-keyword (`:(wat::core::i64,…)`), and its `Op` slot must be
the **service** superset (`probe::ce::Op`) — the surface→service widening is proven for a bare
`Peer` compare but **does not propagate through the tuple**.

## Out of scope — REJECTED, not deferred

- **The `Locus/launch` `apply`.** It stays. A generic impl cannot statically name a per-service fn;
  that is honest genericity, and it is the one hop the closure never needs to follow.
- **Touching the thread tier.** The thread arm ignores `service-forms` entirely — *"serve is
  already in the parent universe"* (`spawn.wat:450`). Shared memory means nothing to ship. This
  stone changes what the PROCESS arm ships and how it is reached; the thread path is untouched.
- **Remote loci.** Not built, deliberately (builder: *"we are not touching any of them until
  processes are complete"*). This stone is what makes them cheap: they are all wire, they all hand
  over a `Peer'`, and they all join with one `extend-type` — no `defservice` edit.
- **The dedup question.** A union of N roots declares shared types N times. The one-entry model has
  ONE root, so the question does not arise here. (If it ever does: dedup on `(head, name)`, never
  name alone — a `recordtype` and its kwargs `defmacro` are two facets of one concept, and this
  arc's census counted 182 such names.)

## The four questions

- **Obvious?** YES — "the child runs the service's entry" is one sentence, and the shipped main
  reads as one line that says exactly that.
- **Simple?** YES — one emitted defn replaces a hand-enumerated bundle; one `fn-forms` call
  replaces `service-forms-def`. Strictly less machinery.
- **Honest?** YES — and this is the load-bearing gain. Today the child main's reach is *dynamic in
  order to evade the checker*; after, it is static and checked. The manifest's silent-omission
  failure mode (a form nobody listed) becomes structurally unreachable: the walk derives the set.
- **Good UX?** YES — a user's service stops losing declarations across the fork, which is the
  defect that opened this whole thread.

## Files

`wat/service.wat` — the only file this stone edits. `child-main-form` (≈`:2065-2125`),
`service-forms-def` (≈`:2126-2190`), and the two `apply` sites.

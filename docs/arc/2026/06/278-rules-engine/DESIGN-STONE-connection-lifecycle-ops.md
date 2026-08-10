# DESIGN STONE — `-on-connect` / `-on-disconnect`: a service is told when a connection begins and ends

> **Drawn 2026-08-10.** Nothing built. Depends on `DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md`
> (ruled + built, `037ef43e`): ctx is mandatory, the leading `-` is the discriminator, and
> `SelfInvocation` / `LifecycleInvocation` already exist on disk.

---

## Why this exists — the chain, and where it closes

We wanted **concurrent rete worlds**, one per connection, isolated. Working backwards:

1. Isolation itself was never missing — `:wat::eval-with-defs!` (`src/runtime.rs`, arc 170) evaluates
   against a **supplied** definition set, so two connections cannot collide by construction.
2. What was missing is **holding** a world per connection *across messages*.
3. That needs a stable per-connection identity in the handler → **ctx** (`conn-id`). ✔ SHIPPED.
4. And it needs the service to be **told** when a connection begins and ends, so it can create and
   destroy its own entry. ✘ **This stone.** All lifecycle arms pass `state` through unchanged today,
   so a service literally cannot observe a connect or a disconnect.

That is the whole remaining gap. Everything else on the isolation path is standing.

---

## ★ The mechanism, grounded — and it is NOT the alarm path

An internal op reached by a fired alarm arrives as **`ServiceEvent::Message idx op`** — the `Alarm`
**carries an `Op` variant** (`(:wat::service::Alarm :after … :op :-tick)`), so when the timer fires
it comes back through the ordinary message path and `serve-dispatch-op` matches it like any other op.

**A lifecycle event carries no op.** `ServiceEvent::Connection [peer]` and
`Closed`/`Lost`/`Rejected [idx …]` (`wat/spawn.wat:195-203`) have no `Op` field and never will —
nobody sent a message. So the macro must **synthesize** the dispatch at those arms:

| arm (`wat/service.wat`) | today | after |
|---|---|---|
| `:1504` `Connection peer` | mint id, `conj`, recur | …then dispatch `-on-connect` if declared |
| `:1593` `Closed idx` | `remove-at`, recur | dispatch `-on-disconnect` **before** the removal |
| `:1603` `Lost idx cause` | assert cause loud, `remove-at`, recur | same |
| `:1633` `Rejected idx cause` | reply `Failed`, `remove-at`, recur | same |

**Dispatch BEFORE the eviction** — the arm needs `(first (nth selectables idx))` to read the
`conn-id` out of the tuple, and after `remove-at` it is gone.

## THE ONE CONTRACT DECISION

**Both ops are OPTIONAL. A service that declares neither behaves exactly as it does today.**

169 public arms were migrated hours ago to make ctx mandatory; forcing every service to also declare
two lifecycle ops would be a second corpus migration for a facility almost none of them want. The
macro emits the dispatch **only when the arm is declared** — absent, the arms keep their current
bodies byte-for-byte.

This is the ruled division restated (`DESIGN-STONE-mandatory-ctx-and-lifecycle-ops.md` § 5): *the
macro reports IPC facts; the user owns policy.* Being told is not being made to care.

---

## ⛔ ONE MAP, NOT TWO — the state IS the value's type

The builder asked whether `:ephemeral` needs two mappings, one for the session and one for its state.
**No — and two is the defect.**

Two maps keyed on the same `conn-id` is the parallel-structure shape STOP-2 already forbids one level
down (the `conn-ids`-vector-beside-`selectables` that desyncs the moment a timer is removed from one
and not the other). Two things that can disagree, with nothing forcing them to move together: a world
removed and a state left behind, or a state advanced past the session it describes.

```clojure
:ephemeral [worlds <- :wat::core::PersistentMap<i64, <UserState>>]
```

where `<UserState>` is the **user's own enum**, whose constructor IS the phase. One key, one lookup,
one thing to remove; the phase cannot desync from the payload because it is the payload's tag. A
separate `phase` field would be rung 2 of the extirpare ladder; this is rung 3 for every state whose
payload differs.

**Honest bound:** two states carrying the SAME payload (e.g. a rete world's `Ready(Session)` vs
`Fired(Session)`) are still structurally interchangeable, so those transitions are refused by the
match arm, not by absence of form. Rung 2 there. Say so rather than overselling the enum.

---

## The exemplar to build — the ratchet with ZERO rete in it

**Deliberately not the rete service.** The lifecycle dispatch is undesigned substrate; the rete state
machine is domain design. Built together, a failure in either looks like a failure in both — and the
ctx strike already showed what an unproven precondition costs. So the first consumer is
structurally identical to the rete protocol and contains no rete:

```
-on-connect          → Provisioned              entry created
set-limit            → Ready(n)                 ONCE; a second is refused
bump          × N    → Ready(n')
seal                 → Sealed(n')
read          × N    → Sealed                   before seal → refused
-on-disconnect       → entry removed            ALL THREE eviction paths
```

Same ratchet as `install-rules → insert-facts → fire-rules → query`, same monotonic law (**each step
forward closes every prior operation**), same named refusal per illegal transition — and if it
breaks, the cause can only be the lifecycle mechanism.

Each illegal transition gets its **own** response variant (`AlreadySealed`, `NotYetSealed`,
`LimitAlreadySet`), never one generic error: they tell a client different things it can act on.

---

## ⛔ OPEN — the disconnect REASON, and I will not guess it

`-on-disconnect` fires from three arms carrying different information: `Closed` (no cause), `Lost`
(a `Failure`), `Rejected` (a `Failure`). The cause must not be dropped — that is the masking class
this arc exists to delete.

But `LifecycleInvocation` is shared with `-on-connect`, which has **no** reason. Putting an optional
`reason` on it rebuilds the exact cannot/did-not-look ambiguity that option B was chosen to kill.

Candidate shapes, **for the builder to rule**:

- **(i)** a fourth context type — `DisconnectInvocation` = core + `conn-id` + `reason` — with
  `-on-connect` keeping `LifecycleInvocation`. Consistent with "the ctx type tracks the kind of
  invocation," which is the rule that won B. Costs a fourth type.
- **(ii)** one `-on-disconnect [s ctx]` where the reason rides *nothing* — the cause is surfaced on
  the honest loud sink as it is today and the handler simply is not told. Cheapest; drops
  information the handler might legitimately want (evict-on-abuse policy differs from clean-close).

I lean **(i)** on the same reasoning that produced the three-type split, but the fourth type is a
real cost and this is exactly the sort of call I have gotten wrong alone today.

---

## The gate

1. **`-on-connect` fires and its ctx is populated** — `conn-id` matches the id the same client's
   public ops later observe, `namespace`/`operation` are the compile-time literals.
2. **★ THE LEAK GATE — all three eviction paths remove the entry.** Connect three clients, then
   evict each way: one clean `Closed`, one `Lost`, one `Rejected` (an over-budget frame). Assert the
   map is empty after. **`Rejected` is the path the seam had lost and the one a hand-written teardown
   forgets** — a gate that only tests `Closed` proves nothing about the other two.
3. **★ ISOLATION + STABILITY.** Connect three, advance each to a *different* state, disconnect the
   MIDDLE one, and assert the two survivors' states are untouched and still their own. This is the
   cross-tenant leak the whole design exists to prevent, and it is the defect most likely to ship
   green.
4. **A service declaring neither op is unchanged** — the byte-for-byte no-op proof, so the 169
   migrated arms stay exactly as they are.

## NOT in this stone — affirmatively cut, not deferred

- **The rete service.** Its states (`Provisioned`/`Installing`/`Ready`/`Fired`), its chunked
  `install-rules`, its query surface. Next stone; this one proves the mechanism it will use.
- **Chunked rule transmission.** Code is EDN and we already transmit large EDN strings; the client
  chunks, the server concatenates, and **the parse is the check** — a mis-assembled string fails
  `read-string` with `ReadOutcome::Malformed`, which catches well-ordered garbage that no sequence
  counter would. Its home is #18 `write-*-batched` / #19 the reader side, both pending. Do not invent
  a second paging mechanism beside them.
- **Update-in-place and re-fire.** A different deployment, by the builder's ruling. So is the durable
  distributed rete.
- **`sift-rules` is NOT prior art for this.** It is alpha-only, no beta joins, and sits *upstream* —
  the DDB-shaped query with a server-side filter that produces the facts rete-as-a-service consumes.
  Two tiers of one pipeline, not two attempts at one problem. (Recorded because the apparatus reached
  for it as precedent on adjacency alone.)

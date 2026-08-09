# DESIGN STONE — ctx is MANDATORY, arity is not a discriminator, and lifecycle is an internal op

> **Ruled by the builder 2026-08-09**, in one thread, after the `Invocation` rename landed. This
> stone SUPERSEDES the opt-in third-param design that shipped in `c8fcfe0d` and the arity table
> `BRIEF-the-call-context.md` recorded. Nothing here is built yet.

---

## The rulings, in the order they were made

### 1. ctx is delivered ALWAYS — never opt-in

> *"i think the user must accept a ctx always - they can choose to ignore it, but it must be
> delivered - optional things always bite us eventually"*

The opt-in design is dead. Every op arm receives a context whether or not it reads it.

### 2. Arity is NOT how the shape is decided

> *"and using arity to measure this is insane - that's such a shitty idea"*

The shipped design inferred an arm's calling convention from its parameter COUNT:

```
1 param  [s]            internal          ← the table that is now dead
2 params [s req]        public, no ctx
3 params [s ctx req]    public, wants ctx
```

**That is meaning-derived-from-position**, and it is the same class of defect this arc spent
2026-08-09 dismantling one level down: `idx` is a seat number, not a name, and anything whose
meaning comes from a position breaks when the world shifts. The table also ran out of room the
moment a second kind of thing needed a payload — an internal op carrying data has NO form under it,
because arity 2 was already spoken for. That collision is not a quirk to route around; it is what
positional schemes do.

**The discriminator is the NAME.** A leading `-` marks an internal op (already true —
`is-internal`, `wat/service.wat:989`; already proven by `-tick` in
`tests/services/probe_arc278_self_scheduling.wat:54`). The shape FOLLOWS from the name:

```clojure
(op-name  [s ctx req])   ;; public   — a client sent something, so there is a request
(-op-name [s ctx])       ;; internal — nobody sent anything, so there is no request
```

Arity becomes a *consequence*, never an input to the decision.

### 3. An internal op is STILL an invocation — it gets a ctx

> *"an internal op needs to be given a invocation id and an invocation time - ctx arrives just the
> same - there's just no user input to receive... if we want to log that we're doing some action,
> or recording telemetry about it... we need that ctx to provide it"*

A fired timer, a connection opening — those are things the service DID, at a time. Without a ctx
they are invisible to telemetry, which is the half of the system this whole context exists to feed.
"No caller" does not mean "no invocation."

This REVERSES `BRIEF-the-call-context.md` STOP-3 ("internal ops get NO ctx at all"). That STOP was
right that an internal op has no *caller*; it was wrong to conclude it has no *invocation*.

### 4. Lifecycle events are internal ops — `:impls` only, NEVER the surface

> *"we need an internal op for something like `-on-connect` `-on-disconnect` etc etc.... this is
> where we handle these life cycle things - they receive our self-state and telemetry-ctx"*

Grounded precedent (`probe_arc278_self_scheduling.wat`): the surface `:probe::Ticker` declares only
`:features [start, poll]` — the client-facing ops with request/response pairs. `-tick` appears
**only in `:impls`**: no request type, no response type, no wire presence. The substrate dispatches
it; no client can name it.

Lifecycle is identical in kind — substrate→service, no caller, nothing crossing a wire. Putting it
in the surface would lie to every client that dials the service.

### 5. The macro reports IPC FACTS; the user owns POLICY

> *"as we cannot predict what any given service will do with their connections, we cannot impose
> this restriction in the macro - nor should we - the macro is ensuring we never fuck up IPC - the
> user must perform any additional connection tracking they must in their own code.... they need to
> manage state in the ephemeral struct and do whatever"*

**KILLED by this ruling:** an earlier proposal in which the macro owned a conn→world map, created
entries on connect and destroyed them on eviction, via a `:per-connection` state slot. That is the
macro imposing policy about a thing it cannot predict.

The division:
- **macro** — surfaces the facts only it can know: this invocation's id, time, namespace,
  operation, and (where one exists) its connection; and that a connection arrived or went away.
- **user** — holds whatever state they want in `:ephemeral` and decides what any of it means.

A user who ignores an eviction path then leaks their own map. That is their bug — but it cannot be
a SILENT one, because the event is an enum-shaped fact and `109/NOTE-full-enum-match-mandatory-no-
wildcard-arm.md` forbids a wildcard arm on an enum scrutinee. Facing every variant is the
substrate's existing law about enums, not the macro deciding policy.

---

## ★ THE SHAPE, RULED — two context types (option B), 4/4 on the four questions

`Invocation`'s five fields split cleanly for a public call. Four of them are unambiguous for an
internal op too — `request-id` (mint one per dispatch), `start-ns`, `namespace` (the service fqdn),
`operation` (the op's own name). **`conn-id` is the one that does not survive the crossing:**
`-on-connect`/`-on-disconnect` are *about* a connection; `-tick` has none.

Two candidate shapes were weighed with the four questions, flat, no "medium":

| | **A** — one type, `conn-id: Option` | **B** — two types, the op's kind picks |
|---|---|---|
| Obvious? | **NO** | **YES** |
| Simple? | **NO** | **YES** |
| Honest? | **NO** | **YES** |
| Good UX? | *moot* | **YES** |

**Why A fails Obvious.** A reader meets `conn-id <- Option<i64>` and cannot answer *"when is this
`None`?"* from the field. They must go learn which op kinds exist and which are
connection-originated. The type does not say the thing it must say.

**Why A fails Simple.** It braids two concepts under one name — *an invocation*, and *an invocation
caused by a connection*. The `Option` is the seam between them.

**Why A fails Honest — the decisive one.** An `Option` truthfully reports *absence*; it cannot
report *which* absence. A reader cannot distinguish "no connection caused this dispatch" from "the
plumbing did not resolve one." That is exactly the conflation ruled out 2026-08-08
(`feedback_none_means_skip_conflates_cannot_with_did_not_look`) — one default covering *cannot* and
*did-not-look*. A telemetry consumer seeing `None` learns nothing about which it is.

**Why B passes Honest structurally.** There is **no field to read**, so a handler cannot ask a timer
for a connection. Top of the extirpare ladder — unrepresentable, not merely caught. A only ever
reaches "caught, if you remember to unwrap."

**The count objection, answered.** B is two types where A is one. The four questions measure
cognitive load per piece, not inventory; each B type is a flat record of always-populated fields
with no internal branching. The complexity moved from a runtime `Option` to a compile-time type
choice the op's own name already makes.

**B's one real cost, and the existing idiom that pays it.** Telemetry wanting to log *any*
invocation uniformly needs a shared core — `namespace`, `request-id`, `start-ns` — carried by both.
That is the `~@` **surface-splice**, already shipped for exactly this shape (`Scope` spliced into
`Metric` and `Log`). Not new mechanism; the mechanism this substrate already chose.

**⚠ WHAT WOULD FLIP IT** — the one thing to prove before briefing: that the splice actually
composes here, i.e. a shared reader can accept both context types without a union. Checkable with a
small probe, not an argument. If it does not compose, B's cost rises and the verdict is back open.

---

## Consequences

**The migration is a codemod, and it is the cheap kind.** Every `[s req]` arm becomes
`[s ctx req]` — 120 arms across 65 defservice files, enumerated BY THE CHECKER rather than a grep,
swept by a `wat-scripts/fixes/` codemod. This is R65 `True American Hate` exactly: the wildcard ban
and mandatory arity make the compiler an enumerator, so a change of meaning becomes a finite located
worklist. B does **not** enlarge it — public arms all take the connection-originated type either
way.

**Self-indictment, kept visible.** The opt-in design was chosen *specifically to avoid* migrating
those 120 sites (`BRIEF-the-call-context.md` STOP-1: *"the opt-in third param is the entire reason
this design was chosen"*). That is the argument R65 names as wrong, applied one day after it was
recorded: the verbosity is prepaid refactoring capacity and the migration is the coupon. The
apparatus dodged a migration in a substrate built to make migrations cheap, and paid for it with a
positional dispatch table that ran out of room within a day.

**World lifetime, for the record (not this stone's build).** A connection's rules are compiled
EXACTLY ONCE, when its world is provisioned, and never change — new rules mandate a new connection.
The client transmits its entire rule definition in one bootstrap message, then inserts and fires
against the frozen world. So world creation is **asymmetric**: `Connection` only mints the
`conn-id`; the world comes into being on the bootstrap op, and is destroyed on all three client
eviction paths (`Closed`, `Lost`, **`Rejected`** — the third is the over-budget-frame path the seam
had omitted). An insert or fire arriving BEFORE bootstrap needs a named response variant; an empty
world would accept facts and derive nothing, which is a silent no-op the no-hidden-failures law
forbids.

**Isolation is already structural and already built.** `:wat::eval-with-defs!`
(`src/runtime.rs:24327`, arc 170) evaluates a form against a world built from a **supplied**
definition set rather than an ambient symbol table. Two connections passing different defs cannot
collide, because neither is registered anywhere global. The live `Environment` threads through
unchanged, so a bound peer survives a re-freeze. What is NOT built is holding that world per
connection across messages — and the only thing blocking it is that the lifecycle events do not
exist, which is what this stone is for.

⚠ `eval-with-defs!` is **deliberately slow** by its own doc — it re-derives the entire world on
every call, as the R1/R9 correct-but-slow ORACLE, with a fast incremental data plane to be built
behind a differential. So the per-connection world must hold the **frozen** bootstrap result, never
the defs text. Storing defs and re-deriving per message would build the oracle into the hot path.

---

## Not decided here

- Whether `(-op [s ctx])` is representable today, or whether the current arity handling reads a
  2-param internal op as a public one. **A ten-line probe answers it; do not assert it.**
- The two context types' NAMES. `Invocation` is ratified for the connection-originated one; its
  self-originated sibling is unnamed and owes an intueri cast.
- The lifecycle event set beyond `-on-connect` / `-on-disconnect`, and whether the three eviction
  paths collapse into one op with a reason or stay distinct ops.

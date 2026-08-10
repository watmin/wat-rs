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

## ✔ PRECONDITION (a) — ANSWERED BY PROBE, 2026-08-09. It is worse than "not representable."

The question was whether `(-op [s ctx])` works today. **It type-checks green and silently drops the
second binder.**

Probe: `tests/services/probe_arc278_self_scheduling.wat` copied to `/tmp`, its `(-tick [s])` changed
to `(-tick [s ctx])`, `target/release/wat --check` → **exit 0, no output**. Then the body was made to
actually REFERENCE `ctx` (`[_c ctx …]` in its `let`) → **still exit 0, no output.**
(Exit code read directly, not through a pipe — the first attempt read `head`'s status, the trap
`CLAUDE.md` names.)

**The mechanism, read at the emission site** (`wat/service.wat:1107-1111`): the internal branch
builds

```clojure
binding-items  [s-binder, state]                        ;; TWO items, always
let-bindings   (with-children param-vec binding-items)  ;; takes param-vec's SHAPE, not its contents
```

so the emitted `let` binds `s`→`state` and **nothing else, whatever `param-vec` contains.** A second
param is not rejected, not bound, not mentioned — it is read past. And because `--check` is not a
complete RED arbiter for unbound symbols (`[[reference_check_is_not_a_complete_red_arbiter]]`), a
body that uses the dropped binder also passes.

**Not a regression from the ctx strike.** The internal branch's `let-bindings` is documented in-file
as byte-identical to its pre-strike form; the ctx strike added arity dispatch to the PUBLIC branch
only. This is a latent silent-discard the design walked into, not one it created.

**Consequence for this stone:** `(-op [s ctx])` is not a form that needs permitting — it is a form
that currently LIES. The internal branch must be changed to bind a ctx, and until it is, any
`.wat` written to this design would compile and quietly do nothing. That is the shape of defect this
arc exists to delete, so it is a STOP for the strike, not a footnote.

## ★ THE STRUCTURE, RULED — THREE records over ONE spliced core, and NO Options anywhere

Weighing B surfaced that there are **three** kinds of invocation, not two, and their field sets are a
strict NESTING. Expressed with the shipped `~@` surface-splice (`wat/telemetry.wat:84`, `Scope` into
`Metric`/`Log`):

```clojure
(:wat::core::defsurface :wat::service::InvocationCore          ;; name owed a cast
  :nature :wat::core::Record
  :features [namespace  <- :wat::core::keyword     ;; service fqdn — compile-time literal
             operation  <- :wat::core::String      ;; op arm's name — compile-time literal
             request-id <- :wat::core::Uuid        ;; minted by the SERVER, per dispatch
             start-ns   <- :wat::core::i64])

(:wat::core::defrecord :wat::service::<A>          ;; SELF-originated: a timer fired
  [~@:wat::service::InvocationCore])

(:wat::core::defrecord :wat::service::<B>          ;; CONNECTION-originated, no client message
  [~@:wat::service::InvocationCore                 ;;   -on-connect / -on-disconnect
   conn-id <- :wat::core::i64])

(:wat::core::defrecord :wat::service::Invocation   ;; a CLIENT CALL
  [~@:wat::service::InvocationCore
   conn-id   <- :wat::core::i64
   parent-id <- :wat::core::Uuid])                 ;; name owed a cast
```

### The caller's id — trust, and why it is NOT optional

The server mints `request-id` **unconditionally**; that is this invocation's identity. `parent-id`
is a *different* value: the id of the invocation on the CALLER's side that caused this one. It
arrives as data and is used **only to draw an edge** — never keyed on, never trusted, never adopted.
A hostile client can put anything there; the worst outcome is a false edge in its own trace. This is
W3C Trace Context's move (mint your own span, record the caller's as parent), applied rather than
invented.

**It is MANDATORY, not an `Option`.** The client is OUR generated code and always mints and sends
one, so a request lacking it is *malformed* — an existing named failure — not a legitimate absence.
The `Option` that earlier drafts carried was an artifact of forgetting we generate the client, and
it would have re-created the very cannot/did-not-look ambiguity that option B was chosen to kill
(for kinds A and B, "absent" would have meant *no client message exists at all*).

### The rejected alternative, expressed rather than gestured at

The builder cut the word "envelope" — *"to me its a bucket of runtime unpacking.... wat is strong
ADT .... let's see these expressed"*. Written out honestly it is not a bag, it is a typed generic
wrapper:

```clojure
(:wat::core::defrecord :wat::service::Call<R>
  [parent-id <- :wat::core::Uuid
   request   <- R])
;; arm: (op [s ctx call]) → every body must unwrap: (:wat::service::Call/request call)
```

**Rejected because it changes every request access in every body**, bolting a second migration onto
the ctx migration for the sake of one field. Materialising both forms is what settled it — the
apparatus had argued FOR this shape twice under the word "envelope," and seeing it written killed it
(R17 self-prompt-injection: reason against the real form, never the description of one).

## Not decided here
- The two context types' NAMES. `Invocation` is ratified for the connection-originated one; its
  self-originated sibling is unnamed and owes an intueri cast.
- The lifecycle event set beyond `-on-connect` / `-on-disconnect`, and whether the three eviction
  paths collapse into one op with a reason or stay distinct ops.

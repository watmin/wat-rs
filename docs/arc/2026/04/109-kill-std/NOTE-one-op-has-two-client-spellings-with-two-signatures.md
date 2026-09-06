# NOTE — one op has TWO client-method spellings, with two different signatures and two copies of every request guard

**Filed 2026-09-06, out of arc 278's `:max-entries` strike. Not fixed. Deliberately not worked in
that session at the builder's direction. Tracked here because it is a DEFINITION-FORM gap in
`defservice`/`defsurface` — it spans every serviceable op in the tree, not one surface.**

## The defect

A serviceable op is reachable under **two** generated client-method names:

| spelling | who builds it | example |
|---|---|---|
| **surface** | `src/runtime.rs` "Path B", hand-assembled `WatAST` | `:queue::Queue/send` |
| **service** | `wat/service.wat` `op-methods`, a quasiquote template | `:queue::queue/send` |

`fqdn-base` (`wat/service.wat:276`) is the `defservice`'s own name, so the macro emits the
lowercase service spelling; Path B fires for `:nature :Peer` surfaces, which have no aggregate
satisfier to look up (`src/runtime.rs:6347`).

`src/runtime.rs` already names the situation in its own comment:

> *"Path B is a **SECOND, independently-drifting copy** of the send-then-recv forwarding
> `wat/service.wat`'s `op-methods` also builds (this is the mechanism `:S/method` calls ACTUALLY
> run through — every corpus fixture calls the SURFACE name, never the service's own
> `<fqdn>/<op>`)."*

### Two consequences

**1. Every request guard must be written twice.** `:max-request-bytes` exists at
`wat/service.wat:2293` *and* `src/runtime.rs:6437`/`:6684`. Arc 278's `:max-entries` was therefore
written into both as well. The mechanism is generic per-cap, so a **new adopter costs nothing** —
but a **new KIND of guard** costs two implementations in two languages, and nothing enforces that
they agree.

**2. The two spellings do not accept the same argument type.** Measured
(`wat-scripts/scratch-pad/probe-does-anyone-hold-a-service-name.wat`):

```
:queue::Queue/send   accepts  [q <- :queue::Queue]                       ← the surface as a type
:queue::queue/send   rejects it, demanding (Peer :- [Queue::Op Queue::Reply])
```

★ This, not "clients hold surface-typed peers", is why the corpus is lopsided. Both methods take
the same surface-typed peer; only one accepts the shorthand annotation people write.

## What was measured, so the next reader need not re-derive it

Corpus census, 2026-09-06:

```
Queue/send 21   queue/send 0        Queue/ack 9   queue/ack 0
Store/put  30   *-store/put 0
```

Both spellings driven against one peer with one request:

```
surface-11 = RequestTooManyEntries(11,10)  depth 0
service-11 = RequestTooManyEntries(11,10)  depth 0
surface-10 = Ok  depth 0->10
service-10 = Ok  depth 10->20
```

★★ **The service spelling is NOT dead code.** It resolves, runs, and enforces `:max-entries`
identically. The two independently-written guards **agree today**. Deleting it would be a policy
call, not a fact — and the measurement argues against deleting.

## The fix, in the order it should be taken

1. **Gate it.** Promote the probe above to a floor test driving both spellings. Drift is currently
   *unobservable* — 0 corpus callers means no test can catch the day they diverge. This is the
   cheap rung and it closes the actual complaint.
2. **Make the signatures agree.** `queue/send` should accept `:queue::Queue` exactly as
   `Queue/send` does. Two spellings of one call with two parameter types is the reason one gets
   all the exercise.
3. **Then, and only then, decide whether two spellings should exist at all.** One implementation
   (Path B dispatching to the generated method, or the reverse) removes the class. That is a
   language-surface ruling, not a cleanup.

⚠ **Do not start at 3.** The path proposed for deletion is the one that works; the reason it looks
dead is defect 2.

## Will it bite before it is fixed?

**Assessed no, for arc 278's remaining work.** Adopting `:max-entries` on a new surface (the topic
batch) is a **declaration**, not new guard code: `defservice` reads the emitted def constants and
Path B reads `member.max_entries`, both generically, and both already conditional on the option
being present. The double-write was a one-time cost for the MECHANISM, not per-adopter.

It bites the next time someone adds a **new kind** of request guard.

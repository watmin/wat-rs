# DESIGN — the probe scan says which of six

Builder: *"let's keep going - do the 6 strings then the 258 recvs."* This is the 6 strings.

**Drawn 2026-09-15. NOT STRUCK.** ⛔ **Whether these arms raise is UNCHANGED, with ONE exception
reported and not fixed** (`Stopped`, below).

## WHY — NINE worlds, one message, seven of them false

`wat-scripts/queue/sqs.wat:586`–`:615`, inside `:queue::queue`'s fold counting landed rows by probe
scan. Every one of six arms raises the identical string:

```
"queue: probe scan failed — peer is dead, not a broken pipe"
```

| arm | what actually happened | is it "peer is dead"? |
|---|---|---|
| `_` over `ScanResponse` `:593` | the store **answered**, and returned a failure response | ⛔ **false — alive and talking** |
| `RecvOutcome::Lost` `:596` | the store peer died | ✓ true |
| `RecvOutcome::Closed` `:600` | clean EOF | ~ gone, but cleanly, not "dead" |
| `RecvOutcome::TimedOut` `:604` | the store is **alive and slow** | ⛔ false |
| `RecvOutcome::Stopped` `:608` | **the world is shutting down** | ⛔ false — **nothing failed at all** |
| `RecvOutcome::Malformed` `:612` | the store could not decode our frame | ⛔ false — alive |

⭑ **Strictly worse than the `ConnectOutcome` case just closed** (`7d1d6ad84`, 35 sites, three worlds).
Here it is **six** worlds, and the two most wrong are the ones that are not failures of the peer at all:
a store that **answered** with a refusal, and a **clean shutdown**.

⛔ **`TimedOut` is now reachable**, which it was not when this was written: the queue's store calls go
through the generated client method, which uses `call-by-deadline`. So *"the store is slow"* is a live
path that reports *"peer is dead"*.

### ⛔⛔ AND IT IS NINE WORLDS, NOT SIX — the `_` hides FOUR, and one contradicts a struck ruling

`ScanResponse` (`wat/query.wat:548`) has four failure variants, all swallowed by the `_` at `:593`:

| variant | what it means | reported as "peer is dead" |
|---|---|---|
| ⛔⛔ **`:Transient [err]`** | **a RETRYABLE store error** | ⛔ and `transient-means-try-again` (struck) was drawn *specifically* to make the queue **retry** these instead of dying on them. **This site dies on one, in the same file.** |
| `:Fatal [err]` | a real store failure | the only one close to true, and still not "the peer" |
| ⛔ `:RequestTooLarge [bytes cap]` | **this caller's frame exceeded the declared cap** | blamed on the store |
| ⛔ `:RequestMalformed [path expected got]` | **this caller's frame did not decode** | blamed on the store |

⭑ So the message is wrong in **seven of nine** worlds, it blames the store for **two failures the
caller caused**, and it kills the queue on an error a sibling stone made retryable. ⚠ **The "6 strings"
this stone was drawn from were the visible half**; the `_` hid the worse half.

### ⛔⛔ A CALLER-CAUSED REFUSAL IS A FIRST-CLASS FAILURE MODE, NOT A LESSER TIER

Builder, correcting this DESIGN's first draft: *"'own bugs' — they are well within scope here… we're
finding all the necessary things to make service-to-service (of any complexity) an exemplar."*

The first draft called `RequestTooLarge` / `RequestMalformed` *"our own bugs"*, which implied a lower
tier of concern. **There is no such tier.** A caller that oversizes or malforms a request is exactly the
case an exemplar must handle well, and this campaign already ruled the **receiving** half:
arc 278 Stone 2 — *"a bad caller, malicious or dumb, cannot crash anything"* — generated a request-shape
guard into every op and forced `:RequestMalformed` onto every response so the refusal is **a value the
caller cannot ignore**.

⭐ **This site is the SENDING half of that same doctrine, and it fails it.** The service does its job:
it refuses honestly and says exactly what was wrong (`bytes`/`cap`, or `path`/`expected`/`got`). The
caller then **throws that away, blames the peer, and dies.** The receiving side was made
crash-proof; the sending side discards the diagnosis it was handed.

⚠ **So their DISPOSITION is in scope too, not only their message** — and the SCORE must say what it
should be. This stone still changes no disposition (see the contract), but *"a caller must not die on a
refusal it caused, holding the very information needed to fix it"* is a finding this stone surfaces and
must not bury.

## ⛔ THE ONE CONTRACT DECISION

**Name all six worlds, carry the cause where one exists, and change no disposition — except report
`Stopped`.**

Five arms keep raising with an honest message. ⚠ **`Stopped` is different and is NOT fixed here:** it
means *the world is stopping*, which is not an error, so raising on it is wrong under any reading of the
builder's doctrine. But `every-waiter-can-bound-its-wait` put `RecvOutcome::Stopped` in **its own bucket
(325 arms)** and explicitly reserved it: *"the builder rules whether it belongs."* **Changing its
disposition at one site while 324 others stand would be a one-sided fix of a class the builder has not
ruled.** So: give it an honest message, and **report that its disposition needs a ruling.**

⭑ **Not a codemod.** `wat/fix.wat` is for a structural rewrite across many `.wat` files; this is **one
fold in one file**. The dial stone's helper was worth building for 35 sites; six arms in one place is a
hand edit, and saying so is the point — reaching for the codemod here would be ceremony.

## Out of scope = REJECTED

- **Changing whether any arm raises** (except reporting `Stopped`). Supervision is deferred.
- **`RecvOutcome::Stopped`'s 324 siblings.** The builder's ruling, off the waiter census.
- **The 258 bare `recv`s.** Explicitly next per the builder, and a separate stone.
- **Reusing `:wat::service::redial-failed!`.** It is for `ConnectOutcome`. A `RecvOutcome` helper may be
  worth extracting **later, across the 51 arms** — building it for six would fix the shape of the fix
  before the population is known.

## Blast radius

```
wat-scripts/queue/sqs.wat   one fold, six arms (:586-:615)
elsewhere                   0 expected — VERIFY
```

## Trap-doors named up front

1. ⛔ **The `_` over `ScanResponse` must name the variants it was hiding**, not merely get a better
   string. A prettier message over a still-collapsed wildcard is the defect restated.
   ⚠ **Check what `ScanResponse`'s failure variants actually are** — do not assume; `Success` is handled
   at `:591` and the rest is a wildcard, so the set is unread.
2. ⛔ **Carry the cause where the variant has one.** `Lost` and `Malformed` carry a `Failure`/cause;
   `Closed`, `TimedOut` and `Stopped` are nullary and have only the site to report. **Do not invent a
   cause for the nullary ones**, and do not drop it for the others.
3. ⚠ **This fold is inside a `defservice` `:impls` body — it runs in a FORKED CHILD.** No parent helper
   is visible here. Whatever you write must be self-contained or stdlib.
4. **The site label must survive.** *"queue: probe scan"* is the only part of today's message that is
   true; keep it.
5. **`cargo build --release` does not compile tests.**

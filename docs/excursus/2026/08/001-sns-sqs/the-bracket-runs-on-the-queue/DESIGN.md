# The bracket runs on the queue

**Step 3, and the reason the queue was promoted at all.** Excursus `001-sns-sqs`. Read first:

- `../the-queue-we-promoted-is-the-largest-panic-holder/FINDING.md` — the census, and its limits
- `../a-momentary-failure-is-not-fatal/DESIGN.md` — **drawn 2026-09-10, never struck.** The
  governing taxonomy. Bracket's placeholders name it explicitly.
- `../bracket-is-a-leaf-and-can-follow-the-queue/FINDING.md` — why the relocation was one file

Landed already: `6e6bb2464` (queue at slot 49), `0d8bada7b` (`bracket.wat` relocated below it).

## The sentence

> **A bracket's transport should be the queue we built for service-to-service, not a hand-rolled
> loop over raw `send`/`recv`.**

## ⛔ The surface does not move

`:wat::bracket::map` and `:wat::bracket::each` (`bracket.wat:902`, `:992`) keep their signature
exactly: `(map locus items work-fn & kwpairs)`. The builder's standing doctrine, given for the
reactor and applying here verbatim:

> *"whatever this means for the substrate, i do not care — the wat surface must remain unchanged;
> the substrate in rust must satisfy the current contracts under the hood."*

⭐ **And the seam already exists.** The first argument IS the transport. `:wat::spawn::Locus`
(`spawn.wat:421`) is a `defsurface` with one feature, `launch`, satisfied today by `ThreadOpts` and
`ProcessOpts` — each extended in **both** `spawn.wat` and `bracket.wat`. So this stone is **a third
Locus satisfier**, not a rewrite of the brackets layer. `(map <queue-locus> …)` is the whole
user-visible change.

## What it buys, stated as the defect it removes

`bracket.wat` carries **30** `assertion-failed!` sites (comments stripped). Ten of them are the
crusade's exact target, and five say so in their own panic string:

- ⛔ `RecvOutcome::TimedOut` → panic, message *"recv: timed out — the peer is alive and silent"*
  (5 sites: `:54 :100 :154 :222 :508`). **A momentary failure, named as momentary, killing the
  process.**
- ⛔ `RecvOutcome::Malformed` → panic, message *"…this arm is an UNMIGRATED PLACEHOLDER
  (a-momentary-failure-is-not-fatal, stone 2 replaces it with report-final)"* (same 5 sites).

The queue's surface answers both **as values**: `send`/`receive`/`ack` with a visibility timeout and
`Success` / `Transient` / `Fatal` / `RequestTooLarge` / `RequestMalformed` responses. A timeout
becomes redelivery; a malformed frame becomes a final report. Neither is a panic.

⚠ **This is not "queue is clean and bracket is dirty."** `queue.wat` holds 72 `assertion-failed!`
sites of its own — but all 72 are **inside the defservice body**, i.e. the server's internals behind
an outcome-typed wire. A caller receives a response variant, never a panic, and a server-side crash
is already carried by the crash-notice machinery hardened earlier in this excursus. Verify that
claim rather than inheriting it.

## The work

### 1. Decide the satisfier's shape, with a measurement

`Locus/launch` is built for **spawn-a-peer-and-handshake**. A queue is a *rendezvous*, not a child.
Two readings, and the stone must pick one **and say why**:

- **(a) a queue-backed `Locus`** — a new opts record satisfying `launch`, where "launch" means bind
  a queue and return a peer-shaped handle over it.
- **(b) a queue-backed runner** — leave `Locus` alone; give the bracket runner loops a queue path
  alongside the peer path.

⛔ Pick by reading `launch`'s five parameters (`ship`, `init`, `serve`, `service-forms`,
`lu-addr-kw`, `lu-mk-kw`) against what a queue can honestly supply. If (a) forces a parameter to be
a lie — a `service-forms` that is never spawned, a `lu-mk-kw` with nothing to construct — that is
the answer, and (b) wins. **Do not force the protocol; report the mismatch.**

### 2. Replace the ten ungraceful arms — do not move them

For the queue path, `TimedOut` and `Malformed` are placed by the governing DESIGN's taxonomy:

| outcome | disposition |
|---|---|
| `TimedOut` · `Stopped` · `Closed`-then-redial-ok | **RETRY** — the peer is alive |
| `Malformed` | **REPORT, FINAL** — deterministic; retrying resends the same bad bytes |
| `Lost`-then-redial-failed | **REPORT, GONE** — nothing to return |

⛔ **`Malformed` is never retried, at any site.** That is the governing DESIGN's one contract
decision, and this stone inherits it rather than re-deciding it.

Every report **names the outcome that produced it** and is **bounded by wall clock, never an attempt
count**, and says which bound it hit.

### 3. A control, judged by mutation

A test that a bracket over a queue locus **survives** a momentary fault that kills it today. The
reproducer already exists and takes 15 s — `../the-induced-failure-rate-is-measured/FINDING.md`
drives `wat-scripts/fanout/circuit.wat` with an induced fault rate. ⭐ **Show it red on the peer
path and green on the queue path**, in the same commit. A control that only passes proves nothing.

### 4. Leave the rest alone

⛔ Do not repair `bracket.wat`'s other 20 sites, do not touch `queue.wat`'s 72, do not strike stone
1b (`Failed` leaving `<S>::Reply`) — that is the substrate's own stone and is still unstruck. This
stone changes the transport of one layer and the ten arms that transport owns.

## The four questions

- **Obvious** — the queue was promoted *for* this; the locus seam already exists.
- **Simple** — ⚠ depends on row 1. A third `Locus` satisfier is simple; a second runner path is
  less so. Row 1 is a decision, not a formality — score it honestly.
- **Honest** — the ten arms are **replaced by values**, not relocated. Row 3 is what makes that
  checkable rather than asserted.
- **Good UX** — `(map locus items work-fn)` is unchanged; a momentary fault stops killing the run.

## Method notes, because this excursus has paid for them twice

- `wat/*.wat` **does not recurse** — 27 of 55 files. Use `find`.
- Strip comments before counting, and count **occurrences** (`grep -o | wc -l`), not lines.
- `:wat::deporder::verify-stdlib` is the load-order authority; `defmacro` refs are order-free.

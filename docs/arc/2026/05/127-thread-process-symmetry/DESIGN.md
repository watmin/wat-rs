# Arc 127 — Thread/process protocol symmetry — **WITHDRAWN**

**Status:** **withdrawn 2026-05-01 in writing, before any
implementation.** This DESIGN is the honest record of an
architectural rethink that was considered and overruled by the
four questions + a re-read of `docs/ZERO-MUTEX.md`. Sequential
numbering preserved; rejected proposal stays.

## What this arc proposed

A substrate-level rethink of threads to mirror processes:

- Retire `:wat::kernel::make-bounded-channel<T>` and
  `make-unbounded-channel<T>` from user-facing code. Channels
  exist ONLY as outputs of `spawn-thread` / `fork-program` /
  `spawn-program` calls.
- Threads expose three pipes by construction:
  ```
  Thread<I, O> {
    stdin  : PipeWriter<I>           ; parent → child
    stdout : PipeReader<O>           ; parent ← child
    stderr : PipeReader<ThreadDiedError>  ; auto-routed panic side-band
  }
  ```
- Symmetric inside the thread: function takes
  `(stdin :PipeReader<I>) (stdout :PipeWriter<O>) -> :unit`. Stderr
  implicit — runtime auto-routes panics.
- User code holds at most 2 channel ends per thread (parent's
  stdin, parent's stdout); never both halves of one channel
  because no primitive returns both halves to one party.

Goal: make the Pattern B Put-ack helper-verb cycle deadlock
**structurally impossible** by eliminating the primitive that
permits it.

## Why it was considered

After arc 124 surfaced 6 deadlock-class test sites, the question
"why don't processes deadlock the same way?" surfaced. Processes
have stdin / stdout / stderr — three pipes, owned by the kernel,
distributed asymmetrically (parent gets writer/reader/reader; child
gets reader/writer/writer). Stderr is invisible to user code; the
kernel auto-routes panics.

Threads currently expose `make-bounded-channel<T>` to user code,
allowing arbitrary ad-hoc channels. The deadlock surfaced because
test code created per-call channels and held both ends. The natural
question: should we restrict threads the same way processes are
restricted?

## Why it was overruled

The four questions (`feedback_four_questions`) applied to the
proposal, against a re-read of `docs/ZERO-MUTEX.md`:

**Obvious?** No. The substrate ALREADY documents the answer in
`ZERO-MUTEX.md` § "Mini-TCP via paired channels":
- Console uses **pair-by-index** via `HandlePool<T>` — each producer
  pops a `(Tx, AckRx)` pair holding ONE end of EACH of two distinct
  channels. The driver gets the corresponding `(Rx, AckTx)`.
- `Service<E,G>` and `CacheService<K,V>` use **embedded reply-tx in
  payload** — request carries the producer's ack/reply Sender as a
  field; producer holds only the corresponding Rx.
- Both shapes give 2-ends-per-role symmetry across distinct
  channels. Neither permits a single role to hold both halves of
  one channel.

The deadlock-class shape (caller binding both `ack-tx` and
`ack-rx` and passing both to a helper) is APPLICATION code
diverging from the documented discipline. The substrate has the
right primitives; the failing tests didn't use them.

**Simple?** No. The proposal would obliterate every existing
service (Console, CacheService, the canonical `service-template`),
every pipeline stage, every fan-in / fan-out pattern, every
arc-103 spawn-program proof. The trading lab's wat
30+-thread-zero-Mutex production substrate would need a complete
rewrite. To solve a problem already solved by existing patterns.

**Honest?** Foundationally no. The premise — "threads have a
stderr-equivalent gap" — is wrong. The substrate already ships
the equivalent:
- Arc 060: `Thread/join-result` returns `Result<R, ThreadDiedError>`
  — the panic side-band.
- Arc 113: cross-thread panic backtraces cascade automatically;
  any downstream `join-result` surfaces the dead thread's panic
  message.
- Arc 110: silent send/recv is illegal — every comm site MUST
  land in `match` or `option::expect`, structurally observing
  disconnect.
- Arc 114: orphaned Senders trigger `HandlePool::finish` panic
  AT WIRING TIME (before any worker starts), naming the resource.
- Arc 117: scope-deadlock prevention catches the closure-capture
  variant of "Sender outlives consumer" at compile time.

What the deadlocked caller doesn't observe is structurally
specific: it's blocked on a recv where one of the writers (the
caller's OWN clone of the same Sender) prevents EOF. That's
caught at compile time by arc 126 — at the call-site that holds
both halves of one channel. No runtime-level stderr-routing
mechanism is needed; the type-checker sees the structural shape
and rejects it.

**Good UX?** Worse than the alternative. Forcing every protocol
through a single thread-spawn-bundled pipe trio breaks legitimate
fan-in (multiple producers → one consumer), fan-out (one producer
→ multiple consumers), and pipeline composition (the
`wat/stream.wat` map / filter / reduce stdlib). Each of these uses
`make-bounded-channel` correctly today; the proposal would force
them all through synthetic spawn-thread bundles purely to enforce
"no user channel allocation."

## What the proposal forgot

Re-reading `docs/ZERO-MUTEX.md`:

> An orphaned Sender (one that never reaches a consumer) causes a
> `HandlePool::finish` panic at wiring time — in the main thread,
> before any worker starts — naming the resource. The deadlock
> that would have happened at shutdown becomes a panic at startup.
> Detectable; loud; fixable.

> The trading lab's wat (production ancestor of this interpreter)
> has run with 30+ threads, **zero Mutex**, for months of
> development and test runs. […] When a bug surfaced, it was never
> a Mutex bug. It was an ordering bug (shutdown cascade), a
> capacity bug (Kanerva's limit, now guarded), or a type mismatch
> caught by the checker.

The architecture works. The 5 deadlock-class test sites are not a
substrate failure — they're application code that diverged from
documented discipline (pair-by-index OR embedded-reply-tx, both
described in `ZERO-MUTEX.md` § "Routing acks").

## What overruled it

**Arc 126** — channel-pair deadlock prevention. A type-check-time
rule that walks AST provenance: trace each pipe-end argument back
to its `make-bounded-channel` pair-anchor; if any two args share
one anchor, fire. The existing substrate stays; the discipline
gap closes via a structural compile-time rule (sibling to arc
117's `ScopeDeadlock`).

Cost: ~200 LOC of check + diagnostic. Versus arc 127's substrate
reshape across every service + pipeline + every consumer of
`make-bounded-channel`. Two orders of magnitude smaller, more
honest, doesn't disturb a working architecture.

## What this arc preserves as durable record

- "Why don't processes deadlock?" is a useful question, but the
  answer for threads is NOT "make threads identical to processes."
  The answer is "the substrate already has the discipline; the
  compile-time check enforces it."
- The four questions kill architectural rewrites that have a
  smaller alternative.
- `ZERO-MUTEX.md` is the load-bearing doctrine doc for this class
  of question. Re-reading it before proposing a substrate reshape
  is mandatory.
- Application-code discipline violations are application-code
  fixes, not substrate reshapes.
- The substrate's empirical record (30+ threads, zero Mutex,
  zero deadlocks in production) is the existing answer to "is
  this architecture right?" — the answer is yes; refinements
  enforce the discipline; reshapes are not warranted.

## Cross-references

- `docs/ZERO-MUTEX.md` § "Mini-TCP via paired channels" + § "Routing
  acks" — the doctrine that already answers the question this arc
  asked.
- `docs/arc/2026/05/126-channel-pair-deadlock-prevention/DESIGN.md`
  — the arc that ships, two orders of magnitude smaller.
- `docs/arc/2026/04/060-join-result/INSCRIPTION.md` —
  `Thread/join-result` as the panic side-band (the
  stderr-equivalent the proposal forgot existed).
- `docs/arc/2026/04/110-kernel-comm-expect/INSCRIPTION.md` — silent
  disconnect is already a compile error.
- `docs/arc/2026/04/113-cascading-runtime-errors/INSCRIPTION.md` —
  cross-thread panic backtraces cascade automatically.
- `docs/arc/2026/04/114-spawn-as-thread/INSCRIPTION.md` —
  HandlePool's wiring-time orphan-Sender panic.
- `docs/arc/2026/04/117-scope-deadlock-prevention/INSCRIPTION.md` —
  closure-capture variant compile-time rule (sibling to arc 126).
- Memory: `feedback_four_questions.md` — the discipline that
  killed this arc.
- Memory: `feedback_proposal_process.md` — rejected proposals stay
  as honest record.

# BRIEF — the wait names its verb

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`.
Read `DESIGN-the-wait-names-its-verb.md` first — it carries the census, the rejected names, and the
clamp correction, so none of those get re-derived.

## THE WORK

The queue's `receive` takes `wait-ns <- :wat::core::i64`, where `0` means "do not wait, sweep" and a
positive value means "park up to this long". One field, two verbs. Replace it with an enum inside the
surface — `:Immediate []` / `:UpTo [d <- :wat::time::NonZeroDuration]` — and a field named `wait`.
The fork at `sqs.wat:487` becomes a `match`, so the mode is read from the constructor and never from
the number. Call sites move by **wat-fix codemod**, not by hand.

## ROOMS — read in this order

1. **`wat-scripts/scratch-pad/probe-nonzeroduration-crosses-the-wire.wat`** — **run it first.** It is
   this exact shape, already working: a request-side enum carrying a `NonZeroDuration`, in a
   `defsurface`, round-tripping at **process** locus. Copy its declaration shape. You are not
   discovering whether this works; you are moving a working shape into the queue.
2. **`wat-scripts/scratch-pad/probe-zero-at-the-boundary.wat`** — the companion: a zero payload comes
   back `RequestMalformed` and the next call on the same connection succeeds.
3. **`wat-scripts/queue/sqs.wat:36-48`** — the `defsurface` head and `:messages`. Your enum goes here,
   in the style of `SendResponse` at `:46`.
4. **`wat-scripts/queue/sqs.wat:54-58`** — `ReceiveRequest`. `wait-ns <- i64` → `wait <- :queue::Queue::Wait`.
5. **`wat-scripts/queue/sqs.wat:437-520`** — the `receive` arm. `:443` binds it; **`:487` is the
   `(<= wait 0)` fork that must become a `match`**; `:515` builds `:deadline-ns (+ start-ns wait)`,
   which becomes `(+ start-ns (:wat::time::nanoseconds d))`.
6. **`wat-scripts/queue/sqs.wat:655-745`** — the tick arm. **Read it; change only the comment.**
   `:678` keeps only waiters with `deadline-ns > now`, which is why `:737`'s clamp is a tick-rate
   floor and not a zero guard — and `arm-tick` at `:211-223` builds `(Nanosecond delay0)` from a
   computed i64, so that clamp is now a panic boundary. **Give it the WHY comment; keep the behaviour.**
7. **`wat-scripts/queue/sqs.wat:11-12`** — *"Instant/Duration on the request record is avoided —
   journal's wire-proven i64 time-ns is the precedent."* Half true now: B-pre made time types
   crossable, and `now-ns`/`visibility-ns` stay i64 for the fixture-drives-the-clock reason, which is
   still good. Rewrite it to say both. This is S21 and it is yours.
8. **`wat-scripts/queue/sqs.wat:765-870`** — `:user::do-receive` (`:783`, hardcodes `0`),
   `:user::do-receive-wait` (`:792,796`), `:user::park-receive!` (`:861,868`). Their signatures carry
   `wait-ns <- i64`; they take a `Queue::Wait` now. **Do not merge them** — that is Stone D.
9. **`wat/fix.wat`** — the codemod framework. Read its header.
10. **`wat-scripts/fixes/response-record-to-enum.wat`** and **`positional-to-kwargs.wat`** — the two
    nearest recorded shapes: a record field becoming an enum, and kwarg-value rewriting.

## SKETCH

```wat
;; in :messages, beside SendResponse
(:wat::core::defenum :queue::Queue::Wait :wat::enum::Pure
  :Immediate []
  :UpTo [d <- :wat::time::NonZeroDuration])

;; the fork, at :487 — a match, not a comparison
(:wat::core::match (:queue::Queue::ReceiveRequest/wait req)
  ((:queue::Queue::Wait::Immediate) <the existing empty-reply branch>)
  ((:queue::Queue::Wait::UpTo d)    <the existing park branch, with
                                     :deadline-ns (+ start-ns (:wat::time::nanoseconds d))>))
```

## THE CODEMOD

⛔ **`.wat` corpus migration → `wat-fix`, never hand-edits or python/sed.** Ten literal call sites
across five files; the three parameter-carrying helpers in `sqs.wat` are hand-typed.

- **Census first**, and report its count before applying: `wat --grep <fix>.wat` prints unapplied
  matches. **Count occurrences, not lines** — the finder emits one long line and `grep -c` undercounts.
- Dry-run on a `/tmp` copy and **diff it** before touching the corpus.
- Apply with every path listed:
  `printf '["pathA" "pathB" …]\n' | ./target/release/wat ./wat-scripts/fixes/<fix>.wat`
- Idempotent; commit it as the recorded migration.
- **Comments are not rewritten** (the tool walks forms). `circuit.wat:144` says *"wait-ns 250 ms is
  the idle wait"* and `probe-parked-waiters-stop.wat:4,7` discuss `wait-ns` in prose — those are a
  separate manual pass. Report them; do not leave them lying.

`sqs.wat` is **userland, not stdlib** — no BOOTSTRAP / STASH-DANCE needed.

## STOP TRIGGERS

1. **A magnitude comparison against the wait survives anywhere.** `<= 0`, `> 0`, `< 1`. That is the
   contract decision; if you cannot remove it, the design is wrong. STOP and report the site.
2. **The codemod cannot express a value-dependent flip** (`0` → `:Immediate`, `N` → `:UpTo (…)`).
   STOP and report — do not hand-edit the corpus instead.
3. **You are about to change `sqs.wat:737`'s clamp behaviour**, or any of the six `1000000` literals.
   STOP. They are correct values now, not workarounds.
4. **You are about to merge `do-receive` and `do-receive-wait`**, or touch `take-one`,
   `wait-pending`, `q-depth`, `accept!`. Stone D. STOP.
5. **The circuit's invariant moves.** `distinct=8000; dup=0` must hold. Any change is a finding —
   capture it, do not tune it away.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor,
no poll-and-stop — three riders on this arc died that way.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the exact
assertion, report.

⚠ **One floor test carries a live, known race that this stone does not fix**:
`probe_async_publish::refused_subscriber_is_retried_not_dropped`. Its mechanism is proven and
deterministic at `probe-refused-retry-self-consumes.wat` and belongs to Stone D. If it goes red, say
so and point at the reproducer — do not chase it, and do not "fix" it here.

Leave your work uncommitted. Prior comparable result for shape: `SCORE-time-crosses-the-boundary.md`.

## REPORT

- the codemod's own census count, **before** applying, and the dry-run diff
- the `match` at `:487`, and a grep showing **no** magnitude comparison survives
- the circuit: `total`, `distinct`, `dup`, and deliveries/s against ~300/s
- what you did with the prose comments the codemod cannot reach
- the rewritten `sqs.wat:11-12`
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.** My last three
  censuses were each wrong in a different way; treat mine as a hypothesis and the finder as the fact.

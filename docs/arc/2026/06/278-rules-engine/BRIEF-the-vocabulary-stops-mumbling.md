# BRIEF — the vocabulary stops mumbling

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`0afac6aa2`, tree clean. Read `DESIGN-the-vocabulary-stops-mumbling.md` first.

## THE WORK

`(:fanout::accept! t "hello")` sends `"hello|<epoch-nanos>"`. It publishes, **rewrites the caller's
payload**, and **retries unbounded on `Full`** — and the name announces none of the three. Rename it
and its sibling to say all of it, give the retry a liveness bound that reports, and sweep the rest of
the mumbling helper names. Backpressure must survive: the retry is correct, it is only silent.

## ROOMS — read in this order

1. **`wat-scripts/fanout/circuit.wat:797-802`** — `:fanout::accept!`. Six lines; the whole body is
   the rewrite. It delegates to `accept-stamped`, whose name is the honest one.
2. **`wat-scripts/topic/sns-fanout.wat:663-672`** — `:demo::accept!`. `PublishResponse::Full` →
   `nap-ms 1` → **recurse**. No bound, no report. No stamp in this one.
3. **`docs/arc/2026/06/278-rules-engine/BRIEF-278-a-liveness-bound-only-catches-a-hang.md`** —
   ⭐ **the taxonomy that governs the bound**: LIVENESS (raise; only a hang may trip it) / WINDOW
   (never raise; it *is* the scenario) / NEGATIVE ASSERTION (coupled). `accept!`'s retry is
   **LIVENESS**. Its rule applies verbatim: *"a bound raised until it never fires is not a fixed
   bound, it is a deleted one."*
4. **`wat-scripts/topic/sns-fanout.wat:582`** and **`circuit.wat:644`** — `face-start-tw` /
   `face-start`. Note the arm asymmetry: `Lost` is treated as success, `Stopped`/`Closed` assert.
   **Do not change the behaviour — write down why it is that way.**
5. **`wat-scripts/queue/sqs.wat:834-892`** — the six `do-` helpers. `do-stats:872` and `do-depth:882`
   issue the **identical** `Queue/stats` call and differ only in which fields they keep.
6. **six `nap-ms`** — `sqs.wat:962`, `sns-fanout.wat:595`, `circuit.wat:657`,
   `probe-visibility-redelivers.wat:23`, `probe-three-waiters-wake.wat:75`(±),
   `probe-parked-waiters-stop.wat:75`. Byte-identical bodies under six prefixes. **Rename in place;
   do not consolidate** — that is a promotion and the builder's ruling.
7. **`wat-scripts/fixes/wait-ns-to-wait.wat`** — the freshest recorded codemod, for the renames.

## STOP TRIGGERS

1. **You are about to shorten `accept!`'s retry, or treat it as a WINDOW.** It is correct
   backpressure; a short bound converts it into **message loss**, strictly worse than the mumble.
   Raise it so only a stall trips it. STOP.
2. **The bound can only say "gave up."** That is the empty ARM again. It must name depth, cap,
   attempts, elapsed. STOP.
3. **You are about to consolidate the six `nap-ms` into one verb.** Promotion is the builder's
   ruling (`sqs.wat:3-5`). Rename in place. STOP.
4. **You are about to merge `do-receive` / `do-receive-wait`** (S33) or change `nap-ms`'s
   outcome-swallowing (S34). Both named and cut. STOP.
5. **You are about to change `face-start-tw`'s `Lost`/`Stopped` behaviour.** Document it, do not
   alter it. STOP.
6. **The circuit's invariant moves.** `distinct=8000; dup=0`. A finding, not something to tune.

## THE CODEMOD

⛔ `.wat` corpus migration → **wat-fix**, never hand-edits or python/sed. The renames are mechanical;
the `accept!` bound and the `face-start` WHY are hand work in two files.

Census first with the finder and **report its count before applying** — count occurrences, not lines.
Dry-run on a `/tmp` copy and diff. Idempotent; commit it as the recorded migration. Comments are not
rewritten — report the prose sites rather than leaving them lying.

## HOW TO WORK

Run every build and test in the **FOREGROUND** and block on it. No `run_in_background`, no Monitor.

Floor is `scripts/floor.sh` (release). **Read the Summary line, never a piped exit code.** On any red
you did not intend: **do NOT re-run.** Copy the whole stdout+stderr block verbatim, name the arm.

⚠ S24 is live: `refused_subscriber_is_retried_not_dropped` can fail loudly with `after-drain=got`.

⚠ **Do not write `(:wat::core::None <Type>)`.** It is a phantom form — type-checks, raises
`UnknownFunction`. Bare `:wat::core::None` only. See
`docs/arc/2026/04/109-kill-std/NOTE-none-is-not-a-function.md`.

Leave your work uncommitted. Prior comparable result: `SCORE-the-instrument-fits-the-question.md`.

## REPORT

- the retry bound **forced to expire**, with what it reported
- the circuit: five runs, `total`/`distinct`/`dup`
- the codemod's own census count, before applying
- what `face-start-tw`'s `Lost`-is-ok arm turned out to be for
- the floor Summary line verbatim
- every STOP that fired
- **the honest deltas.** My censuses have been wrong six times this campaign, each differently, the
  last three all form-vs-token. The count you find is the fact.

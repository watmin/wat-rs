# BRIEF — no client call can hang

⛔ **THIS SUPERSEDES THE EARLIER BRIEF OF THE SAME NAME.** That one had the deadline raise inside
the macro so no call site would change. It is retracted — see the DESIGN's first section. **Do
not build it.**

Add `TimedOut` to `RecvOutcome`, bound the macro's receive, and add the arm across the corpus with
a recorded codemod.

## READ IN ORDER

| room | why you are there |
|---|---|
| `src/` — the `RecvOutcome` definition | the fifth variant, `TimedOut`, nullary |
| `wat/service.wat:2226-2237` | **the target.** `send-recv-form`; `:2237` is the bare `(:wat::kernel::recv c)` |
| `wat/service.wat:2238-2258` | the arms below — they gain a `TimedOut` arm here too |
| `wat/service.wat:3119-3170` | `call-by-deadline` — the bounded receive, already written and parametric. **Use it; do not re-invent it** |
| `wat/service.wat:572-578` | `:max-frame-bytes` — the optional-with-default clause `:deadline-ms` copies |
| `wat-scripts/fixes/declare-queue-drop-knobs.wat` | **the codemod exemplar you wrote last strike** — idempotent, comment-faithful, census-first |
| `wat-scripts/fixes/phantom-none-call-census.wat` | the exemplar for a **form-context** predicate (head + arm set), with both controls |

## THE CODEMOD RULE

For every `match` whose scrutinee is a `RecvOutcome` and which has **no catch-all `_` arm**,
insert:

```wat
((:wat::kernel::RecvOutcome::TimedOut) <the Lost arm's body, with a timeout-specific message>)
```

- **Mirror the `Lost` arm.** 245 of them are `assertion-failed!`; ~22 are `nil`/`Tuple`/`connect`.
  Mirroring gives the same behaviour as a vanished peer, which is defensible everywhere and loud
  where it matters.
- **A match with a catch-all needs nothing.**
- **Idempotent:** a match already carrying a `TimedOut` arm is left byte-untouched.
- **Census first**, diff it, then apply. Count **occurrences, not lines**.

## BLAST RADIUS

`src/` (one variant), `wat/service.wat`, `wat-scripts/fixes/add-timedout-arm.wat`, and the `.wat`
corpus **via the codemod only**. ⛔ **No hand-edited `.wat`.**

⚠ The stdlib is frozen at build time — rebuild before every run.

## STOP TRIGGERS

- **STOP-1** — `call-by-deadline` needs `(:wat::program::Env/peer-kind (:wat::program::env))`.
  Proven from a worker impl; **not** proven from inside a generated client method, which runs
  wherever the caller runs, including `:user::main`. If it is unavailable there, **STOP and report
  the exact error.** Do not special-case a locus.
- **STOP-2** — if the codemod cannot be made idempotent, STOP. A migration that is unsafe to
  re-run is not a recorded migration.
- **STOP-3** — if the floor reds with deadline raises, **do not raise the default.** Report which
  surfaces fired and at what elapsed time; those are the `:deadline-ms` declarations.
- **STOP-4** — do not fold `TimedOut` into `Lost`, and do not hand-edit a `.wat` file to finish
  the migration.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-the-queue-can-drop-too.md` — your own 41-hit, 11-file recorded migration from this run,
including the idempotency proof.

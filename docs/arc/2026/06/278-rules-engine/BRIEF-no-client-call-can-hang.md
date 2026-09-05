# BRIEF — no client call can hang

Bound the one unbounded receive in the `defservice` macro, so no generated client method can wait
forever. `wat/service.wat`.

Read `DESIGN-no-client-call-can-hang.md` first — especially *why the migration dissolved*.

## READ IN ORDER

| room | why you are there |
|---|---|
| `wat/service.wat:2226-2237` | **the target.** `send-recv-form` — the quasiquoted body every generated method expands to. `:2237` is the bare `(:wat::kernel::recv c)` |
| `wat/service.wat:2238-2258` | the arms below it. **They do not change.** `RecvOutcome` keeps four arms |
| `wat/service.wat:3119-3170` | `:wat::service::call-by-deadline` — the bounded receive, already written, already parametric, already used from process-locus impls. **This is the mechanism; do not re-invent it** |
| `wat/service.wat:963` | the precedent: a generated method already raises an unignorable failure |
| `wat/service.wat:572-578` | `:max-frame-bytes` — the optional-with-default clause shape `:deadline-ms` follows |
| `wat/service.wat:372-377` | why it is **not** optional-off |

## SKETCH

In `send-recv-form`, the bare receive becomes bounded, and expiry raises:

```wat
;; A client with no deadline policy gets "die loudly", never "hang forever".
;; A caller that wants to HANDLE a timeout uses :wat::service::call-by-deadline.
~r-sym (:wat::core::match (… call-by-deadline c (~op-variant-kw req) ~deadline-ms ~inert …)
         ((:wat::service::CallOutcome::Answered reply) reply)
         ((:wat::service::CallOutcome::DeadlineFired)
           (:wat::kernel::assertion-failed!
             "<surface>/<verb>: no reply within <N> ms — the peer is alive and silent"
             :wat::core::None :wat::core::None))
         …)
```

⚠ The surrounding `match ~r-sym` on `RecvOutcome` **stays exactly as it is.** Whatever shape you
choose must hand it a `RecvOutcome`, so the four arms below and all 643 call-site matches are
untouched.

## BLAST RADIUS

`wat/service.wat` only. **No `.rs`, no `sqs.wat`, no `circuit.wat`, no codemod — unless row 5
demands one, and then it is recorded under `wat-scripts/fixes/`.**

⚠ **The stdlib is frozen at build time — rebuild before every run.**

## STOP TRIGGERS

- **STOP-1** — ⛔ **the one genuinely unproven thing.** `call-by-deadline` needs
  `(:wat::program::Env/peer-kind (:wat::program::env))` for the timer's locus. It is proven from
  a *worker impl*; it is **not** proven from inside a generated client method, which runs
  wherever the caller runs — including `:user::main` at top level. If the peer-kind lookup is
  unavailable or wrong there, **STOP and report the exact error.** Do not special-case a locus.
- **STOP-2** — if bounding the receive forces the surrounding `RecvOutcome` match to change
  shape, STOP. That is the 643-site season the DESIGN rejected; the whole stone is that it does
  not happen.
- **STOP-3** — if the floor reds with deadline raises, **do not raise the default to make them
  green.** Report which surfaces fired and at what elapsed time. Those are the `:deadline-ms`
  declarations, and they are row 5's answer.
- **STOP-4** — do not add an arm to `RecvOutcome`, do not route a timeout into `Lost`, do not
  touch `call-by-deadline`'s four existing call sites.

## THE CENSUS — wat-grep + rete, not grep

Row 6 wants the true population of generated-method call sites. **Grep provably cannot produce
it**: `(:ns::Surface/verb …)` and `(:ns::Record/field …)` are the same name shape, and a raw
`(:wat::kernel::recv …)` match has the same arms.

The structural discriminator: **a `match` whose scrutinee is a `/`-headed call form and whose
arms are `:wat::kernel::RecvOutcome::` variants.** Copy `wat-scripts/fixes/phantom-none-call-census.wat`
— it is this tree's worked example of exactly this kind of head-plus-context predicate, with its
negative and positive controls.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-every-client-call-has-a-deadline.md` — the stone that built `call-by-deadline`, and whose
grading found the gap this one closes.

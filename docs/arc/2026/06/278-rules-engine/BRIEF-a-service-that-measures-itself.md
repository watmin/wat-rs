# BRIEF — A1+A2: a service that measures itself

**Stone:** `DESIGN-STONE-a-service-that-measures-itself.md` — fully ruled, nothing blocked.
**Two phases in one strike, in order.** A1 is a one-function mint; A2 is the app that needs it.

## Why this exists

**No wat service, anywhere in the substrate, has ever observed a signal.** Census, grounded: the only
`stopped?` in `wat/` is a comment; `sigusr1?`/`sigusr2?`/`sighup?` appear **zero** times. Handlers set
the flags, predicates read them, and the actor layer has never asked. This is the first consumer.

## A1 — mint `Handle/process → (Option Process)`

`start` builds a `Launched` whose `handle` is typed `:wat::kernel::Peer<Sh,Lu>` (`wat/spawn.wat:265`).
That erasure is deliberate — it is what makes `stop` locus-agnostic. But `:wat::kernel::signal` takes
`Process<I,O>`, and `Peer → Process` is a downward narrowing the checker correctly refuses.

**The concrete `Process` already exists** — `spawn-program` returns it and `start` wraps it. A1
un-erases it:

- `Some(process)` on a process locus
- `None` on a thread locus — the honest statement that a thread has no process to signal

**Do not** make `signal` take a pid to dodge this. `clone.rs:215-216` documents a pid as reuse-unsafe
for `kill()`; the bare-pid verb was four-questioned and killed on Honest. The accessor keeps the
guarantee structural.

## A2 — the app, and the sequence is the builder's

```clojure
(signal proc :sighup)     ;; → SignalOutcome::Delivered
(query  client :sighup)   ;; → true
(signal proc :user1)
(query  client :user1)    ;; → true
(signal proc :user2)
(query  client :user2)    ;; → true
(signal proc :terminate)  ;; → the child dies, no notification
;; the admin handle wakes on the lifeline
```

One `defservice` whose serve loop **measures as it serves**: a durable record with a field per
observation, updated when the loop finds a flag set, read back through **ordinary client ops** as the
test progresses.

```clojure
(:wat::core::defrecord …::Obs
  [requests <- :i64  sighup <- :bool  user1 <- :bool  user2 <- :bool])
```

**There is no admin ask.** An earlier draft delivered a final tally via `<svc>/stop`. Cut — querying
as you go proves observation *during real operation*, which a final tally cannot, and it is how a real
app reads its own flags.

**`stop` needs no state path.** A killed service delivers nothing; that is ruled and correct.
`ServiceEvent::Shutdown → nil` (`service.wat:1227`) already does the right thing — **do not touch it.**

## Read in order

1. `wat-tests/service-admin-facet.wat` — the start/connect/op exemplar, green, and it runs **both
   loci one token apart**. Copy its shape.
2. `wat-tests/process/signal-user2-and-hangup-independent.wat` — P3's deftest: how to signal and
   assert on a child's reply. Copy its facing discipline (every outcome arm faced).
3. `wat/service.wat:1215-1230` — the serve loop's `poll`/`ServiceEvent` dispatch, where the flags get
   observed and where `Shutdown → nil` lives.
4. `wat/spawn.wat:258-266` — `Launched`, the erasure A1 undoes.

## The delivery asymmetry — expect it, do not "fix" it

Grounded in the handlers: `substrate_on_stop_signal` sets the flag **and writes the wake pipe**;
`substrate_on_sigusr1` **only sets the flag**. So:

- **sighup / user1 / user2** are a bitflip observed on the **next op**. A blocked service does not wake
  for them. That is correct and is why the sequence queries after each signal.
- **terminate** wakes the blocked `poll` and the service dies.

If a user-signal query returns `false` on the first attempt because no op has run since the signal,
that is the semantics, not a bug — drive an op.

## STOPs — ship nothing, surface the gap

- **⛔ STOP-1 — never modify the runtime to measure.** The builder's rule: *"anything that measures how
  a process behaves is done hermetically."* No patched handler (outside the deliberate break), no
  harness-side flag reset, no global touched. If a measurement seems to need it, the design is wrong.
- **⛔ STOP-2 — no sleep.** The wire is the synchronisation. `mora`.
- **⛔ STOP-3 — do not push state.** The service replies when asked. An unsolicited send blocks on a
  pipe nobody drains, and blocking at teardown is the deadlock the builder forbids.
- **⛔ STOP-4 — do not touch `ServiceEvent::Shutdown → nil`.** Ruled correct.
- **⛔ STOP-5 — no `Tuple` across the wire** (degrades to `Vec`). Records round-trip **tagged** —
  proven: `#probe/Obs {:user1 7 :user2 9}`.
- **⛔ STOP-6 — no `_`-prefixed discard bindings.** The must-use gate is an exact `_` match
  (`check.rs:10926`); `_x` slips it silently. This already cost one inert probe today.
- **⛔ STOP-7 — if A1 cannot be expressed cleanly, STOP before A2.** Do not fall back to a pid-taking
  signal to get moving. Surface it.

## ★ THE DELIBERATE BREAK — the row that matters

Remove `install_substrate_signal_handlers()` at **`src/distribution/spawned_runtime.rs:51`** — the
**child's** install — rebuild, run the app's deftest, confirm it goes **RED**. Restore byte-exact,
confirm green. Report both.

**Not `distribution/mod.rs:347`.** That sits inside `run_with_args`, the CLI entry, a path nextest
never executes; breaking it leaves the test green and looks like a pass. That exact error cost a rider
a cycle today.

## Done means

- `Handle/process` returns `Some` on a process locus and `None` on a thread.
- The app's deftest runs the builder's sequence and asserts the child's own reports.
- All four handlers proven in one run, with the three user signals **discriminated** from each other.
- The break went RED and the restore went green — both reported.
- `cargo nextest run --release` **Summary line** verbatim, count arithmetic explained.
- `cargo clippy` clean.

Do not commit; do not push. Report what you built, what you measured, and every STOP you hit.

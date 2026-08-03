# BRIEF — P3: the racy signal tests become real process measurements

**Stone:** `DESIGN-STONE-process-signal-owner-to-child.md` (P2 landed `ae662ba0`).
**This is the repair.** P2 built the tool; P3 is the only stone that makes anything more reliable
than it was this morning.

## The defect being repaired

Five tests in `src/runtime.rs` mutate three **process-global** `AtomicBool` statics
(`KERNEL_SIGUSR1`/`SIGUSR2`/`SIGHUP`). Under `cargo test`'s shared-process parallelism they clobber
each other. They have survived only because nextest forks per test — a runner-dependent wall nobody
chose, left behind when arc 170's `063ab25f` deleted the per-test fork quarantine as "redundant."

**And the deeper defect: three of them do not test anything.** `reset_user_signals();
set_kernel_sigusr1(); eval("(sigusr1?)")` asserts that a setter sets and a getter gets, inside the
harness's own process. **No signal is delivered. No handler runs.** The builder's ruling:

> *"if we are going to measure a process level setting — we need a dedicated process to observe
> this … we must not modify our runtime to measure a thing … the flags are purposefully process
> global states."*

## The split — only THREE are victims; two are causes

| test (`src/runtime.rs`) | what it asserts | disposition |
|---|---|---|
| `sigusr1_query_reflects_flag_state` :31081 | flag state | **replace** with a real process measurement |
| `sigusr2_and_sighup_independent` :31095 | flag state | **replace** |
| `reset_sigusr1_flips_flag_false` :31110 | flag state | **replace** |
| `reset_sighup_returns_unit` :31121 | `Value::Unit` | **keep in place**, delete its global mutations |
| `user_signal_predicates_refuse_arguments` :31129 | `ArityMismatch` | **keep in place**, delete its `reset_user_signals()` |

The last two never assert a flag. They mutate the statics and clobber their siblings while being
structurally unable to suffer the race themselves. They do not need a process — they need to stop
touching global state. `reset_sighup_returns_unit` asserts the verb's *return shape*, which does not
depend on the flag being set first.

## The shape — a wat `deftest`, and the composition is PROVEN

Probed by the orchestrator this session (`--check`, exit 0): `deftest` + `spawn-peer` + `signal` +
`assert-eq` compose cleanly.

```clojure
(:wat::test::deftest :wat-tests::signal::<name>
  (:wat::core::let
    [child   (:wat::test::spawn-peer (:wat::spawn::process)
               (:wat::core::forms
                 (:wat::core::defn :user::main [] -> :wat::core::nil
                   ;; block on readln => provably alive and past handler install,
                   ;; then answer what this process observes
                   ...)))
     verdict (:wat::core::match (:wat::kernel::signal child :wat::kernel::Signal::User1)
               (:wat::kernel::SignalOutcome::Delivered "D")
               ((:wat::kernel::SignalOutcome::Failed _c) "F"))]
    (:wat::test::assert-eq verdict "D")))
```

**Read in order:**

1. `tests/process/signal_user1_delivers_child_observes_flag.wat` — P2's evidence fixture, green on
   disk. The spawn-signal-ask-observe round trip, working. Copy its mechanism.
2. `wat-tests/spawn/recv-budget-override.wat` — the `deftest` + `spawn-peer` + assert idiom in its
   proper home. Copy its shape.
3. `src/runtime.rs:31073-31135` — the five tests and the comment block above them explaining the
   race and its runner dependence. That comment goes when the tests it describes go.

## The three replacements

Each spawns a child, signals it, asks it what it observes, and asserts the answer. The child's reply
is the only honest evidence — a `signal` that did nothing returns the false answer.

1. **User1 delivered and observed** — child reports `(sigusr1?)` true.
2. **User2 observed, Hangup NOT observed** — the independence claim, now real: signal `User2`, child
   reports `(sigusr2?)` true **and** `(sighup?)` false **in the same reply**. The old test asserted
   this against two statics in one process; this asserts it against a process that actually received
   one signal and not the other.
3. **reset flips it false** — signal `User1`, child observes true, child calls
   `(:wat::kernel::reset-sigusr1!)`, child observes false, and reports **both** observations so the
   transition is what is asserted, not the endpoint.

Assert the reply **structure** exactly. `wat` stdio is EDN; a `contains?` on a rendered string is the
launder this repo has a lint against, and the P2 rider already tripped `no_loose_string_assert` once.

## ★ THE DELIBERATE BREAK — this is the row that matters

**R59, and this stone exists because of it.** The defect being repaired is *a signal test that passed
for weeks while no signal was ever delivered.* A green here that cannot go red repeats that sin under
a new mechanism.

**Comment out `install_substrate_signal_handlers()` (`src/distribution/mod.rs:347`), run the three
new deftests, and confirm they go RED naming the signal.** Then restore it byte-exact and confirm
green. Report both states.

P2's own row-4 break is the worked precedent: sending `Signal::User2` while the child checked
`(sigusr1?)` turned it RED, proving the test depended on *which* signal arrived. Yours must show the
same kind of dependence on the *handler existing*.

## STOP triggers — ship nothing, surface the gap

- **STOP-1 — do not touch the runtime to make a test work.** The builder's ruling. The flags stay
  process-global statics; `install_substrate_signal_handlers` stays as it is (the break is temporary
  and reverted). If a test seems to need a runtime change, the test design is wrong — STOP.
- **STOP-2 — do not use a sleep.** The child must announce or answer over the wire. `mora`: sleep is
  a guess and guesses race. Blocking in `readln` until asked is the proven pattern.
- **STOP-3 — the two cause-tests keep their subjects.** `reset_sighup_returns_unit` still asserts the
  return is `Unit`; `user_signal_predicates_refuse_arguments` still asserts `ArityMismatch`. Delete
  only the gratuitous global mutations. Do not delete or weaken the assertions.
- **STOP-4 — no `let [_ …] nil` ceremony and no `_`-prefixed discard bindings.** `main` is the work
  or a direct call. The must-use gate is an exact `_` match (`check.rs:10926`), so `_x` silently
  slips it — that defect already cost this stone one inert probe.
- **STOP-5 — if a deftest cannot express one of the three, STOP and say which.** Do not fall back to
  a Rust test to get a green. An honest "this one needs the co-located pair shape and here is why" is
  a correct outcome; a quiet shape-switch is not.

## Disposition of P2's evidence fixtures

- `signal_user1_delivers_child_observes_flag.{wat,rs}` — its own header says it is superseded once
  P3 lands its own coverage. If your deftest #1 genuinely covers it, **delete both files** and say so.
  If it does not, keep them and say what the gap is.
- `signal_kill_produces_close_outcome_signaled.{wat,rs}` — **KEEP.** P3 cannot replace it: there is
  no wat door into `close` at all (`resolve/registration.rs` refuses any user-privilege source
  defining under `:wat::`, so no wat fixture can reach it). Do not attempt to route around that.

## Done means

- The three flag tests are **gone** from `src/runtime.rs`, replaced by deftests that measure a real
  process.
- The two cause-tests remain, with their assertions intact and their global mutations gone.
- The stale comment block above them (`:31073-31092`) is gone or rewritten to describe what is
  actually there.
- The deliberate break went RED and the restore went green — both reported.
- `cargo nextest run --release` **Summary line** verbatim, and the count arithmetic explained
  (three deleted, three added, possibly two more deleted if the P2 fixture retires).
- `cargo clippy` clean.

Do not commit; do not push. Report what you built, what you measured, and every STOP you hit.

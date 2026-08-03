# BRIEF — the signal tests cannot race, under any runner

**The builder's ruling, and it is the acceptance bar:**

> *"It is not acceptable that this triggers in `cargo test`. We use nextest — so be it — **the tests
> must never have a race, period.**"*

Not "green under our runner." **Race-free by construction.**

## You are a rider, not the orchestrator

**Ending your turn ENDS you.** Nothing wakes you; no notification is coming. Run every command in the
FOREGROUND and block on it. Your turn ends when the numbers are in your hands.

## The defect

Signal flags are process-global `AtomicBool` statics — correctly; signals *are* per-process. The tests
in `src/runtime.rs`'s `mod tests` mutate that global directly:

```rust
#[test]
fn sigusr2_and_sighup_independent() {
    reset_user_signals();      // clears PROCESS-GLOBAL state
    set_kernel_sigusr2();
    … expect sigusr2 true …    // another test's reset() lands here → false
}
```

Under a threaded runner they clobber each other. Observed on at least three:
`sigusr1_query_reflects_flag_state`, `sigusr2_and_sighup_independent`, `reset_sigusr1_flips_flag_false`.
All pass under `--test-threads=1`; all pass under nextest, which forks per test.

## ★ THE HISTORY — this was fixed, then deleted, and read the deletion before you re-add anything

| | |
|---|---|
| `8a554cd6` (Apr 21) | Quarantine BUILT. Five signal tests each in their own process. Flake rate before: ~1 in 2–3 runs. Verified over three consecutive clean runs. |
| `d74c2dff` | Refactored to `libc::fork` — quarantine **maintained**, couplings reduced. |
| `063ab25f` (arc 170) | **ANNIHILATED** — *"the runner already forks per test."* Five screamers in `src/runtime.rs` "unwrapped to run inline; **no migration, no classification**." |

That reasoning is true for nextest and false for `cargo test`. It traded an **explicit unconditional**
wall for an **implicit runner-dependent** one, and recorded it as removing redundancy. Nothing screamed,
because nextest's isolation silently took over the job.

**Do not simply revert `063ab25f`.** Its instinct was right — hand-rolled per-test forking IS heavy
scaffolding. What it got wrong is that the isolation was load-bearing. The fix must be a wall that
**cannot be mistaken for scaffolding**, which is the whole design constraint below.

## The invariant to establish — mechanism is yours to ground

1. **No race under ANY runner.** `cargo test` threaded, nextest, `--test-threads=1` — all correct.
   Not by luck, not by isolation the runner happens to provide.
2. **Production signatures UNCHANGED.** `set_kernel_sigusr1` / `set_kernel_sigusr2` /
   `set_kernel_sighup` have real production callers — `src/compose.rs:58,61,64` (the signal handlers)
   and `src/process/child.rs:46,50`. A test-only guard parameter on a production function is a
   signature that lies about why it exists. **Do not touch them.**
   (`reset_user_signals`, `runtime.rs:144`, is a different case — every caller is a test. Grounded.)
3. **★ DELETING THE GUARD MUST FAIL THE BUILD, not a test.** This is the requirement that makes it a
   wall rather than a convention, and it is the direct answer to `063ab25f`. If the guard can be
   removed and everything still compiles, the next cleanup removes it again and we are back here.
4. **A NEW signal test must not be able to get it wrong.** If a sixth test can be written that races,
   the mistake is still representable and you have built rung two, not rung three.

**The shape the builder ratified:** move the signal tests out of `mod tests` in `runtime.rs` into their
own module that can only reach a guarded surface — `with_signal_state(|s| …)` or similar — while the raw
setters stay bare for production and unreachable from the tests.

**⛔ STOP-1 — if perfect unreachability is not achievable in-crate, SAY SO rather than settling
quietly.** Rust privacy may not give it to you: a `#[cfg(test)] mod` sees its parent's private items,
and `pub(crate)` is visible to any in-crate test module. If the strongest honest wall is weaker than
requirement 4, **report exactly which requirement you could not meet and what the residual hole is** —
with the `file:line` that proves it. A named weaker wall is worth more than a silent one, and I would
rather rule on the gap than discover it in three months.

## The four questions, already run — do not re-litigate

- **Restoring the per-test fork** fails *Simple* (braids isolation into the assertion) and *Good UX*
  (a sixth test written without it races again).
- **A bare `static Mutex<()>` each test takes** fails *Good UX* for the same forgettability, and fails
  requirement 3 — removing it compiles fine.
- **The guarded surface** was the only shape scoring four YESes, on requirement 3 specifically.

## Prove it

- **The race is gone under the runner that exposed it:** `cargo test --release --lib -p wat` run
  **five consecutive times**, all green. The historical flake rate was ~1 in 2–3, so five clean runs is
  a real signal rather than one lucky pass. Report all five.
- **`--test-threads=1`** green, and **nextest** still green.
- **The tests still test the thing.** They must still exercise the real wat surface —
  `(:wat::kernel::sigusr2?)` reading the real static — not a mock. An isolated test that no longer
  touches the mechanism is worse than a flaky one that does.

## Gates — foreground, every result line verbatim

```
cargo build --release --all-targets            # exit 0, ZERO warnings
cargo clippy --release --all-targets           # likewise
cargo test --release --lib -p wat              # ×5, all green — the load-bearing gate here
cargo test --release --lib -p wat -- --test-threads=1
cargo test --release --test lint
```

**Do NOT run `cargo nextest run`** — the orchestrator weighs the floor centrally.

## Do not

Do not commit, push, stash, or revert anything you did not write. Do not `#[ignore]` a signal test —
that is deferral wearing a fix's clothes, and this arc has the receipts. Do not change a production
signature. Do not weaken an assertion to make a test stable.

# EXPECTATIONS — P3: the racy signal tests become real process measurements

Written **before** the strike. Baseline, from my own re-run at P2's weigh:
**`4342 tests run: 4342 passed (1 slow), 262 skipped`.**

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the three flag tests are GONE | `grep -n 'fn sigusr1_query_reflects_flag_state\|fn sigusr2_and_sighup_independent\|fn reset_sigusr1_flips_flag_false' src/runtime.rs` | zero hits |
| 2 | three deftests replace them | the new `.wat` files | each spawns a child, signals it, and asserts the CHILD's reported observation |
| 3 | **the child observes User1** | run deftest #1 | pass, and the assertion is on the child's reply |
| 4 | **independence is real** | run deftest #2 | `(sigusr2?)` true AND `(sighup?)` false, both in one reply from a process that received exactly one signal |
| 5 | **reset is a TRANSITION** | run deftest #3 | both observations reported (true→false), so what is asserted is the flip, not an endpoint |
| 6 | ★ **THE BREAK GOES RED** | comment out `install_substrate_signal_handlers()` (`distribution/mod.rs:347`), run the three | **all three FAIL**, naming the signal. This is the row that matters |
| 7 | the restore goes green | restore byte-exact, re-run | all three pass |
| 8 | cause-tests keep their subjects | read them | `reset_sighup_returns_unit` still asserts `Unit`; `user_signal_predicates_refuse_arguments` still asserts `ArityMismatch` |
| 9 | cause-tests stop touching globals | `grep -n 'reset_user_signals\|set_kernel_sig' src/runtime.rs` | only the definitions remain — zero calls from those two tests |
| 10 | no sleeps | `grep -in 'sleep' <new files>` | zero |
| 11 | no `_`-prefixed discards, no ceremony main | read the new files | none |
| 12 | floor | `cargo nextest run --release` | **Summary line**, zero failures, and the count arithmetic EXPLAINED |
| 13 | clippy | `cargo clippy --release --all-targets` | clean |

## Count arithmetic — predicted, and a deviation must be explained

Baseline 4342. Minus 3 deleted Rust tests. Plus 3 new deftests. **Net 4342** — unless the P2 User1
fixture retires, which would make it **4341**. Any other number is a finding, not a rounding.

## Independent prediction

**35–60 minutes.** Three deftests over a proven composition, two one-line deletions, three test
removals. The break-and-restore is the only fiddly part.

**Time-box: 2 hours.**

## Trap doors — named in advance

- **Row 6 is the whole strike.** If the break does not turn all three red, the deftests are measuring
  something other than signal delivery and the strike has failed regardless of row 12. This is
  precisely the R59 shape: the tests being replaced passed for weeks while nothing was delivered.
- **A deftest that asserts on the PARENT's `SignalOutcome::Delivered` alone proves nothing about the
  child.** `Delivered` means the kernel accepted the signal, not that a handler ran. The assertion
  must be on the child's reply. A rider could satisfy rows 1–2 and still measure nothing.
- **Row 4 is the one most likely to be faked accidentally.** Asserting `(sighup?)` false is trivially
  true in a fresh process that was never signalled at all. It only means something *alongside*
  `(sigusr2?)` true **in the same reply from the same child** — that is what makes it independence
  rather than a fresh-process tautology.
- **The count moving unexpectedly** means a test was silently dropped or a deftest did not register.
- **A shape-switch to Rust tests** to get a green satisfies the letter and destroys the point.

## How this will be scored

By my own re-run of every row. Rows 3–7 are load-bearing; **row 6 is the strike**. A green row 12
with a silent row 6 is the exact failure this stone was opened to repair, wearing new clothes.

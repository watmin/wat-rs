# EXPECTATIONS — a scope that releases only on the happy path is not a scope

> Written **before** the strike. Scored against the orchestrator's own re-run, never the
> executor's report.

## The scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the control is green before anything | `cargo nextest run --release -E 'test(scoped_work_with_network_releases_the_lease_it_takes)'` | **1 passed** — the normal-return row still holds at HEAD |
| 2 | both probes are RED before the change | `cargo nextest run --release -E 'test(scoped_work_with_network_releases_the_lease_when_the_body)'` | **2 failed**, `arm_lease.rs:414` and `:442`, each `table grew 0 -> 1` |
| 3 | both probes GREEN after | same | **2 passed** |
| 4 | `with-overlay` inherits the cure | a third probe, same shape, driving `with-overlay` with an unwinding body | **1 passed** — written by the rider, RED before the fix if run against a stash of it |
| 5 | the control still green after | as row 1 | **1 passed** — the fix did not change the normal-return lifetime |
| 6 | no release call survives in the wat form | `grep -n 'release-session' wat/rete/syntax.wat` | **no hit inside `with-network`/`with-overlay`**; the `do` is gone, not supplemented |
| 7 | exactly one adopt site | `grep -rn 'adopt-session-lease' wat/` | **exactly 1** |
| 8 | the guard adopts, never acquires | read the constructor | no `rete_arm_intern` call on the guard path |
| 9 | blast radius | `git diff --stat` | only the files DESIGN names. Nothing else |
| 10 | the whole rete surface | `cargo nextest run --release -E 'binary_id(wat::rete)'` | all green |
| 11 | the floor | `./scripts/floor.sh`, read the Summary from the captured log | **5,191 / 5,191** (5,188 + the two probes + the `with-overlay` probe), 21 skipped, exit 0 |
| 12 | clippy | `cargo clippy --release --workspace --all-targets -- -D warnings` | silent, exit 0 |

## The mutation proof — one per arm, and the arms are named

Row 2 → row 3 proves the **wat-error** path and the **host-panic** path separately, because they
are separate mechanisms and a single test could not have proven both — the first draft of this
probe demonstrated exactly that, twice (a panic that blew past its own assertion; then an arm 2
that never ran because arm 1 failed first).

The report must state, per arm: **proven** (driven, red→green), **reachable but not driven**, or
**not reachable, and why**. An unreached arm named as unreached is a pass. **An unreached arm not
mentioned is a fail.**

One further mutation, cheap and worth it: **revert `try_with` to `with` and say what happens.** If
nothing observable changes, that is a *coverage* finding (nothing in the suite lets a guard reach
thread teardown), not a licence to drop the change — report it as such.

## Runtime prediction

50–70 minutes. Four or five release builds at ~2m40s each (the guard, the wat edit — which needs
its own rebuild to take effect at all, the `with-overlay` probe, at least one mutation, likely one
fix-up), one floor at ~370s.

## What would make this strike a failure even if every test passes

**A second release call.** If `with-network` ends up releasing in the happy path *and* relying on
the guard for the unwind path, the lease is released twice on every normal return and the design's
one contract decision has been inverted while the tests stay green (over-release is a silent no-op
on a missing id — it will not show up as a failure, only as a rebuild). The `do` must be **gone**.

The second failure shape: **a guard that acquires instead of adopting.** Count goes to 2, the guard
releases 1, `compile-all`'s lease is held forever — and the probes go green, because the table row
count returns to its starting value only if the release actually reaches zero. Row 8 exists to
catch this by reading, since row 3 alone would not.

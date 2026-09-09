# EXPECTATIONS — thread-owning `ARM_BUILDS`

| what | command | expected |
|---|---|---|
| no atomic survives | `grep -c 'AtomicUsize' src/rete/kernel/arm.rs` | **0** |
| it is a `Cell`, not an atomic in a TLS | `grep -A2 'thread_local' src/rete/kernel/arm.rs \| grep -c 'Cell<usize>'` | ≥1 |
| production untouched | `git diff -U0 src/rete/kernel/arm.rs \| grep '^[+-]' \| grep -v cfg\(test\)` | only the declaration, its rune, and the write line |
| the rune is recategorised | `grep -c 'rune:sequi(ambient-context)' src/rete/kernel/arm.rs` | **2** — `ARM_TABLE`'s and the counter's |
| no `performance-counter` claim left on it | `grep -c 'rune:sequi(performance-counter)' src/rete/kernel/arm.rs` | **0** |
| rune vocabulary still closed | `cargo nextest run --release -E 'test(no_unknown_sequi_rune)'` | green |
| the five tests still pass | `cargo nextest run --release -E 'test(arm_lease)'` | green, unchanged assertions |
| the cost reads still work | `cargo nextest run --release -E 'test(cascade_cost)'` | green |
| **the positive proof** | the new two-thread probe | green — each thread observes exactly its own build count |
| ⛔ **MUTATION — the probe can fail** | revert `ARM_BUILDS` to the process-global `AtomicUsize`, keep the probe, re-run it | **RED.** Restore ⇒ green. **This is the whole strike; if the probe cannot tell the two apart it proves nothing** |
| floor | `./scripts/floor.sh` | 5484 + the new probe, **0 fail** |
| clippy | `cargo clippy --all-targets --release -- -D warnings` | rc=0 |

## Runtime prediction

40–60 min, two floors dominating.

## Trap doors

- **An `AtomicUsize` inside the `thread_local!`.** The pinned contract decision. It pays for
  synchronisation that can no longer be contended and tells the next reader cross-thread sharing is
  still possible.
- **A probe that only asserts "the count went up".** That passes under both designs. It must assert
  each thread sees **its own** count and not the other's — the discriminating claim.
- **Assuming the five tests' assertions need changing.** They should not. If they do, STOP-1.
- **Believing the 22-site count.** It is mine, taken once: 19 in `arm_lease.rs`, 3 in
  `cascade_cost.rs`. Re-derive with `grep -rn 'ARM_BUILDS' src/` and report the delta — handed-down
  counts in this arc have been wrong twenty-one times.

# EXPECTATIONS — C0b.1b (written before the strike; goalposts fixed)

## Mode prediction

- **Mode A — clean ship (~55%).** `ready()` mirrors `select()`; the 2-arg eval arm registers
  listener+peers, peeks, recvs the ready peer, builds the `SelectEvent` enum; infer mirrors
  `infer_accept_prime`. Probe → 24. ~60–110 min. Bigger than a typical strike (4 files).
- **Mode B — small gap (~35%).** Likely shapes: (i) `crossbeam Select::ready()` borrow vs the
  fresh-per-call build needs a slightly different shape (the `selected_op` lifetime); (ii) the
  `Value::Enum` construction for the stdlib defenum needs the exact `type_path`/registration; (iii)
  the parametric `SelectEvent<O>` scheme fights inference and needs a looser shape that still makes
  the probe green. Surfaces; I decide; "use the correct shape, ship, note the delta."
- **Mode C — STOP fires (~10%).** `ready()` genuinely can't peek-without-consume in this Select
  shape, OR the stdlib defenum can't be constructed from eval — a real design fork for the Inquisitor.

## Scorecard (Inquisitor re-runs each independently)

| # | what | command | expected |
|---|---|---|---|
| 1 | grow/serve/shrink loop | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | `1 passed` (returns 24) |
| 2 | C0b.1 connection intact | `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` | `1 passed` |
| 3 | structured-peer-death intact | `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1` | `1 passed` |
| 4 | no new nursery reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | only the known reds (4 baseline; the 2 structured-peer-death probes green), zero NEW |
| 5 | wat-tests unbroken | `cargo test --release --test test 2>&1 \| tail -3` | 242/1 (test_run_string_entry_direct pre-existing) |
| 6 | clean build + clippy | `cargo build --release` ; `cargo clippy` (touched files) | no errors; no new warnings on new code |

Runtime: 60–110 min. If under 30, that's over-specification data.

## Trap-doors (named so they can't surprise the SCORE)

- **`ready()` spurious wakeup.** crossbeam `ready()` can fire then `try_recv` returns `Empty`. The
  eval must loop back to `ready()` on `Empty` — not return a bogus event. If a probe flakes, suspect
  this.
- **Listener index offset.** The listener is user-index 0; peers are 1..N. `:Message`/`:Closed`
  `idx` must be `k-1` (the index into the *peers* vector), or the loop's `nth`/`remove-at` hit the
  wrong slot. The probe's round-trip (r1+r2=24) catches an off-by-one.
- **The 1-arg path must not regress.** `eval_peer_select_prime` branches on arity; the 1-arg arm
  (brackets) stays byte-identical. If any bracket probe goes red, the branch leaked.
- **`:Lost` is not emitted at thread tier.** It exists in the enum; the thread eval never builds it.
  Do not write thread code that synthesizes `:Lost` (it would be untestable + wrong-tier).

## What "done" means

Probe → 24; C0b.1 + both structured-peer-death probes intact; nursery no-new-red; wat-tests 242/1;
build+clippy clean. The SCORE names the `ready()` signature, the `SelectEvent` construction path,
and any relaxed type-shape. No commit by the Shadowdancer — the Inquisitor weighs against its own
re-run, then commits.

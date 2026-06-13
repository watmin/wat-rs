# EXPECTATIONS — C0b.1b (3-input, deadlock-free; written before the strike)

> Supersedes PEEK (aborted: re-grew the channel surface + hung) and the 2-arg fold (no termination
> path → gdb-proven deadlock). This is the 3-input form: `select'` watches the self-peer too, so
> the owner's RAII drain is the shutdown wake. Reuses the existing comms `select()` — no
> `comms/thread.rs` change.

## Mode prediction

- **Mode A — clean ship (~70%).** Factor `wrap_connect_request`; the 3-arg eval registers
  self/listener/clients, `select()`s, maps index 0→`:Shutdown`, 1→`:Connection`, k≥2→`:Message`/
  `:Closed`; infer returns `SelectEvent<I,O>`. Probe → 24 AND terminates. ~50–90 min (3 files).
- **Mode B — small gap (~25%).** `Value::Enum` construction detail; the 2-param `SelectEvent<I,O>`
  scheme vs inference; the `wrap_connect_request` borrow. Surfaces; I decide.
- **Mode C — STOP fires (~5%).** Helper won't factor, or the defenum can't be constructed at eval.

## Scorecard (Inquisitor re-runs each independently)

| # | what | command | expected |
|---|---|---|---|
| 1 | grow/serve/shrink + **clean termination on drop** | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | `1 passed` (returns 24; **does not hang**) |
| 2 | C0b.1 + accept' factor | `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` | `1 passed` |
| 3 | structured-peer-death | `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1` | `1 passed` |
| 4 | no new nursery reds (and no hang) | `cargo test --release -p wat --test nursery -- --test-threads=1` | 4 baseline reds + the 2 structured-peer-death probes green; zero NEW; **completes (no hang)** |
| 5 | wat-tests | `cargo test --release --test test 2>&1 \| tail -3` | 242/1 |
| 6 | build + clippy | `cargo build --release` ; `cargo clippy` | clean |

## Trap-doors (named so they can't surprise the SCORE)

- **A HANG IS THE BUG.** If the probe (or nursery) hangs, the self-peer `:Shutdown` wiring is wrong
  — the owner's drop didn't wake `select'`. That is the exact deadlock this stone annihilates; it's
  a STOP/finding, never a "flaky, re-run." (Interrogate with gdb `thread apply all bt`, don't guess.)
- **Index offsets.** self-peer = index 0 (`:Shutdown`), listener = index 1 (`:Connection`), clients
  = index 2..N (`:Message`/`:Closed` carry `idx = k-2`). An off-by-N hits the wrong client slot →
  the round-trip (24) catches it.
- **Index 0 ignores the result.** The self-peer firing = owner-drop = `:Shutdown` regardless of
  `Ok`/`Err` (the owner never sends ops on the supervisor link; the drain produces `Err(Disconnected)`).
- **`wrap_connect_request` reuse, not duplication** — one helper, two callers (gate 2 proves `accept'`
  still works after the factor).
- **No `comms/thread.rs` change.** If the SCORE shows a comms edit (a `ready`/`try_recv`), the
  Shadowdancer drifted back to the aborted PEEK — reject it.
- **1-arg path byte-identical** (brackets). The 3-arg form is additive.

## What "done" means

Probe → 24 and **terminates on handle-drop** (no Stop op, no hang); C0b.1 + both structured-peer-death
probes intact; nursery no-new-red and completes; wat-tests 242/1; build+clippy clean; one
`wrap_connect_request` helper, two callers; zero comms change. No commit by the Shadowdancer.

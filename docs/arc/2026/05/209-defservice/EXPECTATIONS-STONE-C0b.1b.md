# EXPECTATIONS — C0b.1b FOLD (written before the strike; goalposts fixed)

> Supersedes the aborted PEEK approach (`ready()`/`try_recv` re-grew the channel surface and hung).
> FOLD reuses the existing comms `select()` — no `comms/thread.rs` change.

## Mode prediction

- **Mode A — clean ship (~70%).** Factor `wrap_connect_request` out of `eval_accept_prime`; the
  2-arg eval arm registers listener+peers, `select()`s, branches on index (0→wrap→`:Connection`,
  k>0→`:Message`/`:Closed`), builds the `SelectEvent` enum; infer mirrors `infer_accept_prime` +
  returns `SelectEvent<I,O>`. Probe → 24. ~50–90 min (3 files; no comms change).
- **Mode B — small gap (~25%).** Likely: (i) the `Value::Enum` construction needs the exact
  `type_path`/registration; (ii) the 2-param `SelectEvent<I,O>` scheme fights inference (the
  `:Connection` peer needs both I and O) and needs a careful shape; (iii) the `wrap_connect_request`
  factor has a borrow/ownership wrinkle vs `eval_accept_prime`. Surfaces; I decide.
- **Mode C — STOP fires (~5%).** `wrap_connect_request` can't be factored cleanly, OR the stdlib
  defenum can't be constructed from eval — a real fork for the Inquisitor.

## Scorecard (Inquisitor re-runs each independently)

| # | what | command | expected |
|---|---|---|---|
| 1 | grow/serve/shrink loop | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | `1 passed` (returns 24) |
| 2 | C0b.1 connection + accept' factor intact | `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` | `1 passed` |
| 3 | structured-peer-death intact | `cargo test --release -p wat --test nursery probe_arc209_structured_peer_death -- --test-threads=1` | `1 passed` |
| 4 | no new nursery reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | only the 4 baseline reds + the 2 structured-peer-death probes green; zero NEW |
| 5 | wat-tests unbroken | `cargo test --release --test test 2>&1 \| tail -3` | 242/1 (test_run_string_entry_direct pre-existing) |
| 6 | clean build + clippy | `cargo build --release` ; `cargo clippy` (touched files) | no errors; no new warnings on new code |

Runtime: 50–90 min. If under 25, that's over-specification data.

## Trap-doors (named so they can't surprise the SCORE)

- **Listener index offset.** The listener is registration index 0; peers are 1..N. `:Message`/
  `:Closed` `idx` must be `k-1` (the index into the *peers* vector), or the loop's `nth`/`remove-at`
  hit the wrong slot. The round-trip (r1+r2=24) catches an off-by-one.
- **`wrap_connect_request` reuse, not duplication.** `accept'` must end up = `recv` + the helper, and
  `select'` calls the SAME helper. If the wrap logic gets copy-pasted into select', that's a
  `solvere` duplication — the SCORE must show one helper, two callers. (Gate row 2 proves `accept'`
  still works after the factor.)
- **`:Lost` is not emitted at thread tier.** It exists in the enum for the remote tier; the thread
  eval never builds it. Do not write thread code that synthesizes `:Lost`.
- **The 1-arg path must not regress.** `eval_peer_select_prime` branches on arity; the 1-arg arm
  (brackets) stays byte-identical. If any bracket probe goes red, the branch leaked.
- **No `comms/thread.rs` change.** FOLD reuses `select()`; if the SCORE shows a comms edit, the
  Shadowdancer drifted back toward peek — reject it.

## What "done" means

Probe → 24; C0b.1 + both structured-peer-death probes intact; nursery no-new-red; wat-tests 242/1;
build+clippy clean; one `wrap_connect_request` helper with two callers; zero comms change. The SCORE
names the `SelectEvent` construction path + any relaxed type-shape. No commit by the Shadowdancer —
the Inquisitor weighs against its own re-run, then commits.

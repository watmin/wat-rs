# EXPECTATIONS — Stone C0b.2c (written before the strike)

The independent scorecard. The Inquisitor re-runs every row against its own build before crediting.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the gate round-trips | `cargo test --release -p wat --test nursery process_listener_connect_accept_round_trips_over_abstract_uds -- --test-threads=1` | `1 passed` (reply == 15) |
| 2 | C0b.2b socket peer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b2b_socket_peer -- --test-threads=1` | `1 passed` |
| 3 | C0b.1b select multiplexer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | `1 passed` (grow/serve/shrink + clean termination) |
| 4 | C0b.1 thread connection intact | `cargo test --release -p wat --test nursery probe_arc209_c0b1_thread_connection -- --test-threads=1` | `1 passed` |
| 5 | C0b.2a probe superseded (Test 1 updated in-strike) | `cargo test --release -p wat --test nursery probe_arc209_c0b2a -- --test-threads=1` | `2 passed` — Test 1 rewritten to `listener_with_process_host_now_type_checks` (process host VALID after C0b.2c); Test 2 (thread host valid) unchanged |
| 6 | `socket_pair` unit test intact (DRY refactor safe) | `cargo test --release -p wat --lib comms::process -- --test-threads=1` | all pass |
| 7 | full nursery, no NEW reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `≈903 passed / 4 failed` — the FOUR known baseline reds ONLY (arc-255 reflection ×2 + undefined-builtin ×2); c0b2c now GREEN (was the 5th red) |
| 8 | build clean | `cargo build --release` | no errors |

## Runtime prediction

8–18 min. The thread tier is the template; C0b.2b proved the socket peer + the io_uring-over-socket
wire; the OS sequence is proven by the UDS spike. The work is mechanical mirroring across four files.

## Trap-doors named

- **STOP-5 note:** the C0b.2a probe (#5) asserts a `(process)` host is rejected — but that probe
  binds the result to a thread-typed expectation. Read it: after C0b.2c, `(listener' (process) …)`
  no longer errors, so if the C0b.2a probe asserted "process host → check error" unconditionally it
  would now FAIL. **Re-read `probe_arc209_c0b2a_listener_host_thread_only.rs` before crediting #5.**
  If C0b.2c legitimately makes a process host VALID, the C0b.2a probe's premise is superseded and
  must be updated in THIS strike (the honest delta) — not left red. The Inquisitor decides at weigh
  time; flag it explicitly in the SCORE.
- `UnixStream` → `OwnedFd` conversion (`From<UnixStream>`) — confirm it compiles (STOP-2).
- `make_rust_opaque` `Send + Sync` bound on `UnixListener`/`String` (STOP-4) — both satisfy it;
  if not, report rather than wrap.
- abstract-name uniqueness under repeated `listener'` calls — pid + AtomicU64 counter; collisions
  only if two binds race the same name (they can't — the counter is monotonic per process).

## Out of scope (must NOT appear in the diff)

`select'` 3-arg process arm; `SO_PEERCRED`; well-known-name input; `(remote)`/AF_INET; any change to
`project_peer_io` or the thread-tier code paths.

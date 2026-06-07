# EXPECTATIONS — Stone 214.4.5-fix (process fd preservation)

## Independent prediction (orchestrator, pre-spawn)

- **Runtime band:** 15–25 min Mode A. Mechanical multi-file fix with an exact
  on-disk disconfirming probe + a passing reference (4.4) to mirror. No design
  ambiguity remains. 2× cap = **50 min** → wakeup scheduled.
- **Likeliest surprise:** the io_uring ring fd surviving the sweep but the COW
  ring misbehaving across fork for the apply-loop. Mitigation: the probes are
  single-round-trip-per-child (same shape as the passing 4.4 test), so parity
  should hold. If a multi-op ring issue surfaces, that is a HONEST finding to
  surface, not to paper over (it would mean the child must rebuild its ring
  post-fork — a follow-on, not this stone's claim).
- **Second likeliest:** `init_shutdown_signal_with_inputs` reopening onto a
  preserved fd number. The brief calls this out for explicit verification.

## Scorecard (orchestrator scores against own re-run)

| # | Claim | How verified |
|---|---|---|
| 1 | `Sender`/`Receiver` expose complete owned-fd accessors | read the two new fns; `Receiver` returns read_fd + ring fd |
| 2 | `close_inherited_fds_above_stdio` honors the full skip-list | `skip[0]`-only logic gone; multi-range sweep reads correctly; unit reasoning |
| 3 | `child_post_fork_init_preserving` exists; bare init delegates with `&[]` | one implementation, no duplication |
| 4 | `:process` child passes its comms fds to the preserving init | read spawn.rs child closure |
| 5 | **Both `:process` probes PASS single-threaded** | `setsid timeout 180 cargo test --release --test comms spawn_program_prime_process -- --ignored --test-threads=1` → 2 passed (orchestrator re-runs) |
| 6 | 4.4 reference still green | `peer_process_round_trip -- --ignored` → 1 passed |
| 7 | Library band green | `cargo test --release --lib -p wat` ~940/0/1 |
| 8 | clippy 0 in touched files; both warded-home stamps drift-checked | `cargo clippy --release`; read module docs |
| 9 | Tree dirty (sonnet did NOT commit) | `git status` |

Load-bearing rows: **5** (the fix works) + **2** (the class, not just the site).
Both re-run by the orchestrator independently before scoring.

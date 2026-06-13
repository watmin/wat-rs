# EXPECTATIONS — Stone C0b.3a-0 (written before the strike)

The independent scorecard. The Inquisitor re-runs every row against its own build before crediting.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | the gate echoes over the self-peer | `cargo test --release -p wat --test probe_arc209_c0b3a0_self_peer` | `1 passed` (echo 5→105) |
| 2 | c0b2c process connection intact | `cargo test --release -p wat --test nursery probe_arc209_c0b2c -- --test-threads=1` | `1 passed` |
| 3 | c0b2b socket peer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b2b_socket_peer -- --test-threads=1` | `1 passed` |
| 4 | c0b1b select multiplexer intact | `cargo test --release -p wat --test nursery probe_arc209_c0b1b_select_listener -- --test-threads=1` | `1 passed` |
| 5 | hermetic process round-trip intact (forms-child seam unbroken) | `cargo test --release -p wat --test wat_hermetic_round_trip` | all pass |
| 6 | program-env verbs intact (SELF_PEER mirror didn't disturb PROGRAM_ENV) | `cargo test --release -p wat --test probe_arc211_program_env_ambient` | all pass |
| 7 | full nursery, no NEW reds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` — the 4 known baseline reds ONLY (arc-255 reflection ×2 + undefined-builtin ×2) |
| 8 | build clean | `cargo build --release` | no errors |

## Runtime prediction

8–16 min. The verb mirrors `eval_program_env`/`infer_socket_pair_prime`; the install mirrors
`install_program_env`; the split-fd helper mirrors `sender_receiver_from_fd`. Mechanical.

## Trap-doors named

- **The child-seam install is the root-vs-child guard.** Verify the install lands in
  `run_forms_as_server_child` (child-only), NOT in `invoke_user_main` (shared with root). If it leaks
  into root, root's `(self-peer)` would wrongly succeed over the terminal fd0/fd1. Re-read the diff
  for the install site.
- **dup vs borrow of fd0/fd1.** The self-peer must `dup` fd 0/1 (own independent OwnedFds), not take
  ownership of the real stdio fds (that would close them). Confirm `try_clone_to_owned`/dup, and that
  the child's own readln/println (if any) still work — though a service uses `send' self`, not
  `println` (THE ONE WIRE; don't mix on fd1).
- **The RED is a child StartupError, not an outer check error.** At HEAD the outer program
  type-checks and spawns; the child fails startup (self-peer unknown) → `recv'` raises. GREEN = the
  child type-checks and echoes. (So the gate asserts compute == 105, not a startup result.)
- **infer-dispatch registration.** `:wat::program::self-peer` must be routed in the same infer
  dispatch as `socket-pair'`/`env`; if missed, the verb's type-keyword args fall back to
  value-position errors (the exact HEAD failure). Confirm the infer entry exists.

## Out of scope (must NOT appear in the diff)

Reactor listener-arm; non-blocking accept; `select'`-3arg process branch; any thread-tier change;
any change to root `invoke_user_main`; a thread-tier `(self-peer)` verb.

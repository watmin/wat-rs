# EXPECTATIONS — Stone C0b.3b-a (written before the strike)

Independent scorecard. The Inquisitor verifies each row by its own re-run before any commit.
A pure-mechanism additive primitive — the disconfirm is the probe (RED→GREEN).

| # | What | Command | Expected |
|---|------|---------|----------|
| 1 | `peer_cred` reads self over a socketpair | `cargo test --release -p wat --test comms probe_arc209_c0b3ba_peercred -- --test-threads=1` | `1 passed` (both ends' peer pid == `std::process::id()`) |
| 2 | No comms regression | `cargo test --release -p wat --test comms -- --test-threads=1` | all pass |
| 3 | Nursery baseline holds | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (4 known — ZERO new) |
| 4 | Full surface compiles | `cargo test --release --workspace --no-run` | clean |
| 5 | `PeerCred` is a plain value, `peer_cred` is `Result` | read `src/comms/process.rs` | `struct PeerCred{pid:i32,uid:u32,gid:u32}` + `fn peer_cred(fd)->io::Result<PeerCred>`; NO allow-set, NO accept change, NO wat surface |

## Runtime prediction

5–10 min. One small additive fn + struct (a `getsockopt` wrapper) + a re-export; recompile.

## Trap-doors named

- **`libc::ucred` field types:** `pid` is `pid_t` (i32), `uid`/`gid` are `uid_t`/`gid_t` (u32) —
  cast cleanly; if the target's libc differs, STOP-1.
- **Probe path resolution:** the probe imports `wat::comms::process::peer_cred` — it must be
  `pub` at that path (mirror `pair`). If not resolvable, STOP-2.
- **Scope creep:** any allow-set, `accept` enforcement, wat verb, or `kernel`/`runtime` change
  is OUT (= C0b.3b-b) — `git diff --stat` confined to `comms/process.rs` (+ `comms/mod.rs`
  re-export) + the probe.

## Honest-delta slots (filled at SCORE time)

- Did `peer_cred` over a socketpair report `std::process::id()` cleanly, or any surprise? —
- Any baseline drift in rows 2–4? Diff stat? —

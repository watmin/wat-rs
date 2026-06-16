# EXPECTATIONS — 6c.2 (scorecard written BEFORE the strike)

Baseline at `32e2e9d6` (re-run this session): lib **928 passed / 36 failed** (the 36 pre-existing, NOT
regressions). After 6c.2 the pass count RISES (new tests) and the fail count stays 36.

| # | what | command | expected |
|---|------|---------|----------|
| 1 | the new disconfirming probe is GREEN | `cargo test --release -p wat --test probe_arc272_6c2_pid_gate` | 1 passed (exact-pid admitted; same-uid wrong-pid refused; wrong-uid refused) |
| 2 | the policy rung unit test | `cargo test --release -p wat --lib only_this_peer 2>&1 \| grep "test result"` | 1 passed |
| 3 | the connect-gate unit test | `cargo test --release -p wat --lib connect_admits 2>&1 \| grep "test result"` | 1 passed (3-arg form: exact pid ok, wrong pid/uid refused) |
| 4 | the codec round-trips pid + name | `cargo test --release -p wat --lib waist_proof 2>&1 \| grep "test result"` | all passed (incl. new `address_roundtrips_pid_and_name` + arity rejection) |
| 5 | the 6a live handoff stays green | `cargo test --release -p wat --test probe_arc272_6a_capability_handoff` | 1 passed (parent dials live child; answerer pid == stamped pid) |
| 6 | autobind regression | `cargo test --release -p wat --test probe_arc272_autobind_listener` | passed |
| 7 | no lib regression | `cargo test --release -p wat --lib -- --test-threads=1 2>&1 \| grep "test result"` | passed ≥ 928 (+ the new unit tests); **failed == 36** (unchanged) |
| 8 | `AnyOfMyUser` is gone | `grep -rn "AnyOfMyUser" src/` | no matches (variant + doc + test all removed) |
| 9 | the false "unguessable ⇒ lineage-proven" claim is retracted | `grep -rn "unguessable" src/` | no security-inference uses remain (process.rs autobind-primitive doc may keep a softened mention only if it no longer implies secrecy) |
| 10 | clean build | `cargo build --release -p wat` | builds; no new warnings in the touched files |

**Runtime prediction:** 10–20 min for the Shadowdancer (logic + wire + tests + comment retractions; no
marathon cascade).

**Trap-doors named:**
- The decode validation refactor (byte checks moving from the outer vector to `items[1]`) is the most
  error-prone spot — the empty/over-long tests must be re-pointed at the inner vector, not deleted.
- `libc::getpid()` returns `pid_t` (= `i32` on Linux) — matches `PeerCred.pid` and `minter_pid: i32`.
  No cast surprise expected; confirm no `as` truncation.
- The `'a` lifetime on `CommsPolicy<'a>` is still used by `OnlyMyPeers` — keep it; `OnlyThisPeer` does
  not use it, which is fine (an enum may have a variant that doesn't use the lifetime param).

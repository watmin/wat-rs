# SCORE — a peer remembers its address; nothing was repaired

**SCORED.** Executor: grok, 2026-09-14, branch `sns-sqs`, HEAD `7497c6a95` (DRAWN). Did not commit.

⛔ The headline is that **a dialed peer can tell you where it was dialed from**.
No deadline was repaired. `call-by-deadline` is untouched. The publisher is
untouched. This is stone 1 of three — a memory and a reader.

```
     Summary [ 559.044s] 5249 tests run: 5249 passed (9 slow), 22 skipped
```

`.floor/2026-09-15T01-06-51Z/` — `scripts/floor.sh` exit **0**, **no `ARM.txt`**.
Count **5247 → 5249** (+2 tests). Not a shrink. Clippy **0**. NORUN **0**.
`git status -- wat/ wat-scripts/` **empty**.

---

## ⭑ THE HEADLINE — Some where it was dialed; None everywhere else

```
cargo nextest run --release -E 'test(a_peer_remembers_its_address)'
```

```
client-proc=Some;accepted-proc=None;fresh=Some;orig=10;fresh-reply=14;client-thread=None;accepted-thread=None;thread-orig=10;timer=None
```

| cell | result |
|---|---|
| ⭐ dialed process-tier client | `Some` |
| ⭑⭑ round trip | `connect` that address; fresh client `Some`; echo 7 → **14** |
| ⛔ original after the peek | echo 5 → **10**. `dialed-from` is a peek (`with_ref`, never `take`) |
| ⛔⛔ accepted (process) | `None` — STOP-1. Not a painted brick |
| ⛔ thread-tier client and accepted | `None` — no address exists |
| timer / dead sentinel | `None` |
| ⚠ `Peer` is not a wire type | `!is_capability_type_path(PEER_TYPE_PATH)`; `Address` still is |

---

## The five constructors, each decided (no defaulted param)

`from_socket` takes `Option<SocketAddress>` as a required argument. The
compiler is the trap-door-2 latch: a new site cannot silently forget.

| site | decision |
|---|---|
| `address.rs` `SocketAddress::connect` | ⭐ `Some(self.clone())` — `self` IS the address |
| `listener.rs` `accept` | `None` — remote bound no listener |
| `runtime.rs` poll-accept | `None` — same class as `accept` (DESIGN named `:375`; this path is the other accept) |
| `process/verbs.rs` self-peer | `None` — inherited fds, not a dial |
| `runtime.rs` timer / dead sentinel | `None` |
| `from_thread` | always `None` in the constructor — thread has no socket address |

The poll-accept site is why the required argument exists. A defaulted `None`
would have compiled it as forgetful; it would have been STOP-1 for every
`poll'` service.

---

## WHAT LANDED

- `Peer.dialed_from: Option<SocketAddress>`. `SocketAddress: Clone`.
- `(:wat::kernel::dialed-from peer) -> (Option :- [(Address :- [I O])])`.
  `infer_dialed_from` threads I,O via `project_peer_io` (the `infer_recv_prime`
  / `infer_peer_process` shape). Type params erased at runtime; the checker
  carries them.
- `with_ref`, never `take`. Closed cell raises `"peer already closed"`.
- Name is **`dialed-from`**, not `peer-address`. An accepted peer *has* a
  getpeername() and it is not dialable.
- Probe: `tests/comms/probe_a_peer_remembers_its_address.{rs,wat}`.
- Checker-skip ledger: `dialed-from` named (bespoke infer, no TypeScheme).

Did **not** touch `call-by-deadline`, the generated client method, or the
publisher (STOP-5). No `redial` primitive. No new outcome enum.

---

## Floor delta (row 8)

**559.044 s** vs baseline **535.808 s**. Delta **+23.2 s** on this box.
The orchestrator's quiet-box run is the one that grades. Observation, not
a gate. Count **+2**.

---

## GRADING MAP

| # | expected | result |
|---|---|---|
| 1 | ⭐ dialed peer remembers | `client-proc=Some` |
| 2 | ⭑⭑ round trip | fresh connection; echo 7→14 |
| 3 | ⛔ peek steals nothing | orig echo 5→10 after the read |
| 4 | ⛔⛔ accepted reports None | `accepted-proc=None` |
| 5 | ⛔ thread-tier None | `client-thread=None;accepted-thread=None` |
| 6 | self-peer + dead sentinel None | None at each site, not defaulted. Timer `None` on the probe |
| 7 | ⚠ Peer still not a wire type | `!is_capability_type_path(PEER_TYPE_PATH)` |
| 8 | floor | **5249 passed**, 0 FAIL |
| 9 | clippy | 0 |
| 10 | tests compile | NORUN=0 |
| 11 | ⛔ `.wat` corpus zero | `wat/` `wat-scripts/` empty |
| 12 | scope stated | this SCORE's headline |

# SCORE — Stone 8.2: StdInService reborn (the trio completes)

**Mode A.** Sonnet flight: ~17 min (predicted 20–35 — under band). Three
orchestrator scoring catches, one of them a DARK-CLASS surfacing (R1).

## Scorecard (every row = orchestrator's own re-run/read)

| # | Row | Result |
|---|-----|--------|
| 1 | Gate-probe 82 GREEN (2/2) | ✓ own run |
| 2 | stdin.wat: 3 forms (Req{tid} + Rep{tid,line} + ONE pure handle); EOF doctrine comment carried verbatim into the None arm | ✓ read whole (67 lines w/ comments + the R1-restored typealias block) |
| 3 | Trio generalization: ONE `ServiceMsg<R>`/`ServicePeer<R>`/`spawn_service_peer<R>`; three instantiations (write pair `R=()`, stdin `R=String`); ZERO aliases | ✓ read diff + class-grep `WriteServiceMsg\|WriteServicePeer\|spawn_write_service_peer` → zero |
| 4 | Old stdin machinery DEAD: `StdInServiceEvent`, `spawn_stdin_bridge`, `make_event_value`, `unwrap_value_sender/receiver`, `sender_value`/`receiver_value`, `extract_control_tx`, freeze's `spawn_service`/`join_service`, `stdin_thread_value` | ✓ class-grep each → zero live (retirement-record comments only) |
| 5 | eval_kernel_readln: `-> :T` parsing + EDN coerce verbatim; transport = Req{tid} → `stdin_ctrl` → `Ok(Ok)/Ok(Err→"stdin read failed")/Err` | ✓ read diff |
| 6 | Reply-routing proof (row K): two tids, fed "1"/"2", Req(a)+Req(b) → each reply_rx its own line | ✓ own run + read body |
| 7 | EOF cascade (row L): feed dropped → reply_rx.recv() Err AND `join().is_err()` — both asserted | ✓ own run + read body |
| 8 | Rows C/F/J UN-IGNORED (alpha helpers 12/0/0 — was 7/0/3); arc-170 ignore-drawdown −3 | ✓ own run |
| 9 | lib 943/0/1 · nursery 853/4/4 (the 4 = the known parked-255 reds; the new ignore is R1's banked gate) · check --all-targets 0 errors · clippy touched-surface clean | ✓ own runs |
| 10 | FULL CORPUS: **649/0/54, histogram all-zero** | ✓ own run |

## Orchestrator scoring catches

**R1 — the typealias deletion + the DARK CLASS it rode in on (FIXED + BANKED).**
The rebirth DELETED stdin.wat's load-bearing
`(:wat::core::typealias :wat::kernel::ThreadId :wat::core::i64)` — the
declaration the whole trio's Req/Rep records consume — and EVERY GATE STAYED
GREEN. Disconfirming probe proved why: **the checker LENIENTLY accepts
undeclared field-type keywords** (`:wat::kernel::CompletelyBogusNeverDeclared`
passes startup). This is the TYPE-keyword sibling of the fresh-var leniency
that hid `+'2` — the undefined-leaf dark class arc 255 exists to kill. Fixes:
(a) typealias RESTORED in stdin.wat (which loads first of the trio) with the
incident inscribed at the declaration; (b) the probe banked as an `#[ignore]`'d
arc-255 disconfirming gate (`tests/nursery/probe_diag_typealias_leniency.rs`
— un-ignore when 255 makes undeclared type keywords check errors); (c) the
self-referential load-order comment ("must load AFTER stdin.wat" in
stdin.wat itself — a copy slip) corrected. **The green check is never the
bar** — no gate could see this deletion; only reading the deletion diff did.

**R2 — warded-home doc-drift**: `src/comms/mod.rs` named the now-dead
`StdInServiceEvent` as a live future-impl example. Rewritten to the truth
(services speak Value-shaped Req/Rep, not Rust enums).

**R3 — trivial**: one `unused_mut` in the new routing test (write_all takes
`&self`). Removed.

**Sonnet's honest delta accepted**: stdin.wat 67 total lines vs the BRIEF's
"~20" — the excess is the doctrine comment the BRIEF itself demanded carried
verbatim. Code forms = 3, as designed.

## The annihilation map: the quarry is tenantless

thread_io.rs post-8.2 (~389 lines): ThreadIO + thread-local + the three eval
arms + register/deregister + RuntimeServices + ThreadId + next_thread_id —
ALL live, perfected, universe-resident machinery. ZERO condemned code
remains. **8.2w (next): lift the survivors into `src/services/`, `git rm
src/thread_io.rs`, sweep the `crate::thread_io::` imports, and cast the FULL
VIGILIA on the completed home — the trio-completion vigilatum the ward note
reserved.** Then 8.3 (child-universe boot + the deadlock tombstone).

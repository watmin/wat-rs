# Arc 208 INSCRIPTION — Process I/O returns Result (mirror arc 110/111 at process tier)

**Status:** SHIPPED 2026-05-17. `Process/readln` and `Process/println` flipped to `Result<_, Vector<ProcessDiedError>>`; `validate_comm_positions` walker extended to enforce silent Process I/O illegal at process tier; 4 consumer files converted to honest match-on-Err with ServerDied propagation; arc 203 slice 3f honest delta closed.

## What arc 208 gave the substrate

Arc 208 closed the asymmetry between thread tier and process tier on error-propagation discipline. Where arc 110/111 established `Result<_, Vector<ThreadDiedError>>` for every thread-tier send/recv site, process-tier `Process/readln` and `Process/println` panicked on disconnect. Arc 208 mirrored the thread-tier shape at process tier:

| Verb | Before arc 208 | After arc 208 |
|---|---|---|
| `:wat::kernel::Process/readln` | `[ProcessPeer<I,O>] -> :I` (panics on disconnect) | `[ProcessPeer<I,O>] -> :Result<:I, :Vector<ProcessDiedError>>` |
| `:wat::kernel::Process/println` | `[ProcessPeer<I,O>, :O] -> :nil` (panics on disconnect) | `[ProcessPeer<I,O>, :O] -> :Result<:nil, :Vector<ProcessDiedError>>` |

Plus:

- `validate_comm_positions` walker extended — `":wat::kernel::Process/readln"` and `":wat::kernel::Process/println"` added to the `matches!` list; silent Process I/O at `do`-body and function-argument positions is a freeze-time compile error at process tier, mirroring arc 110's thread-tier discipline exactly. Walker absorbed in slice 1 per `feedback_no_known_defect_left_unfixed` — the addition was two lines; atomic with the flip.

- 4 consumer files converted to honest match-on-Err with `ServerDied` propagation:
  - `wat-tests/counter-service-process-N3.wat` — 7 service wrappers (provision, deprovision, stop, get, increment, reset, test-forge) converted from `Result/expect` to nested `match` with `Err(ServiceError::ServerDied chain)` arms
  - `wat-tests/counter-actor-proof-process.wat` — 4 wrappers (get, increment, reset, shutdown) converted with `assertion-failed!` on Err (structurally honest for a proof-of-concept with no ServiceError type)
  - `tests/wat_process_peer_ipc_round_trip.rs` — embedded wat string converted from `Result/expect` to nested match
  - `tests/probe_counter_actor_process_diag.rs` — embedded wat string converted (4 call sites)

- Arc 203 slice 3f honest delta CLOSED — process-tier wrappers now surface transport failure as `ServiceError::ServerDied` through the same `Result<T, ServiceError>` return type that already carried `AccessDenied`. Transport failure no longer panics in wrapper bodies.

- `crash-test-proc` helper RETAINED with rationale — `crash-test-proc` tests `Process/drain-and-join`'s error path by spawning a subprocess that panics and detecting the abnormal exit. It has no `Process/println` or `Process/readln` calls. Its purpose is orthogonal to transport I/O Result-returning: it demonstrates the drain-and-join error path (subprocess exits AFTER communication); the transport I/O path demonstrates subprocess dying DURING communication. Both failure modes have distinct substrate paths and distinct demonstration value.

- 7 new tests in `tests/wat_arc208_process_io_result.rs` — type-scheme registration (T1), happy-path round-trip Ok (T2), Err path for dead peer println (T3), Err path for dead peer readln (T4), chain head content (T5), walker fires on forbidden println position (T6), walker fires on forbidden readln position (T7).

## Slices

| Slice | Commit | What |
|---|---|---|
| **1 — substrate flip + walker** | `44cde7b` | `Process/readln` + `Process/println` type schemes flipped in `src/check.rs:13402-13462`; eval handlers rewritten in `src/runtime.rs:17989-18099` to return `Ok(Result(Ok(v)))` / `Ok(Result(Err(chain)))` instead of panicking; `validate_comm_positions` extended at `src/check.rs:2152-2177`; 7/7 tests pass. Sub-decision settled inline: `Result<:I, ...>` (plain, no Option) because substrate transport does not distinguish clean EOF from subprocess exit. Walker absorbed in slice 1 per `feedback_no_known_defect_left_unfixed`. |
| **2 — consumer ripple + ServerDied propagation** | `9218e68` | 4 consumer files converted from `Result/expect` to honest match-on-Err; arc 203 slice 3f delta CLOSED; crash-test-proc retained with explicit orthogonal-demonstration rationale; `deftest_counter_service_process_N3` + `deftest_counter_actor_process_proof` + all consumer tests green; workspace baseline preserved (4 pre-existing failures only). |
| **3 — closure paperwork** | PENDING (orchestrator commits) | INSCRIPTION (this file) + DESIGN status CLOSED + 058 changelog row + SCORE-SLICE-3. |

## Substrate touchpoints (final inventory)

| File | Arc 208 change | Commit |
|---|---|---|
| `src/check.rs:13402-13462` | `Process/readln` + `Process/println` type scheme registrations flipped to `Result<:I, Vector<ProcessDiedError>>` + `Result<:nil, Vector<ProcessDiedError>>` | `44cde7b` |
| `src/check.rs:2152-2177` | `validate_comm_positions` walker extended — `":wat::kernel::Process/readln"` + `":wat::kernel::Process/println"` added to `matches!` list | `44cde7b` |
| `src/runtime.rs:17989-18099` | `eval_kernel_process_readln` + `eval_kernel_process_println` eval handlers rewritten — Ok path wraps in `Result::Ok`; Err path wraps in `Result::Err(single_died_chain(process_died_error_channel_disconnected()))` | `44cde7b` |
| `wat-tests/counter-service-process-N3.wat` | 7 service wrappers converted to match-on-Err with `ServiceError::ServerDied chain` propagation | `9218e68` |
| `wat-tests/counter-actor-proof-process.wat` | 4 wrappers converted to match-on-Err with `assertion-failed!` on Err | `9218e68` |
| `tests/wat_process_peer_ipc_round_trip.rs` | Embedded wat string converted from `Result/expect` to nested match | `9218e68` |
| `tests/probe_counter_actor_process_diag.rs` | Embedded wat string (4 call sites) converted to nested match | `9218e68` |
| `tests/wat_arc208_process_io_result.rs` | NEW — 7 tests covering Result shape, Ok/Err paths, chain content, walker enforcement | `44cde7b` |

## Arc 208 intentionally does NOT cover

- **`Process/stdin`, `Process/stdout`, `Process/stderr` accessors** — these expose the OS pipe ends as IOReader/IOWriter; they are not I/O verbs in the send/recv sense. Consumers who want typed-channel semantics wrap with `Sender/from-pipe` / `Receiver/from-pipe`. Arc 208's scope is the two I/O verbs that previously panicked on disconnect.

- **`Process/drain-and-join` and `Process/join-result`** — `drain-and-join` already returned `Result<nil, Vector<ProcessDiedError>>` correctly before arc 208; `join-result` is `restricted_to :wat::` and returns Result. Neither is in scope because neither was broken.

- **`Process/exit-code`, `Process/kill`, and other Process verbs** — not part of the I/O verb family. Arc 208's scope is the two verbs that connect to the send/recv transport.

- **Cross-tier transport abstraction** — `Sender/send` and `Process/println` are genuinely different transports (thread-peer vs subprocess stdin); abstracting them into one interface is the protocols arc (defservice meta-form) concern. Arc 208 keeps the honest asymmetry: same error-propagation discipline, different transport handles. Per `feedback_no_new_types`: abstracting the difference away is the wrong shape.

- **`Result<Option<:I>, ...>` shape for `Process/readln`** — Arc 208 uses `Result<:I, ...>` (plain, no Option wrapper). The substrate PipeFd transport (`src/typed_channel.rs:324-386`) maps clean EOF to `RecvOutcome::Disconnected`, which maps to the Err chain. There is no transport-distinguishable "subprocess closed stdout cleanly while still running" scenario in the current substrate architecture. The plain Result is honest. If a future consumer arrives with concrete need to discriminate clean EOF from subprocess exit, a new arc opens with that specific transport change as the design input.

- **Orphan-process leak resolution** — Arc 208 does NOT directly address the residual orphan-process leak documented in arc 170 INTERSTITIAL § 2026-05-17 "Orphan-process leak investigation." That investigation named the root cause as FD lifecycle in `spawn_process.rs` — the parent process's fd table retains references that prevent the child from receiving EOF. This is a distinct substrate concern from error-handling discipline. Arc 208 is in the same general area (honest subprocess communication) but does not fix the orphan leak. The INTERSTITIAL notes are the diagnostic record for that separate path.

- **Walker coverage at let-binding RHS position** — the `validate_comm_positions` walker reaches `WatAST::List` children (`do`-body and similar list contexts) but does NOT reach into `WatAST::Vector` binding nodes. This is the same design limit as arc 110's thread-tier walker for `Sender/send` and `Receiver/recv` — the boundary is documented in arc 110's INSCRIPTION and in slice 1 SCORE row G. Arc 208 carries the same coverage contract forward without claiming to expand it.

## Discipline lessons inscribed

### The mirror-precedent pattern (load-bearing carry-forward)

When the substrate vends asymmetric transports with the same semantic role — thread-tier `Sender/send` / `Receiver/recv` vs process-tier `Process/println` / `Process/readln` — the error-propagation discipline mirrors across them. Arc 110/111 established `Result<_, Vector<ThreadDiedError>>` at thread tier. Arc 208 mirrored to `Result<_, Vector<ProcessDiedError>>` at process tier. Same shape, different transport, same walker enforcement.

Per `feedback_simple_is_uniform_composition`: N identical compositions across similar surfaces IS simple. The mirror-precedent pattern IS the simplest possible substrate evolution when the precedent is settled. Resist the reflex to treat parallel work as "new territory" — the arc 110/111 INSCRIPTION carries the answer; the substrate asks only for application.

**Carry-forward rule:** When a new substrate transport joins the substrate — a third tier or a new communication primitive — the first question is: does it carry the same semantic role as existing transports? If yes, apply the mirror pattern. The error type changes (there is no generic `TransportDiedError`); the shape does not.

### Walker absorbed in slice 1 — the timing discipline

BRIEF § slice 1 listed the walker rule as "conditional — defer to slice 2 if non-trivial." Slice 1 audit found the extension was a 2-line addition to the `matches!` list in `validate_comm_positions`. The right call: absorb in slice 1. Splitting a 2-line substrate change into a subsequent slice creates a window where the Result flip is live but the walker is not — a discipilne gap that arc 110 closed atomically when it shipped both the Sender/send Result flip and the walker rule together.

Per `feedback_no_known_defect_left_unfixed`: when we know how to surface a failure right now, do it now. The walker was trivial. The disc gap was real.

### The substrate-as-teacher cascade ran cleanly across both slices

**Slice 1 sub-decision:** `Result<:I, ...>` vs `Result<Option<:I>, ...>` for `Process/readln`. The four-questions ran on the actual substrate evidence (`src/typed_channel.rs:324-386`) rather than intuition. The substrate's transport showed there was no discriminable "clean EOF, subprocess still alive" path. The substrate told us the shape; the four-questions confirmed it.

**Slice 2 crash-test-proc retention:** Slice 2's mandate was to retire the "workaround" that crash-test-proc represented. Audit showed crash-test-proc was never merely a workaround — it was always testing the drain-and-join path independently. The substrate's orthogonal paths (transport I/O failure vs drain-and-join post-communication exit) deserved separate demonstrations. Retention with explicit rationale is honest; premature retirement would have been wrong.

## Cross-references

- **Arc 110** (`docs/arc/2026/04/110-silent-comm-illegal/INSCRIPTION.md`) — thread-tier walker: `validate_comm_positions` rule banning silent kernel-comm at forbidden positions. Arc 208 extends this walker to process-tier verbs; arc 110's INSCRIPTION is immutable historical record.

- **Arc 111** (`docs/arc/2026/04/111-send-recv-result/INSCRIPTION.md`) — thread-tier Result flip: `Sender/send` → `Result<nil, Vector<ThreadDiedError>>`; `Receiver/recv` → `Result<Option<T>, Vector<ThreadDiedError>>`. Arc 208 mirrors this flip at process tier; arc 111's INSCRIPTION is immutable historical record.

- **Arc 112** (`docs/arc/2026/04/112-process-result/INSCRIPTION.md`) — inter-process Result attempt for the now-retired `fork-program` API. Arc 170 retired `fork-program`; arc 208 supersedes arc 112's intent for the current `ProcessPeer<I,O>` surface. Arc 112's INSCRIPTION is immutable historical record.

- **Arc 113** (`docs/arc/2026/04/113-error-chain-widening/INSCRIPTION.md`) — chain widening: single error → `Vector<error>` backtrace. Arc 208 reuses this shape (`Vector<ProcessDiedError>`); arc 113's INSCRIPTION is immutable historical record.

- **Arc 203 slice 3f SCORE lines 32-44** — the honest delta that named the gap: "process-tier user wrappers can surface AccessDenied via Result but transport failure still panics." That gap is now closed by arc 208 slices 1 + 2.

- **Arc 203 DESIGN § "What arc 203 demands from upstream" demand 2** — the upstream-demand framing that opened arc 208. Arc 203 demand 2 is now satisfied. Arc 203 closure still waits on demand 1 (protocols arc — defservice meta-form).

- **Arc 170 INTERSTITIAL § 2026-05-17 "Orphan-process leak investigation"** — arc 208 does NOT fix the orphan-process leak described there. Those notes name the FD-lifecycle root cause in `spawn_process.rs`; that investigation is a separate path from arc 208's error-handling scope.

- `feedback_inscription_immutable` — arc 110/111/112/113/203 INSCRIPTIONs and SCORE docs are immutable historical record; arc 208 cross-references them without modification.
- `feedback_no_known_defect_left_unfixed` — walker absorbed in slice 1; crash-test-proc retention verified before deciding; arc 203 slice 3f delta closed completely.
- `feedback_simple_is_uniform_composition` — the mirror-precedent pattern is simple because it IS uniform composition. N tier-flips of the same shape are simple measured complexity.
- `feedback_attack_foundation_cracks` — substrate trust is binary. Process-tier panic-not-Result was a known defect named at arc 203 slice 3f; arc 208 closed it.
- `feedback_any_defect_catastrophic` — >0 known defects = 0 trust. Arc 208 satisfies this axiom at the process tier.

---

Arc 208 inscribed. 2026-05-17.

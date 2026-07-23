# BRIEF — send' OUTCOME WALL, Strike 2b: face the SendOutcome at the PEER TEST FIXTURES (the last 19 RED)

> **Tier:** sonnet shadowdancer. **Arc:** 278 send'-wall Phase 2 (see `DESIGN-send-outcome-wall.md` STATUS).
> **Base:** Phase 1 + Strike 2a are in the working tree (66 → 19 RED). This strike faces the outcome at the
> remaining **19 peer/wire test fixtures**, targeting a green floor. Do NOT add the checker force (Phase 3).
> Leave uncommitted (the wall lands atomic).

## Why (one paragraph)

`send'` returns `:wat::kernel::SendOutcome::{Sent, Closed, Lost[cause]}`. The last 19 RED are peer/wire test
fixtures (arc-259 / arc-214 / arc-293 + the spawn echo) whose `.wat` fixtures `send'` in a round-trip and
now leak the outcome, or whose `.rs` assertions expected the old `nil`/raise. Face each, honestly, with the
same rule as 2a — **never a `_`-swallow**.

## The 19 (from the 2a floor; group by nature)

```
wat kernel::spawn::tests::spawn_thread_peer_echo_round_trip
wat::comms probe_arc214_stone46b_select_prime::probe_1_select_returns_ready_index_and_value
wat::comms probe_arc259_s2a_thread_self_peer::s2a_thread_prog_drives_self_peer
wat::comms probe_arc293_W2a_struct_no_cross::{record_still_round_trips_after_backstop, struct_crosses_thread_peer_in_locus, struct_rejected_at_wire_decode, record_still_sends_after_backstop}
wat::comms probe_arc293_W2c_compile_time_send::{record_send_to_process_peer_still_type_checks, struct_send_to_thread_peer_still_type_checks}
wat::kernel probe_arc259_s2cii_b_defclause::s2cii_b_two_arg_host_dispatch
wat::kernel probe_arc259_s2ci_spawn_thread_prime::s2ci_spawn_thread_prime_round_trip
wat::kernel probe_arc259_s2d_raii_hinge::{peer_used_then_dropped_without_close, blocked_peer_dropped_without_close_does_not_hang}
wat::program probe_arc259_peer_env_install::{thread_peer_kind_is_thread, thread_peer_reads_its_own_os_thread_id}
wat::program probe_arc259_program_init_fn::{erroring_init_fn_kills_the_peer, thread_init_populates_user_program, thread_default_user_program_is_empty_env}
wat::types probe_arc214_stone46i_typed_peer::probe_2_spawn_program_prime_thread_types_to_peer
```

Read each failing test to find WHERE the leak is (a `.wat` fixture the test `call_beside`/`startup_from_file`s,
or a Rust assertion), and classify:

1. **`.wat` fixture `send'` in a round-trip (most)** — the fixture `send'`s then `recv'`s; the death is faced
   at the `recv'` (the recv' wall). Face the send': `(match (send' p m) ((Sent) <proceed to recv'>)
   ((Closed) <proceed>) ((Lost _c) <proceed>))` — the send' outcome is faced, the death surfaces at the
   following `recv'`. NOT `(let [_ (send' …)] …)`.
2. **`.rs` type-check assertions (`probe_arc293_W2c_compile_time_send`)** — these assert `send'`'s inferred
   TYPE. It was `:wat::core::nil`; it is now `:wat::kernel::SendOutcome`. Update the expected type string in
   the assertion to `:wat::kernel::SendOutcome`. (Confirm by reading — the test likely asserts the checked
   type of a `send'` expression.)
3. **`.rs` behavior assertions expecting the old raise** (e.g. a use-after-close / dropped-peer test that
   `expect_err`'d the `"peer already closed"` / `"channel disconnected"` raise) — the raise is now a
   `SendOutcome::{Closed|Lost}` VALUE. The fixture must now `match` it and the test assert on the value (or
   the fixture surfaces it deliberately). Ground each: `s2d_raii_hinge`, `erroring_init_fn_kills_the_peer`
   are the likely raise-expecters. Update them to face the value — this is the *point* of the wall (the
   failure is now a value the test faces, not a raise it catches).

## STOP triggers

- **STOP-0:** you add the Phase-3 checker force, or touch a STDLIB `.wat` (that was Strike 2a) — STOP.
  Scope is the test fixtures/drivers of these 19.
- **STOP-1:** a test genuinely REQUIRED the raise as its contract (it was testing "send' MUST raise on a
  dead peer") and there's no clean value-facing — STOP, report it; that's a contract decision (the wall
  says it should be a value, but flag it rather than force).
- **STOP-2:** facing a fixture surfaces a NEW real bug (not just the outcome plumbing) — STOP, report.

## Verify (weigh by your own re-run)

1. `cargo build --release` compiles (the .rs assertion updates).
2. `./target/release/wat --check` clean on any edited `.wat` fixtures.
3. Floor: `mkdir -p /tmp/claude-scout && cargo nextest run --release 2>&1 | tee /tmp/claude-scout/sendwall_2b_floor.log` —
   READ the Summary. **Target: 0 failed** (all 19 faced/updated). If any remain, report them + why.
   **Do NOT run a competing floor — run ONE, in the foreground of your turn (do not background it and end
   your turn); wait for it to finish and read the Summary yourself.**

## Deliverable

The 19 faced/updated. Report: (1) each site's classification (fixture-face / type-assert-update /
raise→value-update) + the facing. (2) the floor Summary (target 0 failed). (3) `git diff --stat`. (4) any
STOP-1 contract-flags. Do NOT commit (the wall lands atomic after Phase 3).

## Blast radius

The test fixtures/drivers of the 19 (tests/comms, tests/kernel, tests/program, tests/types + the
`kernel::spawn::tests` unit test) + any `.wat` fixtures they load. NO stdlib `.wat` (2a), NO checker force
(Phase 3), NO `src/` except the `.rs` test assertions themselves. Scratch logs → `/tmp/claude-scout/`.

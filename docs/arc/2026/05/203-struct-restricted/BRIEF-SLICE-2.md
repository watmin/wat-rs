# BRIEF — Arc 203 Slice 2: Counter/Client capability proof (minimal single-user)

**Phase:** First wat-side consumer of struct-restricted. Proves the capability pattern (server-issued opaque-typed handle) works end-to-end in real consumer context, not just isolated unit tests.

**Predecessor:** Slice 1 (substrate primitive shipped at `26c9298`) — substrate accepts `:wat::core::struct-restricted` form and registers per-accessor + ctor whitelists with the arc 198 walker.

**Successor:** Slice 3 — ServiceWithProvisioning thread-tier (multi-user, Provision/Deprovision admin protocol).

## Goal

Ship one deftest demonstrating the capability pattern in action:
- Counter actor under `:counter/` namespace
- Mints `:counter::Client` capability struct via the restricted constructor
- Hands Client to the test caller
- Caller invokes client-side wrappers (under `:counter/` namespace) which internally read Client's public accessors (`in!`/`out!`) to talk
- Server-side dispatch reads Client's restricted accessors (`server-id`/`client-id`) to validate operations
- Round-trip: spawn → Increment → Get → assert → Shutdown → assert final state

Single user. Single state. No Provision/Deprovision admin protocol. The pattern is the value; the bigger demo is slice 3.

## Required form (the `:counter::Client` capability)

Per arc 203 DESIGN settled shape:

```scheme
(:wat::core::struct-restricted :counter::Client
  [:counter/]                                                          ;; only :counter/* can mint Client/new
  ([:counter/] server-id <- :wat::core::keyword                        ;; only :counter/* can read server-id
   [:counter/] client-id <- :wat::core::keyword)                       ;; only :counter/* can read client-id
  (in!  <- :wat::core::Sender<counter::Request>                        ;; any caller can read in! (talk channel)
   out! <- :wat::core::Receiver<counter::Response>))                   ;; any caller can read out! (listen channel)
```

`server-id` is the server actor's secret-witness (random UUIDv4 at spawn time); `client-id` is per-user (random UUIDv4 at Provision). Server validates incoming operations by reading `Client.server-id` and comparing to its own (the server-internal value stored when the actor was spawned).

## Required artifacts

### `wat-tests/counter-client-capability-proof.wat`

One deftest with the full flow:

**Prelude declarations (top-level under deftest prelude form):**
- `:counter::Request` enum: `(Get)`, `(Increment :wat::core::i64)`, `(Reset)`, `(Shutdown)` (per arc 170 INTERSTITIAL Counter pattern with corrected unit-variant list syntax)
- `:counter::Response` enum: `(Value :wat::core::i64)`, `(Ok :wat::core::i64)`, `(Final :wat::core::i64)`
- `:counter::Client` struct via `:wat::core::struct-restricted` per the form above
- `:counter/spawn` defn: takes `initial-state <- :wat::core::i64`; spawns thread (using arc 091 `uuid::v4` for server-id + initial client-id); returns `:counter::Client`
- `:counter/dispatch` defn: server-side message loop; takes ThreadPeer; matches Request variants; handles Get/Increment/Reset/Shutdown per arc 170 Counter pattern
- `:counter/get`, `:counter/increment`, `:counter/reset`, `:counter/shutdown` client-side wrappers: take `:counter::Client`; access `Client.in!` + `Client.out!`; round-trip via the mini-TCP lockstep

**Body (the test):**
- Spawn `:counter/spawn` with initial state 10 → bind `client!`
- `:counter/increment client! 5` → assert returns 15
- `:counter/increment client! 7` → assert returns 22
- `:counter/get client!` → assert returns 22
- `:counter/reset client!` → assert returns 0
- `:counter/shutdown client!` → assert returns 0 (Final state)

This MIRRORS the existing `wat-tests/counter-actor-proof-thread.wat` test from the prior Counter actor proof commit, but using `:counter::Client` as the capability bundle (replacing the bare ThreadPeer + manual id tracking pattern). Side-by-side comparison: same observable behavior; structurally cleaner capability flow.

## Scope (no substrate changes)

Pure consumer. Zero edits to src/. All artifacts in wat-tests/ + docs/.

If you find yourself wanting to extend the substrate, STOP — slice 1 shipped what's needed; surface the gap as a slice 2 honest delta.

## STOP triggers

1. **`struct-restricted` form rejects something the BRIEF assumes works** — slice 1 might have a parsing edge case the BRIEF didn't anticipate; surface, don't paper over
2. **`uuid::v4` not callable from a `:counter/`-prefixed defn** — arc 091 ships it under `:wat::measure::uuid::v4` (or similar); verify exact FQDN from `crates/wat-measure/` or `src/runtime.rs` before assuming; surface if the verb namespace differs from BRIEF assumption
3. **Restricted accessor enforcement blocks a server-side read that should work** — `:counter/dispatch` reads Client's restricted fields; if walker fires on legitimate server-side access, the BRIEF's `:counter/` prefix expectation is wrong; surface
4. **Workspace baseline regresses** beyond the 3 pre-existing failures (deftest_wat_tests_tmp_totally_bogus + startup_error_bubbles_up_as_exit_3 + t6_spawn_process_factory_with_capture_round_trips) — STOP

## HARD constraints

- **DO NOT commit.** Orchestrator commits atomically after independent verification.
- **cwd discipline:** FIRST action: `cd /home/watmin/work/holon/wat-rs/`; verify with `pwd`. NEVER operate in `.claude/worktrees/`.
- DO NOT touch substrate code (src/). Pure consumer slice.
- DO NOT use `--no-verify` / `--no-gpg-sign`.
- DO NOT add Rust drivers; this is wat-side via `:wat::test::deftest` + `:wat::test::assert-eq`.
- DO NOT mint new substrate types/verbs/structs/special-forms.

## Decay disclosure (orchestrator)

Counter actor pattern is established (wat-tests/counter-actor-proof-thread.wat shipped at `9b0c517`; INTERSTITIAL § 2026-05-16 deeper has the canonical shape). The `:counter::Client` capability wrapping is the NEW shape this slice introduces — uses arc 203 slice 1's struct-restricted.

Substrate behaviors to verify before drafting code (don't speculate):
- `uuid::v4` FQDN — grep `crates/wat-measure/` or `src/runtime.rs` for the exact path; surface if non-obvious
- `:counter/dispatch` accessing `Client.server-id` — should work because `:counter/dispatch` is under `:counter/` prefix; if walker fires anyway, that's a slice 1 honest delta

## SCORE methodology

4 rows YES/NO + evidence:

| Row | What | Evidence |
|-----|------|----------|
| A | Counter/Client mints cleanly via struct-restricted; deftest compiles | `cargo test --release -p wat --test test deftest_counter_client_capability_proof` builds without TypeError |
| B | End-to-end round-trip succeeds (Increment, Get, Reset, Shutdown all assert correctly) | Same test passes |
| C | Workspace failure count = baseline (3 pre-existing) | `cargo test --release --workspace --no-fail-fast` shows ≤ 3 failures (the 3 documented pre-existing ones) |
| D | Capability pattern matches DESIGN — server reads server-id/client-id (restricted accessors) inside `:counter/`-prefixed defns; user only invokes wrappers (no direct access to restricted fields outside `:counter/`) | code review of the artifact confirms structural pattern |

## Time-box

Predicted: 30-60 min sonnet. Hard stop: 90 min.

## Workspace baseline

Same 3 pre-existing failures as slice 1's baseline. Post-slice-2 target:
- Pass count: ≥ baseline + 1 (one new deftest passes)
- Fail count: ≤ 3 (no regressions)

## On completion

1. Write `docs/arc/2026/05/203-struct-restricted/SCORE-SLICE-2.md` per § SCORE methodology.
2. Return final summary: rows passed/failed, workspace delta, file paths touched, any honest deltas surfaced (especially around substrate behavior assumed by BRIEF that turned out to differ), suggested DESIGN corrections (if any).

You are launching now. T-minus 0.

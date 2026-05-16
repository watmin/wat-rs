# Arc 170 — Bracket Combinator Implementation Stones

**Purpose:** track the implementation stones for the bracket-combinator direction (run-threads + run-processes + walker collapse + fallout). Design phase captured in `INTERSTITIAL-REALIZATIONS.md` across six 2026-05-16 entries. This file is the WORKING CHECKLIST — check off as each stone ships.

**Discipline anchors:**
- `feedback_test_first` — write failing test BEFORE implementation
- `feedback_iterative_complexity` — small funcs; prove each stepping stone
- `feedback_simple_forms_per_func` — cap at ONE outer let* per function
- `feedback_attack_foundation_cracks` — cracks are diagnostic
- `feedback_refuse_easy_solutions` — no transitional helpers, no scaffolding
- `feedback_realizations_open_directions` — arc 170 stays OPEN until INSCRIPTION

---

## Stones

### Stone A — `Thread/drain-and-join` + `Process/drain-and-join` substrate helpers — **SHIPPED 2026-05-16**

- [x] Test: Thread happy + panic, Process happy + panic — `tests/wat_arc170_stone_a_drain_and_join.rs` (4/4 green)
- [x] Implementation: `eval_kernel_thread_drain_and_join` (`src/runtime.rs:16949`) + `eval_kernel_process_drain_and_join` (`src/runtime.rs:16445`) + drain helpers
- [x] Dispatch arms: `src/runtime.rs:4288` (Process) + `:4329` (Thread)
- [x] Type registrations: `src/check.rs:12482` (Process) + `:12619` (Thread)
- [x] No existing tests broken (workspace baseline maintained)
- [x] No callers migrated yet — Stone B handles that

**Scope:** Substrate (`src/runtime.rs` + `src/check.rs`).
**Actual:** ~50 min sonnet (predicted 90-120). 6/6 SCORE rows PASS.
**SCORE:** `SCORE-STONE-A-DRAIN-AND-JOIN.md`

---

### Stone B — Walker collapse: hide `*_join-result` from user namespace — **SHIPPED 2026-05-16**

- [x] Test: user wat code calling `Thread/join-result` → compile error
- [x] Test: same for `Process/join-result`
- [x] Test: substrate-namespace caller (`:wat::*`) → check passes (Thread + Process)
- [x] Implementation: `validate_join_result_user_namespace` (`src/check.rs:3094`) + `CheckError::JoinResultUserNamespace`; hooked into `check_program` at `src/check.rs:1939`
- [x] ~40 user-namespace `*_join-result` call sites migrated to `*_drain-and-join` across 18 files (crates/wat-* + tests/ + wat-tests/)

**Scope:** Walker + caller sweep.
**Actual:** ~75 min sonnet (predicted 90-120). 6/6 SCORE rows PASS.
**Note:** Ad-hoc walker rule; arc 198 (`defn-restricted`) will generalize this into a substrate primitive; future refactor replaces this specific rule with primitive use.
**SCORE:** `SCORE-STONE-B-WALKER-COLLAPSE.md`

---

### Stone C — Mint `Thread/Client<I,O>` + `Thread/Server<I,O>` + `Process/Client<I,O>` + `Process/Server<I,O>` type pairs

- [ ] Test: declaring `Thread<I,O>` auto-generates `Thread/Client<I,O>` + `Thread/Server<I,O>` companion types
- [ ] Test: verbs dispatch on side (client.readln returns O; server.readln returns I)
- [ ] Test: same for Process pair (client + phantom-server-uses-ambient)
- [ ] Implementation: substrate type generation in `src/types.rs`; verb registration

**Scope:** Substrate type system.
**Predicted:** 120-180 min sonnet — type-system work is fiddly.
**Dependencies:** none (independent of A/B).

---

### Stone D — `run-threads` bracket macro

- [ ] Test: minimal — single factory + single client-fn; result threads through
- [ ] Test: multi-factory — 3 factories with different `Thread<I,O>` types; tuple of Thread/Client handles passed to client-fn
- [ ] Test: panic in any factory → graceful shutdown to siblings → ProcessGroupErr propagates
- [ ] Implementation: variadic defmacro in `:wat::kernel::*`; expands to N spawn + Tuple-construct + client-fn call + N drain-and-join
- [ ] Reference: `:wat::test::program` at `wat/test.wat:228-231` is the variadic macro precedent

**Scope:** Wat-level macro implementation.
**Predicted:** 90-120 min sonnet.
**Dependencies:** Stone A (drain-and-join helper) + Stone C (Client type).

---

### Stone E — `run-processes` bracket macro

- [ ] Test: minimal — single factory + single client-fn
- [ ] Test: multi-factory
- [ ] Test: panic cascade
- [ ] Implementation: mirror Stone D for processes
- [ ] Verify: process-server uses ambient `(readln)`/`(println)`; client uses `Process/Client/*` verbs

**Scope:** Wat-level macro implementation.
**Predicted:** 60-90 min sonnet (mirror of Stone D).
**Dependencies:** Stone A + Stone C (Process pair).

---

### Stone F — Migrate -with-io callers; delete fallout

- [ ] Migrate `wat-tests/kernel/services/ambient-stdio.wat:117` (Layer 2 readln-echo) to bracket
- [ ] Migrate `tests/wat_arc170_program_contracts.rs:1046` (T18 echo-doubled) to bracket
- [ ] Migrate `tests/wat_arc170_program_contracts.rs:1123` (T18b assert-fail) to bracket
- [ ] Delete `:wat::test::run-hermetic-with-io` macro (`wat/test.wat`)
- [ ] Delete `:wat::test::run-hermetic-with-io-driver` fn
- [ ] Delete `:wat::test::run-hermetic-send-inputs` helper
- [ ] Delete `:wat::test::run-hermetic-drain-outputs` helper
- [ ] Delete `:wat::test::RunResultIO<O>` struct registration in `src/types.rs`
- [ ] Delete `:wat::test::run-hermetic-with-prelude` macro (`deftest-hermetic` inlines expansion)
- [ ] Migrate proof deftest at `wat-tests/test.wat:157-161` to plain `deftest-hermetic`

**Scope:** Wat + Rust caller sweep + macro/struct deletion.
**Predicted:** 120-180 min sonnet.
**Dependencies:** Stones D + E shipped.

---

### Stone G — Retire arc 117/133 sibling-binding walker machinery

- [ ] Verify: all sibling-binding deadlock scenarios now caught by bracket walker rule (Stone B) OR not relevant under new substrate
- [ ] Delete arc 117/133 machinery from `src/check.rs` (sibling classification + sender-bearing detection + process-join-before-output-drain)
- [ ] Walker now consists of: binary `*_join-result`-in-user-namespace check + standard type checks
- [ ] Verify: all tests still pass

**Scope:** Walker retirement.
**Predicted:** 60-120 min sonnet.
**Dependencies:** Stone B + Stone F (ensures no regressions hidden by old machinery).

---

### Stone H — INSCRIPTION + USER-GUIDE + Recovery doc updates

- [ ] Draft `INSCRIPTION.md` for arc 170 capturing the eight-step trajectory (argv-to-main → OTP supervision)
- [ ] Update USER-GUIDE: bracket combinator section; remove -with-io references; add actor-model framing
- [ ] Update Recovery doc Section 13 (IPC contract) — extend if needed for bracket semantics
- [ ] Update CONVENTIONS.md if new naming patterns surface
- [ ] Update ZERO-MUTEX.md if relevant
- [ ] Mark task #325 closed; task #229 (arc 109 v1 milestone) re-evaluates

**Scope:** Docs.
**Predicted:** 90-180 min orchestrator.
**Dependencies:** All implementation stones shipped + workspace tests green.

---

## Open questions before Stone A starts

1. **Helper naming:** `Thread/drain-and-join` vs `Thread/await` vs `Thread/finalize` vs `Thread/collect`. Default = `drain-and-join` (honest about what happens).
2. **Stone ordering confirm:** A → B → C in parallel → D + E in parallel → F → G → H. Adjust if user wants different cadence.
3. **First slice protocol:** orchestrator writes BRIEF + EXPECTATIONS; sonnet executes Stone A; orchestrator writes SCORE. Standard cadence.

---

## Status

- [x] Design phase complete (2026-05-16, captured in INTERSTITIAL-REALIZATIONS.md)
- [x] Stone A — drain-and-join helpers (2026-05-16, ~50 min, 4/4 tests green)
- [x] Stone B — walker collapse (2026-05-16, ~75 min, 4/4 tests green, +40 migrations)
- [ ] Stone C
- [ ] Stone D
- [ ] Stone E
- [ ] Stone F
- [ ] Stone G
- [ ] Stone H

**Arc 170 closes via Stone H's INSCRIPTION — not before.**

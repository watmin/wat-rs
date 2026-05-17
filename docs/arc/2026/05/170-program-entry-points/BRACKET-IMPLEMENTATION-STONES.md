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

### Stone C — Mint `ThreadPeer<I, O>` + `ProcessPeer<I, O>` substrate types — **REVISED 2026-05-16**

**Original framing (`Thread/Client<I,O>` + `Thread/Server<I,O>` + Process pair) superseded** per INTERSTITIAL-REALIZATIONS.md § 2026-05-16 (Stone C revision). Single `ThreadPeer<I, O>` type with type-param swap encodes the Client/Server distinction — the side is conceptual, not structural. Process server stays ambient (asymmetry honest at substrate-primitive level).

**Decomposed into 2 sub-stones per `feedback_iterative_complexity` (and the arc 198 slice 2 calibration lesson — small bounded stones beat one-shot type-system work):**

#### C1 — `ThreadPeer<I, O>` type + 2 verbs + tests

- [ ] Mint `:wat::kernel::ThreadPeer<I, O>` substrate type (I = read direction; O = write direction; peer-relative naming)
- [ ] Mint `:wat::kernel::Thread/readln peer -> :I` verb
- [ ] Mint `:wat::kernel::Thread/println peer data:O -> :wat::core::nil` verb
- [ ] Tests: type minting + verb dispatch + type-param swap semantics (two peers wired together; one reads what the other writes)
- [ ] Substrate-internal pipe wiring helper (not yet exposed to bracket; that's Stone D)

**Scope:** Substrate type-system addition + 2 verb registrations.
**Predicted:** 30-45 min sonnet.
**Dependencies:** none.

#### C2 — `ProcessPeer<I, O>` type + 2 verbs + tests (mirror)

- [x] Mint `:wat::kernel::ProcessPeer<I, O>` substrate type (client-side wrapper around Process/stdin + Process/stdout)
- [x] Mint `:wat::kernel::Process/readln peer -> :I` verb
- [x] Mint `:wat::kernel::Process/println peer data:O -> :wat::core::nil` verb
- [x] Tests: type minting + verb dispatch + interaction with existing Process/stdin/stdout/stderr accessors
- [x] Process server stays ambient (uses bare `(readln)` / `(println)`) — no peer struct minted on server side
- [x] Document the asymmetry: Thread has peer-on-both-sides; Process has peer-on-client-only

**Scope:** Substrate type-system mirror of C1 + 2 verb registrations + interaction with existing Process accessors.
**Predicted:** 30-45 min sonnet (mirror pattern from C1).
**Dependencies:** Stone C1 (template established).

---

### Stone D — `run-threads` bracket macro — **DECOMPOSED 2026-05-16**

**Original monolithic Stone D (single-factory + multi-factory + panic cascade) superseded** per INTERSTITIAL-REALIZATIONS.md § 2026-05-16 (Stone D design pass). Per Stone C calibration — `feedback_iterative_complexity` + `feedback_simple_forms_per_func` — bounded stones beat one-shot multi-piece work.

Four-questions outcomes (settled with user 2026-05-16):
- **Factory signature:** `:Fn(ThreadPeer<I, O>) -> :nil` (A) — peer is the surface everywhere else; spawn-thread's raw channels stay inside the macro adapter. YES YES YES YES.
- **Client-fn signature:** variadic positional `(client-fn peer₁ peer₂ ... peerₙ)` (A) — Lisp-natural; concrete types post-expansion; no Tuple destructure. YES YES YES YES.
- **Decomposition:** D1 (single-factory) + D2 (multi-factory heterogeneous) + D3 (panic cascade) (decompose) — Stone C lesson; each stone one teaching moment. YES YES YES YES.

#### D1 — minimal `run-threads` with single factory + round-trip

- [x] Mint `:wat::kernel::run-threads` macro accepting bare factory + client-fn (D1 supports single-factory only; D2 extends to N via variadic positional collector). Tuple-wrapped form deferred per honest delta: wat has no expand-time AST destructuring, so extracting child from `(Tuple factory)` AST is not expressible in a wat-level defmacro.
- [x] Macro expansion target: `(let [thread (spawn-thread <wrap-fn>) client-peer (ThreadPeer/new (Thread/output thread) (Thread/input thread)) result (client-fn client-peer) _ (Thread/drain-and-join thread)] result)`
- [x] `<wrap-fn>` = `(fn [server-rx <- :Receiver<I>, server-tx <- :Sender<O>] -> :nil (factory (ThreadPeer/new server-rx server-tx)))` — bracket converts raw spawn-thread sig to ThreadPeer for the user's factory. Honest delta: macro takes pre-baked `server-rx-type` (full `Receiver<I>` keyword) + `server-tx-type` (full `Sender<O>` keyword) as positional args; wat tokenizes parametric type keywords `<...>` atomically so `~` unquote does NOT splice inside `<>` brackets at expand time (same constraint `:wat::test::run-hermetic-with-io` documented at wat/test.wat:800-815).
- [x] Test: single factory echoes one String round-trip; client sends "hello" via Thread/println, reads back via Thread/readln, asserts
- [x] No panic cascade yet (factory completes cleanly; D3 handles panics)

**Scope:** Wat-level macro + 1 test.
**Predicted:** 30-45 min sonnet.
**Dependencies:** Stone A (drain-and-join) + Stone C1 (ThreadPeer).

#### D2 — multi-factory heterogeneous `(Tuple factory-A factory-B ... factory-N)`

- [ ] Extend the macro to pattern-match N children of the Tuple AST
- [ ] Expansion generates N let-bindings (each its own concrete `ThreadPeer<Iₖ, Oₖ>`), variadic client-fn invocation, N drain-and-join calls
- [ ] Test: 3 factories with heterogeneous types (e.g. `ThreadPeer<String,i64>`, `ThreadPeer<i64,String>`, `ThreadPeer<String,String>`); client interacts with each; asserts
- [ ] Verify: heterogeneous tuple iteration via macro expansion (NOT runtime tuple iteration); types resolve at expansion time

**Scope:** Macro extension + 1 multi-factory test.
**Predicted:** 30-45 min sonnet.
**Dependencies:** D1.

#### D3 — panic cascade + `ProcessGroupErr`

- [ ] Factory panic → bracket detects via drain-and-join Result; cascades shutdown to siblings; wraps as `ProcessGroupErr`
- [ ] Macro expansion changes: wrap drain-and-join Results, decide cascade policy on first Err
- [ ] Bracket return type: `Result<R, ProcessGroupErr>` (was raw `R` in D1+D2)
- [ ] Test: 2-factory setup; one factory panics mid-stream; verify sibling is signaled to shut down cleanly; verify `ProcessGroupErr` carries first panic + sibling-shutdown-status
- [ ] If `ProcessGroupErr` enum doesn't exist yet → mint it (small substrate addition, vetted via four-questions first)

**Scope:** Macro panic-cascade extension + new error type + 1 panic-cascade test.
**Predicted:** 60-90 min sonnet (panic semantics + new substrate type).
**Dependencies:** D1 + D2.

**Reference:** `:wat::test::program` at `wat/test.wat:228-231` is the variadic macro precedent (variadic `&` collector + `~@` splice).

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
- [x] Stone B — walker collapse (2026-05-16, ~75 min, 4/4 tests green, +40 migrations; ad-hoc rule retired by arc 198 slice 2 Stone 4 on 2026-05-16; tests now pass via arc 198's walker)
- [x] Stone C1 — `ThreadPeer<I, O>` + 2 verbs (2026-05-16, ~35 min, 3/3 tests green)
- [x] Stone C2 — `ProcessPeer<I, O>` + 2 verbs + real-spawn integration test (2026-05-16 post-revision, substrate-composition proof; user-facing surface is Stone D's run-processes bracket; commit `e4b9461`)
- [x] D1 — minimal `run-threads` single-factory + round-trip (2026-05-16, initial commit `d704820` verbose-form; refactored same-day to clean call form `(run-threads :I :O factory client-fn)` via arc 143 slice 2's computed-unquote pattern after arc 199 REJECTED — substrate already sufficient; 1/1 test green; baseline preserved at 4)
- [ ] D2 — multi-factory heterogeneous expansion (unblocked — arc 201 closed 2026-05-16; full reflection chain shipped: signature-of-fn + extract-arg-types + Bundle/children + Bundle/first; build on D1's clean call form)
- [ ] D3 — panic cascade + `ProcessGroupErr` — depends on D2
- [ ] Stone E (decomposes per same pattern when D family settles) — unblocked
- [ ] Stone F
- [ ] Stone G
- [ ] Stone H

**Arc 170 closes via Stone H's INSCRIPTION — not before.**

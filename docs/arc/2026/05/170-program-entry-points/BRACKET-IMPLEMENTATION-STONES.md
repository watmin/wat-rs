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

#### C3 — type-keyword honesty fix (revises C1/C2's deliberate shortcut) — **OPEN 2026-05-17**

**The defect Stone C2 left behind:** ThreadPeer<I,O> + ProcessPeer<I,O> field types are declared as `:rust::crossbeam_channel::Receiver<I>` / `:rust::crossbeam_channel::Sender<O>` (src/types.rs:1003-1066). The COMMENT at lines 1040-1045 acknowledges the shortcut explicitly: *"The Receiver<I> / Sender<O> field types are deliberately the SAME typed-channel substrate ThreadPeer uses — `typed_recv` / `typed_send` are transport-polymorphic (Crossbeam tier-1 for threads, PipeFd tier-2 for processes), so the Process/readln + Process/println eval handlers can mirror Thread/readln + Thread/println verbatim modulo the struct tag."*

The runtime IS transport-polymorphic (the Value wrapper branches between crossbeam-backed and PipeFd-backed inner at recv/send time). But the TYPE-KEYWORD at the user level lies — a Process's "Sender" is NOT a `crossbeam_channel::Sender`; it's an OS-pipe-backed typed-channel abstraction. ProcessPeer's PipeFd-backed transport is named after the wrong crate.

**The honest answer (per arc 109 K-channel rename):** `:wat::kernel::Sender<T>` / `:wat::kernel::Receiver<T>` are the canonical names for the typed-channel abstraction. Arc 109 already minted these as aliases (src/check.rs:3056-3057 + 492-493); they unify with the underlying crossbeam at the type system level. Renaming the FIELD-TYPE keywords to the honest abstraction names costs ~0 runtime behavior (aliases unify) and ~0 cognitive load (anyone reading ProcessPeer's declaration knows what they ARE).

- [ ] Update `src/types.rs` ThreadPeer + ProcessPeer field declarations: `:rust::crossbeam_channel::Receiver<I>` → `:wat::kernel::Receiver<I>`; `:rust::crossbeam_channel::Sender<O>` → `:wat::kernel::Sender<O>`
- [ ] Update `src/check.rs` `Sender/from-pipe` + `Receiver/from-pipe` return type registrations to the honest names
- [ ] Sweep consumers in tests/ + wat-tests/ + wat/ that explicitly reference `:rust::crossbeam_channel::Sender/Receiver` in type-annotation positions; substitute the honest names
- [ ] Workspace test: 0 regressions (aliases unify; behavior unchanged)
- [ ] No new walker code, no new error variants — pure rename/sweep

**Scope:** Substrate rename + consumer sweep. ~10-30 sites depending on consumer surface.
**Predicted:** 60-90 min sonnet.
**Dependencies:** C1 + C2 shipped (the target types exist).
**Blocks:** arc 203 slice 3 (ServiceWithProvisioning) — slice 3 wants to declare struct fields honestly without inheriting the lie.

**Origin:** User flagged 2026-05-17 mid arc 203 slice 2 spawn. Sonnet's transcript revealed it was about to follow ProcessPeer's pattern and propagate the lie to Counter/Client. User's framing: *"why is a process using a crossbeam with stdio?"* → four-questions confirmed Path A (fix substrate FIRST). Per `feedback_attack_foundation_cracks` + `feedback_any_defect_catastrophic` + `feedback_no_known_defect_left_unfixed`: substrate trust binary; fix before more consumers inherit.

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

#### D2 — multi-factory heterogeneous (coordinator-fn form) — **SHIPPED 2026-05-16**

**Design revised** (per BRIEF-STONE-D2-COORDINATOR.md): D1's positional-types call form (`(run-threads :I :O factory client-fn)`) retired; D2 ships the coordinator-fn form (`(run-threads (fn [a <- ThreadPeer<I,O> ...] -> T (:user::fn a ...)) factory-a factory-b factory-c)`) for BOTH N=1 and N=3.

- [x] `run-threads` variadic macro rewritten for coordinator-fn form: reflect coordinator fn at expand time via `signature-of-fn → extract-arg-types → Bundle/children → atom-value → keyword/to-string + string::concat + keyword/from-string` to extract `Receiver<I>` / `Sender<O>` channel types; dispatch to `run-threads-n1` (N=1) or `run-threads-n3` (N=3) sub-macros
- [x] `run-threads-n1` helper macro (N=1 fixed template): coordinator arg names via `extract-arg-names + to-watast → WatAST::Symbol` as valid let binder; peer pairing via `ThreadPeer/new(Thread/output, Thread/input)`; coordinator invocation via `(~coordinator ~@arg-names)`
- [x] `run-threads-n3` helper macro (N=3 fixed template): same pattern for 3 slots; literal index-based names `thread-0/1/2`, `_drained-0/1/2`
- [x] D1 test updated to coordinator-fn form: `(run-threads (fn [peer <- ThreadPeer<S,S>] -> S (:my::echo-client peer)) :my::echo-factory)` with keyword factory reference
- [x] D2 test: 3 heterogeneous-behavior factories (uniform `ThreadPeer<String,String>` types for type-system clarity); coordinator delegates to named fn `(:my::three-fac-coordinator a b c)`; asserts `["hello","world","pong"]`
- [x] STOP-trigger-1 disclosed: fresh binding name construction from keyword (`keyword/from-string → WatAST::Keyword`) blocked by `parse_let_binding`; resolved by using literal index-based names for thread/drain slots and coordinator's own binder names for peer slots
- [x] Factory call-form convention settled: keyword references (`factory-name`, not `(factory-name)`) used throughout; macro template `(~factory-k (ThreadPeer/new server-rx server-tx))` direct-calls the factory fn with peer — honest delta on original BRIEF's call-form convention
- [x] Baseline preserved: 4 pre-existing failures unchanged; 2 new tests pass (D1 updated + D2 new)

**Scope:** Macro rewrite in `wat/kernel/run_threads.wat` + D1 test update + D2 new test.
**Actual:** TBD min sonnet (coordinator-fn form + N-dispatch via computed-unquote + arc 201 reflection chain at expand time).
**SCORE:** `SCORE-STONE-D2-COORDINATOR.md`
**Dependencies:** D1 (ThreadPeer primitives) + arc 201 (reflection chain) + arc 200 (macro vector handling).

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
- [x] D2 — coordinator-fn form macro rewrite (2026-05-16; N=1+N=3 via run-threads-n1/n3 sub-macros; arc 201 reflection chain at expand time; D1 test migrated to coordinator-fn; D2 new 3-factory test passes; 2/2 tests green; baseline preserved)
- [ ] D3 — panic cascade + `ProcessGroupErr` — depends on D2
- [ ] Stone E (decomposes per same pattern when D family settles) — unblocked
- [ ] Stone F
- [ ] Stone G
- [ ] Stone H

**Arc 170 closes via Stone H's INSCRIPTION — not before.**

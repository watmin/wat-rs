# 6.w VIGILIA FINDINGS — the triple-ward diagnostic pass (PRE-FIX LEDGER)

> **STATUS at curare (2026-06-07, compaction-near): DIAGNOSTIC PASS 11/12 INWARD
> DONE; tree clean at deb21407 (no fix edits yet).** This doc preserves every
> ward's findings so the fix-sweep is recoverable post-compaction WITHOUT
> re-casting 12 wards. The agent transcripts are ephemeral; THIS is the record.
>
> **BUILDER STANCE (load-bearing):** *"shockingly stable is our greedy stance —
> i doubt you'll convince me to bank anything."* → EVERY solvable finding gets
> FIXED in the sweep. No runes-as-deferral for anything in-reach. Runes only
> for: (a) genuinely-correct primitives mis-flagged, (b) doctrine-sanctioned
> ambient exemptions made grep-honest, (c) perspicere's intentional-depth.

## STILL PENDING (the next self does these FIRST, before the sweep)
1. ~~**excusare**~~ — **REPORTED (folded in below, "EXCUSARE VERDICTS").** channel/
   + kernel/ carry ZERO exemptions; process/ has 4 (1 L1 strike, 1 HOLDS, 2 L3
   text). runtime.rs SHUTDOWN statics carry no suppressions — the prose-vs-rune
   question resolves CLEAN (a `// SAFETY:` on an `unsafe{}` is not a checker
   suppression, so sequi F9 needs no rune). One side-catch: a stale doc lie.
2. ~~**circumspicere**~~ — **REPORTED + GROUNDED (folded in below, "CIRCUMSPICERE
   VERDICTS").** 0 L1, 3×L2, 1×L3 — clean perimeter, no ship-blocker. All 4
   findings verified true at their cited file:lines by the orchestrator. It
   explicitly ruled out the items already in this ledger (no double-report) and
   ruled out the comms raw_fds() worry (design-aware, comms/process.rs:35).
ALL 14 WARDS REPORTED. Now: draw the convergence sweep (sonnet, sequenced —
process/ structural first), execute, re-verify L1+L2=0, lay the TRIPLE vigilatum
(channel/ + process/ + kernel/ each get the stamp).

## THE SPINE — δ-2/δ-3 pidfd migration (resolves FIVE findings at once)
`src/process/handle.rs` `wait_or_cached` + `Drop` use raw `libc::waitpid(self.pid)`
/ `libc::kill(self.pid)` instead of the `Pidfd` methods sitting in the same
struct. Flagged by **sequi (F6/F7 L1), struere (PR-1 L1), temperare (P2),
secare (L2 + TOCTOU)** — 4 wards. AND purgare flagged `Pidfd::poll_exit` +
`try_wait` as DEAD (zero callers). **SAME ROOT:** the methods are dead because
the race-path doesn't use them. THE FIX (= arc-213 δ-2/δ-3, methods exist at
clone.rs:159 wait_status / :190 send_signal):
- migrate `wait_or_cached` → `self.pidfd.wait_status()` (map ExitStatus→i64 via
  the extract_exit_code shell convention)
- migrate `Drop` → `self.pidfd.send_signal(SIGKILL)` + `wait_status()`
- secare TOCTOU: gate the wait behind `reaped.compare_exchange(false,true,...)`
  so exactly one caller proceeds (Drop vs concurrent wait_or_cached)
- retire the raw `pid` field (δ-3); `extract_exit_code` (handle.rs) retires with
  it (solvere F-PR-3 dual-exit-decode resolves here)
- this brings poll_exit/try_wait ALIVE (purgare F2/F3 resolved by USE not delete)
- VERIFY: process join + the gamma/hermetic enveloped tests still green.

## CHANNEL/ (channel/{mod,inner,transfer}.rs)
- **Stale "Crossbeam" labels** (intueri L2): transfer.rs:48/147/263 doc + mod.rs:41
  → "comms::thread" (Crossbeam HARD-CUT at 5.1). FIX.
- **Duplicate EDN-decode body** (solvere F-CH-1 L2): typed_recv PipeFd arm
  (~241-256) and typed_try_recv PipeFd arm (~343-354) identical → extract private
  `decode_pipe_line(Option<String>) -> RecvOutcome`. FIX.
- **`_types`/`_span` named-unused-but-USED** (struere CH-2 L2): typed_try_recv
  PipeFd arm consumes them (read_line uses _span, read_edn uses _types) → drop the
  underscores. FIX.
- **sender_close advisory-only** (sequi F2, struere CH-1 L2): Comms variant only
  sets the flag; structural disconnect waits for Arc drop → DOCUMENT honestly
  (the flag gates sends; the peer sees EOF on last-clone-drop). FIX = doc/contract.
- **shutdown_rx() per try_recv** (temperare C-2 L2): called every invocation incl.
  hot no-data case → move inside the `None`/disconnect arm. FIX.
- **SeqCst broadcast-fd load** (temperare C-1, L3): :201/:311 → Acquire (once-written
  fd). FIX (cheap correctness-equiv).
- **typed_try_recv "non-blocking" blocks on partial line** (struere CH-1 L2):
  read_line after poll(0) blocks on partial write → document "poll-gated not
  read-gated". FIX = doc (write side always atomic newline frames).
- **typed_recv shutdown-dep not in signature** (sequi F1 L2): RecvOutcome::Shutdown
  IS in the return; the global dep is ZERO-MUTEX ambient-context → HOLD + rune:
  this is the doctrine's declared cascade channel (not threadable without bloating
  every signature). rune:sequi(ambient-context).
- **try_as_comms_receiver deep type** (perspicere CH-1): rune:perspicere(read-once).
- **select/make-pipe-channel "follow-up arc" prose** (exigere C-1 L1 transfer.rs:361,
  C-2 L1 :394): "would need epoll for a follow-up arc" / "slice-2 if a real
  consumer demands it" → affirmative scope-bound (Slice 7 brackets is the named
  home for select-over-pipe; make-pipe-channel is intentionally Rust-internal) or
  remove. FIX = reword affirmative.
- mora: arc-253 2-state collapse HELD (no regression) — PASS, no action.

## PROCESS/ (process/{mod,clone,child,handle,verbs,stdio}.rs)
- **emit_panics_to_stderr_fork/_spawn + emit_structured_exit dup** (intueri L1,
  solvere KNOWN-DEBT, struere PR-2, temperare P1 — 4 wards; verbs.rs:6 says
  "merge is 6.w's task" = THIS stone): merge to one `emit_panics_to_stderr` +
  one `emit_structured_exit` (libc::write IS fork-safe, the verbatim-keep was a
  lift-discipline now discharged). FIX. Resolves the exigere P-4 comment too.
- **clone3-name-lie** (intueri known-debt): `eval_kernel_fork_program(_ast)`,
  `fork_program_from_source`, `run_in_fork`, `ForkedProgramHandles` use clone3 not
  fork(2). Surface names map to `:wat::kernel::fork-program*` (the wat verb) — so
  either rename Rust internals to clone_* + keep wat verb spelling, OR inscribe
  "fork = the wat verb name; impl is clone3" at each decl. DECIDE in sweep (lean:
  inscribe — the wat surface is the contract; renaming Rust churns dispatch). FIX.
- **ChildHandleInner phantom-outer** (intueri known-debt): no `ChildHandle` outer
  exists → rename to `ChildHandle` (callers use Arc<ChildHandle>). FIX.
- **input/output vs stdin/stdout vocab split** (intueri L1): eval_kernel_spawn_process
  uses input_r/w out_r/w; siblings use stdin_/stdout_ → normalize to stdin/stdout.
  FIX.
- **3 child-branch dup** (solvere F-PR-1 L1, ~400 lines): child_branch /
  child_branch_from_source / spawn_process_child_branch share the 6-step post-fork
  pipeline → extract `run_forked_child(...)` shared kernel in child.rs; the 3 arms
  provide only world-construction. FIX (structural; biggest item — verify carefully).
- **conformare span discards**: verbs.rs:231 (L1 eval_kernel_fork_program_ast),
  :1051 (L1 eval_kernel_spawn_process) — list_span in scope, thread it; :704 (L2,
  arc-138 comment misapplied, list_span available) thread it + delete the wrong
  comment; :613 fork_program_from_source spanless-by-domain BUT re-span at the :720
  propagation (list_span available) via .map_err. FIX all 4.
- **mod wrap LoaderWrap inline in fn body** (solvere F-PR-2 L2): verbs.rs:562 →
  hoist to module level (pub(super) struct). FIX.
- **_inherit_config never read** (purgare F5, struere): verbs.rs:503
  fork_program_from_source — WIRE it into child_branch_from_source (so fork-program
  inherits like fork-program-ast does), not delete. FIX (capability consistency).
- **LifelineWriter::close dead** (purgare F4): zero callers (all use into_owned_fd)
  → DELETE. FIX.
- **CloneArgs over-export** (purgare F1): pub → pub(super); drop the mod.rs re-export.
  FIX.
- **spawn_lifelined_any verbatim clone of spawn_lifelined** (temperare P3, solvere):
  extract `spawn_lifelined_inner(Box<dyn FnOnce(i32)>)`; the two public fns apply
  UnwindSafe/AssertUnwindSafe at the surface. FIX.
- **wait_or_cached name** (intueri L2): → `wait_or_cached_exit`. FIX (or moot if
  the δ-2 migration restructures it).
- **Phase 1C stale comment** (exigere P-3 L1): handle.rs:40 "Once Phase 1C ships..."
  describes shipped work → present tense (document why Option<lifeline> persists).
  FIX.
- **perspicere runes**: try_wait Result<Option<ExitStatus>> (PR-1 read-once),
  spawn_lifelined(_any) Result<(Pidfd,LifelineWriter)> (PR-2 mumble-alias),
  expect_option_string/expect_vec_ast (PR-3 read-once ×2). ADD runes. FIX.
- **δ-1/δ-2 attested-arc comments** (exigere P-1/P-2 L2, arc 213 verified): become
  MOOT once the δ-2 migration ships in this sweep (the spine). Remove/update.
- clone.rs:289 design-choice annotation (exigere P-5): NOT a finding — keep.

## KERNEL/ (kernel/{mod,peer,spawn}.rs)
- **:process tier discards sym** (struere KR-1 L1, sequi F5 L1): spawn.rs:431
  child apply-loop uses `SymbolTable::new()` (bare) while :thread threads sym →
  programs calling helpers fail silently under :process. FIX = `let child_sym =
  sym.clone()` before the closure (mirror thread tier) + VERIFY with a NEW
  process-tier-helper round-trip test (does the cloned sym survive the fork +
  resolve a `:my::helper` call). The CAPABILITY fix the builder wants done, not
  banked. If sym-clone-across-fork proves unsound, THAT makes it a deeper stone —
  fight first.
- **_program_env phantom arg** (struere KR-2, sequi F4 L2): spawn.rs:204 evaluated
  + discarded ("Stone 4.6 wires it") → surface honestly: error on non-nil env OR
  reduce arity to 2 now. DECIDE in sweep (Stone 4.6 is real, but a silent phantom
  is dishonest). FIX = surface it.
- **mini-TCP metaphor** (intueri L2): spawn.rs:259 "Two mini-TCP pairs" → factual
  "bounded channel pairs (comms::thread::pair, depth-1, cascade-aware)". FIX.
- **child/pidfd vs join asymmetry** (intueri L2): peer.rs:150 Process.child vs
  Thread.join → rename Process.child → Process.pidfd (parallel to the type names).
  FIX.
- **spawn_lifelined_any facade bypass** (solvere F-KE-1 L2): spawn.rs:398 calls
  crate::process::clone::spawn_lifelined_any directly; not re-exported from
  process/mod.rs → add `pub(crate) use clone::spawn_lifelined_any`. FIX.
- **HolonRepresentable for Value placement** (solvere F-KE-2 L3): spawn.rs:97 impl
  for a core type in a peripheral module → move to value home OR comment-point. FIX
  (light).
- **kernel/mod.rs Stone 4.5 stale "does NOT live here"** (exigere K-1 L1): spawn.rs
  IS here → update to shipped; Stone 4.6 bullets → attested-arc rune (arc 214). FIX.
- **:remote future-arc in error string** (exigere K-2 L1): spawn.rs:231 user-facing
  error embeds "(:remote is a future arc)" → "supported tiers: :thread, :process".
  FIX.
- **thread-peer silent error swallow** (secare L3): spawn.rs:285 `Err(_) => break`
  discards program errors; parent sees only RecvError → send the error as a message
  before exit (message-addressed). FIX (or L3-document if out of scope).
- **test sleep** (mora L1): spawn.rs:579 thread::sleep(50ms) in echo-round-trip test
  → join the peer (the JoinHandle is the wire). FIX.
- **SHUTDOWN_* statics lack formal runes** (sequi F9; excusare may confirm): the
  arc-6.4 shutdown statics in runtime.rs carry prose not `// rune:sequi(...)` →
  ADD rune:sequi(ambient-context) (ZERO-MUTEX cascade; grep-honest). FIX.
- **Arc::clone per apply-loop iter** (temperare K-1 L3): structural (apply_function
  takes Arc by value) → LEAVE (callee-API-bound; not a defect).

## EXCUSARE VERDICTS (reported post-curare; 4 exemptions, all in process/)
channel/ + kernel/ = 0 exemptions. runtime.rs SHUTDOWN statics = 0 suppressions
(sequi F9 resolves CLEAN — `// SAFETY:` is not a checker silence). process/ = 4:
- **P-1 (clone.rs:43) L1 STRIKE:** `#[allow(non_camel_case_types)]` on `CloneArgs`
  is INERT — the type is already UpperCamelCase; the lint never fires; no reason
  text; dead since birth (arc 213 α `5e43d7cb`). **FIX: delete the allow** (no code
  change). This is the only thing standing between process/ and clippy-zero-honest.
- **P-2 (verbs.rs:296) HOLDS:** `too_many_arguments` on `child_branch` (10 params)
  — structural warrant stated verbatim in the doc (six fds + forms + config +
  lifeline raw + lifeline OwnedFd; one call site @218). CLEAN, no action.
- **P-3 (verbs.rs:754) L3 TEXT:** `too_many_arguments` on `child_branch_from_source`
  (12 params) HOLDS but the doc delegates to the sibling without naming the count
  → state "Twelve parameters: four source-parse (source, canonical, loader, argv)
  + the same eight fd/RAII params as child_branch."
- **P-4 (verbs.rs:1111) L3 TEXT:** `too_many_arguments` on `spawn_process_child_branch`
  (10 params) HOLDS but the doc cites `fork.rs::child_branch_from_source` — a DEAD
  FILE (killed 6.3 `97542181`) → re-point to `src/process/verbs.rs::child_branch_from_source`.
- **SIDE-CATCH (runtime.rs:248, out of excusare scope, for the sweep):** the
  `SHUTDOWN_BROADCAST_READ_FD` doc says "Once set, never re-set (idempotent init)"
  — now a LIE: 6.4's `init_shutdown_signal_with_inputs` re-sets it in fork children
  (~line 356). Stale doc lie → FIX the comment to state the fork-rebirth re-set.

## CIRCUMSPICERE VERDICTS (the perimeter, cast LAST; reported post-curare; all GROUNDED true)
0 L1, 3×L2, 1×L3. Every finding orchestrator-verified at its file:line. circumspicere
did NOT re-report ledger items (stale-Crossbeam labels, the runtime.rs:248 idempotent
doc-lie = excusare side-catch, the secare thread-tier swallow) and ruled OUT the comms
raw_fds() worry (comms/process.rs:35 documents ring+pipe fd survival; 4.5 was design-aware).
- **F1 (L2) CLAIM-VS-CODE — out-of-home doc lie:** `docs/ZERO-MUTEX.md:478-489` says
  the cli's shared state is `CHILD_PID` + `kill(2)` forwarding. CODE (verified) is
  `CHILD_PGID` (`crates/wat-cli/src/lib.rs:118`) + `killpg(2)` (`:615`) — arc 106
  generalized PID→PGID (lib.rs:111 says so); the doc never caught up. FIX = two-word
  doc edit (CHILD_PID→CHILD_PGID, kill(2)→killpg(2) broadcast). Out-of-home (not a
  ward target) but a shipped claim circumspicere is built to catch → FIX in the sweep.
- **F2 (L2) EGRESS — fork-unsafe exit, IN-SCOPE (runtime SHUTDOWN init):**
  `runtime.rs:334` + `:349` use `std::process::exit(1)` in the pipe2()-failure branches
  of `init_shutdown_signal_with_inputs` — which runs in FORK CHILDREN (via child.rs:327
  step 4). `std::process::exit` runs atexit handlers (the parent's, COW-inherited) in a
  fork child → unpredictable teardown. The adjacent `libc::write(2,…)` already uses raw
  libc. FIX = `std::process::exit(1)` → `unsafe { libc::_exit(1) }` (×2; one-char-class,
  matches the neighbor). VERIFIED on disk.
- **F3 (L2) NEGATIVE-SPACE — silent opaque child exit (kernel/ home):** `spawn.rs:427/433/
  444/452` the `spawn_process_peer` child apply-loop exits `libc::_exit(1/0)` with NO
  structured diagnostic, unlike EVERY verbs.rs child (which calls `emit_structured_exit`
  → `#wat.kernel/ProcessPanics` EDN to stderr). Parent sees only `Exited(1)`, no cascade.
  This is also a SILENT-SWALLOW dark-class instance ([[feedback_silent_swallow_is_dark_class]]),
  sibling to the inward secare L3 (thread-tier spawn.rs:285). FIX = emit a structured
  diagnostic before the error `_exit`s (re-export/reuse emit_structured_exit or the
  emit_panic_envelope pattern at stdio.rs:125) + a NEW process-tier-error-diagnostic test.
- **F4 (L3) UNENFORCED-INVARIANT — undocumented benign double-close (runtime, IN-SCOPE):**
  the wake-fd is closed at child.rs:322 (close_range step 3) then again at runtime.rs:309-311
  (rebirth guard step 4) → EBADF, discarded. Benign (single-threaded child, no fd recycling
  step3→4). CODE IS ALREADY the guarded `if old_write_fd >= 0` form. FIX = add the cross-step
  comment so a future editor doesn't "fix" the ordering into a real recycled-fd double-close.

## SWEEP DECOMPOSITION (sequenced; the convergence drawn from the full map)
Greedy stance → every finding above FIXED. Sequenced by risk/file to keep each kill
verifiable (examinare: small strikes, weigh each):
- **Strike 1 — process/ structural (heaviest, riskiest, FIRST):** THE SPINE (δ-2/δ-3
  pidfd migration, handle.rs) + 3-child-branch dedup (solvere F-PR-1, child.rs) +
  emit_panics/emit_structured_exit merge + all process/ inward mechanical + excusare
  P-1 strike/P-3/P-4 doc + circumspicere **F2** (_exit ×2) + **F4** (double-close comment)
  + the runtime.rs:248 idempotent-doc side-catch. Verify: clippy process/ clean, lib
  green, enveloped gamma/hermetic green, the δ-2 exit-code decode preserved.
- **Strike 2 — kernel/:** KR-1 :process sym-clone capability fix + NEW process-tier-helper
  test + circumspicere **F3** structured diagnostic + NEW diagnostic test + all kernel/
  inward mechanical. Verify: clippy kernel/, lib, arc214 nursery, the 2 new tests.
- **Strike 3 — channel/:** all channel/ inward mechanical (labels→comms::thread, decode
  dedup, drop underscores, doc contracts, SeqCst→Acquire, exigere affirmative rewords).
  Verify: clippy channel/, lib.
- **F1** (ZERO-MUTEX.md doc) folded into Strike 1 (adjacent to the runtime fixes).
- Strikes 2+3 are file-disjoint from each other (kernel/ vs channel/) → may run parallel
  AFTER Strike 1 lands green (Strike 1's spine restructures handle.rs that kernel/ depends
  on at the type level; sequence it first to avoid a moving floor under Strike 2).

## PROCESS/ RE-CAST #1 (post-Strike-1; the "consistently zero" loop — NOT yet zero → Strike 1b)
Full inward vigilia re-cast over process/ via workflow (each agent fetched its signed spell).
3 wards clean (temperare/secare/sequi). circumspicere HUNG mid-perimeter (killed; re-run
standalone next round — it's the odd one out, must not block the barrier). Inward findings =
the Strike 1b worklist (Strike 1 left residue — the loop caught it):
- **solvere L2 (verbs.rs:415-469 ≡ 875-924):** Strike 1's dedup merged the two FORMS callers
  into run_forked_child but left child_branch_from_source's BYTE-IDENTICAL exit-protocol tail
  (validate→run-main→outcome→EXIT_*-map→envelope) un-extracted. FIX = extract
  `fn finish_forked_child(world) -> !`; both callers keep only their world-builder + tail-call it.
  (+ L3: emit_structured_exit/emit_panics_to_stderr share the ProcessPanics envelope encode →
  extract `emit_chain_envelope`.)
- **struere L2 (verbs.rs:1298):** `.spawn(...).expect("Thread::Builder::spawn failed")` PANICS on
  EAGAIN/RLIMIT_NPROC while the OS-process siblings map_err to a clean RuntimeError. FIX =
  return `Ok(startup_error_result("thread spawn failed: {e}"))` (match the freeze-fail path :1141).
- **excusare L2 (verbs.rs:821) — STRUCK ILLEGITIMATE-AT-BIRTH:** the `rune:purgare(arc-031-
  incomplete)` on inherit_config cites CLOSED arc-031; the awaited `startup_from_source_with_inherit`
  doesn't exist; no open successor → deferral-to-nowhere silencing a live unused-binding finding.
  FIX = re-point at a named OPEN arc that adds the freeze primitive (structurally-right owner =
  freeze.rs) OR RIP the inherit_config param + its plumbing (the honest YAGNI cut). DECIDE in sweep.
- **conformare L2 (verbs.rs:702):** eval_kernel_fork_program ScopedLoader error uses
  Span::unknown() while args[1].span() is available (the sibling at :719 patches list_span). The
  arc-138 "no WatAST trace" comment is FALSE. FIX = args[1].span().clone() + delete the wrong comment.
- **exigere L2 (verbs.rs:363):** "deferred per DESIGN" — unverifiable tracker (work is arc 012's
  Scope-through-fork slice). FIX = rune:exigere(attested-arc) naming arc 012, or scope-affirmative.
- **exigere L2 (verbs.rs:818-820):** "not yet exposed / until that primitive exists" inherit_config
  deferral — pairs with the excusare strike above; resolve together.
  (+ L3 clone.rs:296-299 scope-defense "would require touching spawn_process.rs (scope violation)"
  → present-tense rationale.)
- **intueri L2 — STALE BREADCRUMBS from the merge:** comments point at DELETED files/symbols —
  verbs.rs:350 "line 1116"→806; verbs.rs:954 "src/fork.rs:574"→verbs.rs (eval_kernel_fork_program_ast);
  clone.rs:307 "fork.rs:1447"→clone.rs:191; child.rs:206 "fork.rs send_signal"→clone.rs::send_signal;
  clone.rs:238/73 reference removed extract_exit_code; clone.rs:297/verbs.rs:247,626/stdio.rs:15 cite
  deleted spawn_process.rs. FIX = re-point to live locations (prefer fn-names over line numbers — drift-proof).
- **purgare L3 ×2 (clone.rs:129 poll_exit, :167 try_wait):** STOP-3 keep is LEGITIMATE but UNMARKED
  → add `// rune:purgare(safety-margin) — completes the pidfd primitive surface; no current caller`
  to each, OR delete. (excusare HELD the perspicere read-once runes on them; the deadness is purgare's.)
- **perspicere L3 ×3 (runtime.rs:195/216/233):** SHUTDOWN_RX_PTR/shutdown_rx()/SHUTDOWN_TX_PTR carry
  AtomicPtr<Receiver<()>> (2-level) → mint `type ShutdownRx/ShutdownTx` (reuse ×3) OR rune
  intentional-structure (Receiver<()> is self-naming). Human/sweep judgment.
- **excusare HELD (good):** the run_forked_child + child_branch_from_source too_many_arguments allows
  (domain-truth, RAII per-fd Drop) and the perspicere read-once runes — all HOLD.

## PROCESS/ RE-CAST #2 (post-Strike-1b; inward-only workflow, circumspicere standalone)
NOT zero → Strike 1c. The complementarity law in action: 7 inward findings (0 L1, 4 L2, 3 L3),
5 wards clean (sequi/temperare/exigere/secare/excusare), excusare HELD all of Strike 1b's runes.
Split: 2 introduced by Strike 1b's dedup + 5 pre-existing instances the earlier passes' sampling
missed. circumspicere standalone REPORTED: 0 L1, 2 L2, 1 L3 — clean perimeter, ruled
OUT the scares (Arc drop/wait race impossible; setpgid idempotent; no broadcast-fd leak;
ZERO-MUTEX verified). Its 3 perimeter findings fold into Strike 1c below (CIRC-F1/F2/F3).
**Strike 1c worklist (6 L2 + 4 L3):**
- **struere L2 (handle.rs:34) — the sharpest:** ChildHandle's 4 fields are `pub` but the doc
  promises a reap-once invariant the public fields don't enforce — any crate code could
  `handle.reaped.store(true)` and silently disable the Drop reap → zombie leak. grep confirms
  ZERO cross-module field access (all flow through ChildHandle::new + wait_or_cached_exit/Drop in
  handle.rs). FIX = drop `pub` from reaped/cached_exit/lifeline_w/pidfd (zero-behavior tightening
  that lets the type enforce the documented coordination). [pre-existing latent gap]
- **solvere L2 (verbs.rs:214):** dispatch-arg encoding bypass — expect_vec_ast/expect_string/
  expect_option_string/arity_2 exist but eval_kernel_fork_program_ast (:214-239) + spawn_process
  (:922-946) + fork_program (:670/:678/:689) hand-roll the same parse inline (Vec<WatAST> ×3,
  String/Option/arity ×2). FIX = route through the helpers; enrich expect_vec_ast to carry the
  per-element + outer span both arms use. [pre-existing]
- **conformare L2 (verbs.rs:247):** make_pipe error paths (4 sites: :247-249, :959-961, :1190-1192
  via :1109/:1152) propagate via bare `?` without re-stamping list_span, while the sibling
  spawn_lifelined paths DO (.map_err at :308/:1029). FIX = re-stamp the make_pipe sites with
  list_span (leave make_pipe's own Span::unknown() — correct at the primitive). [pre-existing, complementary to round-1's verbs.rs:702]
- **intueri L2 (verbs.rs:371):** run_forked_child's TWO doc blocks contradict — first ends
  "Called from exactly one site" (STALE, dedup gave it 2 callers @295/@1016), second correctly
  lists both. FIX = fold into one `///` above the #[allow], delete the stale sentence. [Strike-1b-introduced]
- **solvere L3 (verbs.rs:377):** residual post-fork PROLOGUE+epilogue dup between run_forked_child
  (:389-435/:464-491) and child_branch_from_source (:795-833/:851-889) — Strike 1 extracted only
  the tail (finish_forked_child). FIX = extract `redirect_stdio_and_init(...)` + `run_user_main_in_child(world) -> !`;
  each caller reduces to prologue-call + world-build(+set_argv) + epilogue-call. [pre-existing; deliberate prior boundary — finish the extraction]
- **perspicere L3 (verbs.rs:123):** finish_forked_child's `outcome: std::thread::Result<Result<Value,
  RuntimeError>>` (2-level) is the lone un-runed deep type (3 siblings are runed); the match needs
  the structure → DECIDED: add `// rune:perspicere(intentional-structure)` (do NOT alias). [Strike-1b-introduced]
- **purgare L3 (runtime.rs:465):** `#[cfg(test)] pub fn reset_shutdown_signal()` zero callers, doc
  claims a non-existent consumer → DECIDED: DELETE (leaf, no behavior change; no cascade-re-init
  test is planned). [pre-existing]
- **CIRC-F1 L2 (claim-vs-code; mod.rs:9 vs clone.rs:310):** mod.rs:9 ships "Linux 5.3+" but
  close_range is 5.9+; on 5.3-5.8 it ENOSYS-skips silently (child.rs:202) and the child inherits
  all parent fds — the exact hygiene invariant the step claims. FIX = raise mod.rs:9 to the real
  floor ("clone3 5.3+, close_range 5.9+, deploy floor 6.x") to match the code; (the breadcrumb +
  realizations already state the 6.x deploy floor). [circumspicere]
- **CIRC-F2 L2 (negative space; verbs.rs:396-403/:802-809 vs child.rs:292-294 claim):** dup2
  failures BEFORE child_post_fork_init _exit(EXIT_STARTUP_ERROR) with NO ProcessPanics envelope,
  while child.rs:292-294 claims "all failures" emit structured. DECIDED: emit a minimal
  libc::write(2,...) diagnostic before each pre-init _exit (makes the claim TRUE by construction
  rather than narrowing it — failure-engineering); the dup2 source fds are the just-created pipe
  ends so the borrowed-stderr write is safe. [circumspicere]
- **CIRC-F3 L3 (egress; stdio.rs:76-101):** lend_ambient passes fd -1 to OwnedFd::from_raw_fd on
  dup failure (UB precondition; close(-1)=EBADF, benign). FIX = guard dup_fd failure explicitly
  (return Result/Option or abort with a clear message) rather than hand an invalid fd to from_raw_fd. [circumspicere]

## WARD SET — WIDENED to FOUR homes (2026-06-07, builder caught the gap)
6.w was framed as the TRIPLE (channel/ process/ kernel/) but the real 214-touched
surface is FOUR homes — and I had tunnel-visioned process/ alone for 3 strikes:
- **process/** — created 6.3. Re-cast round 2 CLOSED (Strike 1/1b/1c green). Round 3 = confirm consistent-zero. (no stamp yet)
- **channel/** — created 6.1. NOT warded. (Strike 3 brief committed.)
- **kernel/** — created Slice 4. NOT warded. (Strike 2 — to draw.)
- **comms/** — the engine (Slices 1-3), STAMPED 2026-06-01 (9-spell) — but **DRIFTED**:
  5 commits touched it AFTER the stamp (arc253 try_recv 2-state collapse `d3150a04`;
  4.5 raw_fds `5cf044e4`; 8.2/6.3/6.4), last modified 2026-06-07 19:21 (thread.rs).
  The stamp certifies stale code → comms/ needs a DRIFT RE-CAST to the current bar + re-stamp.
- (services/ STAMPED 2026-06-08 full 14-ward AFTER all the above — current, NOT in scope.)
NEW PLAN (off the one-home march): process/ spine is committed → kernel/, channel/, comms/
are file-disjoint → ward all in PARALLEL (concurrent inward workflows + standalone
circumspicere each), loop each to consistent-zero, then vigilatum all four.

## KERNEL/ — FIRST WARD (inward done; 26 findings 5 L1/15 L2/6 L3; circumspicere pending)
secare/excusare/conformare CLEAN (excusare: 0 exemptions in the home). Strike 2 worklist:
- **CAPABILITY FIX (KR-1) — struere/sequi/intueri all flag spawn.rs:420:** `:process` child
  apply-loop uses capability-less `SymbolTable::new()` vs `:thread`'s `sym.clone()` (:265) →
  programs calling defined helpers fail silently under :process. sequi: NO technical blocker
  (fork copies the address space; captured sym.clone() is valid in the child). FIX = clone sym
  into the child closure before the fork (mirror :thread) + a NEW process-tier-helper round-trip
  test (a :process program that resolves a `:my::helper`). STOP only if sym holds a non-fork-safe
  resource (fd/thread handle) making the clone unsound → then it's a deeper stone, fight first.
- **HolonRepresentable for Value (solvere:97 L2 + purgare:97 L3 + solvere:431 L2):** DEAD (zero
  consumers; process tier uses String wire), misplaced in the spawn dispatcher, AND a 3rd copy of
  the Value↔EDN-string codec. FIX = DELETE the impl; extract `value_to_edn_string`/`edn_string_to_value`
  in edn_shim.rs as the single codec; route the child apply-loop (spawn.rs:431/448) through them.
- **peer fields pub→pub(crate) (struere:53 L2, peer.rs):** Thread<I,O>/Process<I,O> fields all pub
  but doc says "never constructed by user code" + kernel is pub mod → external code could break the
  input+output+join/child invariant. FIX = pub→pub(crate) (all constructors in-crate; zero-cost).
- **thread-peer error swallow (struere:285 L2):** :thread loop `Ok(Err(_))|Err(_) => break` discards
  the fn error/panic; parent's recv()→RecvError indistinguishable from clean close, while :process
  signals via _exit(1). Tier asymmetry + silent-swallow dark class. DECIDE in sweep: surface the
  error to the parent (typed error frame on the output channel — parity with :process) OR, if that's
  a protocol change beyond 6.w, document the contract honestly + rune(host-constraint) and note as a
  named follow-up feature. Lean: fight to surface; STOP→document if it ripples to consumers.
- **temperare:347 L2:** `parent_types = sym.types().map(|t|(**t).clone())` deep-clones the ENTIRE
  TypeEnv per :process spawn, only to pass `&parent_types` by borrow to extract_closure. FIX =
  borrow the Arc'd TypeEnv directly (default only on None). Drops O(types) clone per spawn.
- **exigere (5 L1 + ~8 L2):** L1 — mod.rs:15-17 stale "## What does NOT live here (pending stones)"
  lists the spawn dispatcher as pending though it LIVES here → rewrite to present lineage; spawn.rs:231-233
  ":remote is a future arc" in a USER-FACING error string → "supported tiers: :thread, :process";
  spawn.rs:161 + :204-205 + :94 bare future-prose → present-tense. L2 — the many "Stone 4.6 will…" /
  "until Slice 5 retires" refs are rune-eligible (4.6 + Slice 5 verifiable on disk) → add
  rune:exigere(attested-arc) citing the DESIGN/BRIEF, OR convert to present-lineage where 4.6 sub-stones landed.
- **_program_env phantom arg (struere:205 L3 / exigere:204):** arg[1] evaluated + discarded ("Stone 4.6
  wires it"). 4.6a-i/ii/b are DONE per the breadcrumb — INVESTIGATE in sweep whether it should be threaded
  NOW or the positional dropped; surface honestly either way (no bare deferral).
- **perspicere:544 L3:** `Arc<ThreadOwnedCell<Option<Thread<Value,Value>>>>` (4-level, recurs ~8×) →
  mint `type ThreadPeerCell = …` typealias.
- **intueri L3 ×3:** spawn.rs:335 duplicate doc summary (fold to one above #[allow]); :259 "mini-TCP"
  metaphor → factual "crossbeam bounded(1) pairs"; :420 add the child_sym WHY comment (pairs w/ KR-1).

## KERNEL/ circumspicere (perimeter; 2 L1 / 3 L2 / 1 L3 — fold into Strike 2)
- **F-1 L1 (peer.rs:52,141 vs tests/comms/peer_process_round_trip.rs:118):** "never constructed
  directly by user code" is FALSE — the integration test builds `Process{…}` via pub fields.
  This is the COMPLEMENT to inward struere:53 (which assumed zero cross-module construction).
  FIX = add a kernel peer test-constructor (mirror channel's make_thread_peer_pair_for_test) +
  pub→pub(crate); OR keep pub + soften the doc claim with a rune. Prefer the constructor+pub(crate)
  (enforces the invariant); STOP→soften if the test-constructor is too invasive.
- **F-2 L1 (DESIGN.md:571 "same fn body runs in :thread or :process"):** contradicted by the empty
  SymbolTable. RESOLVED BY the KR-1 capability fix (clone sym → claim becomes true). No separate work.
- **F-4 L2 (spawn.rs:143-147 ProcessPeerBundle):** drop-order invariant (peer before _lifeline_w)
  is load-bearing but unguarded → add `// INVARIANT: declaration order is load-bearing; DO NOT reorder`.
- **F-5 L2 (spawn.rs:432-444):** :process child fn-error → _exit(1); parent gets bare RecvError with
  no error channel. OVERLAPS inward struere:285 (resolve together: surface the error, or document the
  close().wait_status() recovery path + the Stone 4.6 work item).
- **F-7 L3 (peer.rs:189-195 Process::close):** doc says "closes the pipe read end" but Drop also
  closes the persistent io_uring ring fd → add the second-resource note.
- **F-3 L2** is in comms/ (PIPE_BUF comment overstated) — NOT kernel/; the comms/ drift re-cast catches it.

## STDIO-SERVICES-AS-WAT (8.4 design direction — builder, 2026-06-07; post-6.w)
GROUNDED CORRECTION: the trio is ALREADY the driver-model. `wat/kernel/services/{stdout,stderr,
stdin}.wat` exist; `src/services/peer.rs:81 handle_fn: Arc<runtime::Function>` = the handler is a
WAT fn. Shape: driver (Rust libc::write/read via IOWriter/IOReader, exposed `:wat::io::*`) ← wat
service handler (wat/kernel/services/*.wat, EDN-encode + IOWriter/write-all) ← Rust universe loop
(spawn_service_peer). Exactly the sqlite pattern (rust driver hidden, wat management tooling). So
8.4 `:wat::services::start` = the wat-surface verb wrapping spawn_service_peer so USER wat code starts
services the way freeze.rs starts the trio; DOGFOOD = route the trio boot through the verb (or ship a
wat demo service). NOT "rewrite trio in Rust" (wrong) and NOT "trio can't be wat" (wrong — it already
is). OPEN design Q for 8.4: can the verb run at freeze-time bootstrap, or does the trio boot stay a
special Rust path with the verb proven via a user demo? Decide when 8.4 is drawn. SHIP, not cut.

## KERNEL/ RE-CAST #2 (post-Strike-2; Strike 2b worklist — ALL doc/comment drift from Strike 2's edits)
NOT zero → Strike 2b. inward 8 (0 L1/3 L2/5 L3, 6 wards clean) + circumspicere 3 (1 L1/2 L2).
No risky code (one #[doc(hidden)] attr). The loop caught Strike 2's wake:
- **circ F1 L1 (peer.rs:205-207):** Process::close doc FALSELY credits Pidfd::Drop with closing the
  io_uring ring fd — the ring fd is in self.output (Receiver), closed by drop(self.output) BEFORE the
  Pidfd returns; Pidfd closes only the pidfd. (Strike 2's OWN F-7 doc-fix introduced this.) FIX = correct.
- **circ F2 L2 (spawn.rs:29,36,148):** 3 doc sites describe the payload type WITHOUT the Option layer the
  new ThreadPeerCell alias added → reference ThreadPeerCell + add a ProcessPeerCell alias for symmetry.
- **circ F3 L2 (spawn.rs:10):** module doc "forks via spawn_lifelined" → "spawn_lifelined_any" (repointed at F-KE-1).
- **excusare L2 ×2 (spawn.rs:142,189) CLOSED-DEFERRAL:** the attested-arc runes cite 4.6a-i for RUNTIME
  env-wiring, but 4.6a-i SHIPPED check-only (validates args[1]:wat::program::Env at check.rs:10709-10723;
  never owned runtime wiring; _program_env discarded at spawn.rs:190). FIX = convert to present-lineage
  (4.6a-i shipped the check-time env validation; runtime accepts the 3rd arg to match the check arity,
  evals for side-effects) + re-point the runtime-env-wiring to TASK #211 (the real open owner). NO shipped-stone citation.
- **exigere L2 (peer.rs:249-250):** test doc "Stone 4.5 is not yet built" is a STALE LIE (4.5 IS built) →
  present-tense (hand-built to stay lib-safe; mirror the sibling at spawn.rs:507-508).
- **struere L3 (peer.rs:165):** Process::new_for_test pub+ungated → add #[doc(hidden)] (test-only ctor
  shouldn't read as a sanctioned public constructor in rustdoc).
- **purgare L3 (spawn.rs:89):** ThreadPeerCell doc overclaims adoption (the 6 downcast sites are in
  runtime.rs flat-sea, un-updated; only the test uses it) → soften: "intended for the 4.6a-ii downcast
  sites; adoption pending runtime.rs warding."
- **intueri L3 (mod.rs:8):** dup "## What lives here" header (Strike 2's mod.rs rewrite) → merge.
- **exigere L3 (mod.rs:22-26):** pending-stones Stone 4.6 → cite the DESIGN path inline or rune.
- **excusare HOLDS-with-note (spawn.rs:269 secare rune):** core host-constraint HOLDS; the 4.6a-ii recv'
  follow-up ref rotted (4.6a-ii shipped without the recovery-contract doc) → land the "errors observed
  via channel close; recover via join()/close().wait_status()" note on the recv'/close' docs, or re-point.

## CONVERGENCE BAR (builder, 2026-06-07): L1+L2=0 — L3 are polish, NOT chased
The stamp/converged bar is **L1+L2=0** (+ clippy-in-home 0). L3 findings are recorded but
do NOT gate convergence — they asymptote (every doc carries another nit; driving L3→0 is the
diminishing-returns trap; warding terminates by judgment per [[feedback_runes_earned_through_combat]]).
Per-home loop: cast vigilia → sweep L1+L2 (fold cheap adjacent L3 only if free) → ONE
confirmation re-cast → L1+L2=0 → STAMP → next home. NOT a multi-round L3-chase (the kernel/
4-round marathon over-applied; do not repeat for channel/ comms/ process/).

## SCOPE BOUNDARY (NOT 6.w — affirmed, not banked)
- The Phoenix flat-sea migration (runtime.rs 29k / check.rs 19k / freeze / edn_shim
  / load / io / lexer …) is a SEPARATE named campaign. 6.w wards only what 214
  built/touched (channel/ process/ kernel/ + comms/-drift). The INSCRIPTION names this affirmatively.

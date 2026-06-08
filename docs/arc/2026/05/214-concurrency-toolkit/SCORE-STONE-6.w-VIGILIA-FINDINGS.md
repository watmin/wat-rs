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
1. **excusare** — IN FLIGHT at curare (agent a25c4fa61b2f4becd). Collect its
   rune-verdicts; fold into the sweep.
2. **circumspicere** — NOT YET CAST (it is the perimeter, cast LAST with the
   full inward map). Cast it over channel/+process/+kernel/ with these findings
   embedded; add its perimeter findings to the sweep.
Then: draw ONE convergence sweep (sonnet), execute, re-verify L1+L2=0, lay the
TRIPLE vigilatum (channel/ + process/ + kernel/ each get the stamp).

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

## SCOPE BOUNDARY (NOT 6.w — affirmed, not banked)
- The Phoenix flat-sea migration (runtime.rs 29k / check.rs 19k / freeze / edn_shim
  / load / io / lexer …) is a SEPARATE named campaign. 6.w wards only what 214
  built (channel/ process/ kernel/). The INSCRIPTION names this affirmatively.

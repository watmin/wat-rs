# BRIEF — the `spawn-program'` OUTCOME WALL (peer-lifecycle Strike 5 — the LAST wall, the landing)

> **The work in one paragraph.** `spawn-program'` (and its prime siblings) is the last peer verb still
> returning a bare concrete `Thread'`/`Process'` and RAISING its *creation* failures. This strike does two
> weldings at once: (1) **thread/process/remote all become one `Peer'<I,O>`** — the unification the rest of
> the substrate already has (connect'/accept'/select'/poll' are all `Peer'`; spawn is the lone outlier); and
> (2) the creation act returns a matchable `:wat::kernel::SpawnOutcome<I,O>` — `Spawned[peer]` +
> the transport-general world-fault arms — so a masked spawn failure is structurally unrepresentable. Named
> by intueri-cast (ratified): `SpawnOutcome` is RECLAIMED for creation; the arc-060 join-result death value
> is renamed to **`Demise`**. This is the campaign's landing (`VNDE ORTVM` — arc 170 opened to solve IPC).

## The name set — RATIFIED (intueri-cast + builder), do not re-fork
```clojure
;; CREATION (reclaims the SpawnOutcome name) — Impure + parametric; Spawned holds a live Peer'.
;; the exact TWIN of ConnectOutcome<S,R>, with one creation-specific arm (Exhausted) the dial verbs lack.
(:wat::core::defenum :wat::kernel::SpawnOutcome<I,O> :wat::enum::Impure
  :Spawned  [peer <- :wat::kernel::Peer'<I,O>]   ;; thread · process · remote — ALL one Peer' (the unification)
  :Exhausted[cause <- :wat::kernel::Failure]     ;; OS/host refused to ALLOCATE the unit (thread EAGAIN / fork
                                                 ;;   ENOMEM / remote no-capacity). back off + retry the SAME target.
  :Refused  [cause <- :wat::kernel::Failure]     ;; unreachable / no listener at the coordinate. retry, target may appear.
  :Rejected [cause <- :wat::kernel::Failure]     ;; identity/auth (mTLS / peer-cred). NOT retryable.
  :Failed   [cause <- :wat::kernel::Failure])    ;; transport/wiring io (pipe-pair / socket-wrap / handshake).

;; TERMINATION (the arc-060 join-result value, RENAMED off SpawnOutcome + arms harmonized to the manner-of-ending).
;; a Rust-internal enum (value/value.rs:1093), the value a spawned unit sends on completion, consumed by
;; join-result -> Result<(), DiedError-chain>. NOT wat-facing; a pure Rust rename.
;;   SpawnOutcome::{Ok(v), RuntimeErr(e), Panic{message,assertion}}  →  Demise::{Returned(v), Errored(e), Panicked{message,assertion}}
```
Refused/Rejected/Failed **reuse ConnectOutcome's exact vocabulary** for the identical concepts (family harmony —
the caller who handles a dialed peer's failures handles a spawned peer's the same way); `Exhausted` is the one
genuinely-new creation kind (connect' has no word for "the OS refused to make the unit itself"). All five ride
EVERY spawn call (the locus is a runtime choice, so exhaustive-match forces handling each — R52's universal-
variant doctrine, the "verbosity is the shield").

## Read in order (the rooms)
1. **The connect' strike `1e7065a2`** (`git show 1e7065a2`) — the immediate twin. Copy its shape: `ConnectFail`
   inner-Result, `connect_as_value` conversion seam, the four `connect_outcome_*` builders, `infer` + must-use.
   This whole strike mirrors it, plus the Demise rename + the return unification.
2. `src/value/value.rs:1093` — the arc-060 `SpawnOutcome` enum (`Ok/RuntimeErr/Panic`) + `:1139` its
   `Receiver<SpawnOutcome>` — the rename target (→ `Demise`). `src/value/mod.rs:45` re-export.
3. `src/runtime.rs` — the 26 `SpawnOutcome::` refs (the join-result eval: `eval_kernel_process_join_result`
   ~`:22071`, `Thread/join-result` ~`:22564`, the `SpawnOutcome::Ok/RuntimeErr/Panic` matches ~`:22105-22200`,
   the construction ~`:22516`). All rename to `Demise::Returned/Errored/Panicked`.
4. `src/types.rs` — register the NEW `:wat::kernel::SpawnOutcome<I,O>` (Impure, parametric), mirroring the
   `ConnectOutcome` registration just landed (~`:1356`). Note: the concrete `Thread`/`Process` structs
   (`:1527/:1575`, carrying `stdin/stdout/stderr` + a `join` `ProgramHandle`) are the FUSED types the unification
   decomplects — GROUND whether they survive (as the runtime opaque backing) or fold.
5. `src/check.rs` — the concrete-return sites: `infer_spawn` (`:11556` → Process'), `infer_spawn_thread_prime`
   (`:11601` → Thread'), `infer_spawn_process_prime` (`:11664` → Process') + the runtime head-builders
   (`runtime.rs:23387/23391`). These return `SpawnOutcome<I,O>` (Spawned arm = `Peer'<I,O>`). + `MUST_USE_PARAMETRIC_HEADS`
   gains `"wat::kernel::SpawnOutcome"`; `push_must_use_error` gains a spawn' branch.
6. `src/kernel/spawn.rs` — the eval + the creation-fault raise sites to convert: thread `Builder::spawn failed`
   (`:696` → `Exhausted` if EAGAIN/limit else `Failed`), process `comms::process::pair (input/output/err) failed`
   (`~:768/778/794` → `Failed`; fork EAGAIN/ENOMEM → `Exhausted`). The `ThreadLaunch`/`ProcessLaunch` ctor
   `.expect()`s (`:714/888`) STAY raises (must-never-happen substrate bugs). Add `spawn_outcome_*` builders
   (`runtime.rs`, beside `connect_outcome_*`) via `message_only_failure`.

## STOP triggers (rejection criteria — halt + surface, do NOT improvise)
- **STOP-1 (SCOPE — ground FIRST, the load-bearing decision):** *which verbs* unify + get the wall. Grounded
  surface: primes `spawn-thread'`/`spawn-process'`/`spawn-program'` (`check.rs:5055/5063`, `infer_*_prime`) vs
  non-prime `spawn-thread`/`spawn-process`/`spawn-program`/`-ast` (arc 114/105, `:10350`/`:1212`). Determine the
  exact set that returns `Thread'`/`Process'` and should unify to `Peer'`; if a non-prime is legacy/retiring or a
  DIFFERENT surface, surface it — do NOT blanket-convert. (connect'/accept' precedent: only the prime peer verbs
  were walled.)
- **STOP-2 (the `join`/`Demise` seam):** the unified `Peer'<I,O>` return must NOT lose the join/reap capability
  that a spawned CHILD has and a dialed peer does not (join is a LOCAL ownership op; a `Peer'` over the wire can't
  be joined). GROUND how join-result reaches a spawned unit after the return is `Peer'` — it dispatches on the
  runtime opaque head (`THREAD/PROCESS_PEER_TYPE_PATH`, `runtime.rs:26062+`, `#[restricted_to]`), so the static
  type folds to `Peer'` while the kernel still reaps the local child. If a spawned `Peer'`'s join can't be reached
  post-unification, STOP — the reap/`Demise` must not be dropped (a lost death-reason is the exact no-hidden-
  failures sin).
- **STOP-3:** if `spawn.rs`'s creation faults can't be split cleanly into `Exhausted` (allocation) vs `Failed`
  (wiring io) — e.g. a single opaque io error lumps EAGAIN with pipe-fail — STOP, report the real error surface
  (maybe `Exhausted` folds into `Failed` for the thread/process tiers now, remote-only later). Do NOT guess the split.
- **STOP-4:** the concrete `Thread`/`Process` structs (types.rs:1527/1575) or their accessors are genuinely USED
  as the static type by a non-spawn consumer (not just the runtime opaque backing) → surface it before folding.

## The sweep (checker-scouted — NOT a grep) — the BIG one (~186 files touch a spawn verb)
Once `infer_spawn*` returns `SpawnOutcome`, build `target/release/wat` + run the floor / `--check` to find EVERY
site now facing an unfaced outcome. Face each: `(match (spawn-program' …) (SpawnOutcome::Spawned peer) …
(SpawnOutcome::Exhausted c …) (Refused c …) (Rejected c …) (Failed c …))`. Per-site: a fatal spawn where the
program can't start → `assertion-failed!` (the grounded 3-arg form `(assertion-failed! (Failure/message c) :None
:None)`, per connect'); in a retry/pool context → back off. **Recorded codemod** — sibling of
`wrap-connect-prime-in-connectoutcome.wat`; dry-run on `/tmp` copies + `diff` + `--check` samples + idempotency
before applying. **Atomic** — no green state where spawn returns the outcome but a site drops it. Trust the CHECKER
+ the floor, not the grep (the recv'/connect' lesson: the grep undercounts embedded-in-`forms` sites).

## The probe (RED-first)
`tests/comms/probe_arc278_spawn_outcome_wall.{rs,wat}` (mirror `probe_arc278_connect_outcome_wall`):
- happy spawn → `SpawnOutcome::Spawned[peer]` (peer asserted live — send'/recv' a round-trip through it).
- a forced creation failure → `Exhausted` or `Failed` if cheaply reachable (e.g. an rlimit-constrained spawn);
  else assert the eval mapping + say so (no faking — the connect'/close' precedent).
- structural `Value::Enum` asserts, never a loose `Debug`-string contains. RED before, GREEN after.

## Sequencing — RETIRE-FIRST (builder-ruled 2026-07-24: "kill what we came here to kill, then impl demise on what remains")
Rigging Demise onto the join-result NOW means renaming across the whole IPC/spawn set — including the ~81
`join-result`/`drain-and-join` sites on the non-prime we're about to DELETE. Don't patch the doomed. Kill the
non-prime first, then Demise + the wall touch only the surviving PRIME surface. The non-prime retirement is
BOUNDED (grounded — NOT a capability arc): stdout-text ≡ `recv'` (EDN value wire); a child's stderr/death ≡
`Lost` (recv'); no capability the prime lacks; ~5 consumers DIE, ~5–10 migrate as cheap `recv'`-drains, the
harness reimpls on the prime (`deftest'`/`run-hermetic'` already exist, R55). Each phase weighed by the
orchestrator's OWN `--release` re-run (Summary line, never a piped exit; unused-span lint green) before the next.

**PHASE 0 — kill the non-prime (the cascade; keep the floor green throughout):**
- **0a — flip the harness onto the prime** (biggest lever, touches ~0 of the 474 macro call sites): `deftest`'s
  body `run-thread`→`run-thread'`, `deftest-hermetic`'s `run-hermetic-with-prelude`→a prelude-aware `run-hermetic'`
  (add the prelude param to `run-hermetic'` — the one small addition the scout flagged); `TestResult` shape
  unchanged. The 474 `deftest`/`deftest-hermetic` call sites don't move.
- **0b — migrate the ~5–10 DIRECT channel/join callers** to the prime (`spawn-program'` + `recv'`-drain / thread
  self-peer). Hand-rewrite per site (NOT codemod — semantic reshape). (Scout list: counter-service/actor-proof
  thread+process, service-template, the two LRU CacheServices, wat_spawn_fn, arc112 send/recv, process-peer IPC.)
- **0c — delete the ~5 DIES-WITH-IT files** (feature-tests of the retiring `Process`-struct/raw-stdio, or
  already-ignored scaffolding): `spawn_process_stdio.{rs,wat}`, `spawn_process_stdin.{rs,wat}`,
  `probe_run_hermetic_ast_stdout_capture.{rs,wat}`, `ambient-stdio.wat`, `sandbox.wat`. (Confirm each has no live
  consumer before deleting — the whole-tree grep, R52.)
- **0d — retire the non-prime** verbs (`spawn-thread`/`spawn-process`) + the concrete `Thread`/`Process` structs +
  `Thread/Process/join-result`/`drain-and-join`/`stdin`/`stdout`/`stderr` accessors (RETIREMENT_TABLE in check.rs
  + eval removal), now zero callers. Floor green → **NON-PRIME DEAD, one prime spawn surface remains.**

**PHASE 1 — Demise** (now touches only the surviving prime join-result): rename the arc-060 `SpawnOutcome`
(value.rs:1093) → `Demise`, arms `Ok/RuntimeErr/Panic` → `Returned/Errored/Panicked`, across the ~surviving refs
(down from 32 once the non-prime path is gone). Floor green.

**PHASE 2 — the `SpawnOutcome` creation wall + the return unification** (on the clean prime family only): register
`:wat::kernel::SpawnOutcome<I,O>`, convert the prime `spawn-*'` eval creation-faults → the five arms, `infer_*_prime`
→ `SpawnOutcome` (Spawned arm = `Peer'<I,O>`), must-use, the prime-only corpus codemod sweep, the RED probe.
Atomic; floor green.

**Weigh + BANK each phase. When Phase 2 lands → the walls are WHOLE** (recv'/send'/poll'/close'/accept'/connect'/
spawn') → the `VNDE ORTVM` realization (BUILDER'S to voice).

## Weigh (the orchestrator re-runs; do NOT trust the report)
- the RED probe: RED before, GREEN after.
- **the floor: `cargo nextest run --release`, read the Summary line** (never a piped exit). Any OTHER new RED = a
  swallow site the scout missed → STOP-1. The unused-span lint MUST stay green.
- content-integrity: the diff is the Demise rename (value/runtime) + types.rs (SpawnOutcome register) + check.rs
  (infer + must-use) + spawn.rs (eval + builders) + the faced sweep sites + the codemod + the probe. Nothing else
  moved. Do NOT touch the recv'/send'/poll'/close'/accept'/connect' walls.

## Copy for shape
The `connect'` strike (`1e7065a2`) — the exact twin (it just landed). `BRIEF-connect-outcome-wall.md` for the
full pattern; `BRIEF-accept-outcome-wall.md` for the parametric-Impure-outcome shape.

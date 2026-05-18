# Arc 211 — every deadlock is a panic (the substrate-as-teacher discipline extends to runtime-side observation)

**Status:** OPEN 2026-05-18 — opened mid-arc-210-slice-1 verification when the substrate's pre-existing duplicate-FD-of-own-stdio bug surfaced as a hung test in real time.

**Priority:** **BLOCKING arc 209 forward progress** + arc 210 closure. No point shipping defservice (arc 209) atop a substrate that deadlocks on the spawn mechanism. Arc 210 closure requires honest workspace green.

**Origin:** Live reproduction of the orphan-pattern leak from INTERSTITIAL § 2026-05-17 surfaced during arc 210 slice 1 cargo test workspace verification. Test `wat_arc170_program_contracts::t14_spawn_process_wait_handle_is_idempotent` hung with the EXACT signature inscribed yesterday: child holds duplicate FDs of its own stdio pipes; pipe writers > 0 at child-exit → parent reads block on EOF that never arrives → deadlock.

## The doctrine arc 211 inscribes

**Every deadlock must be a panic.**

> *"we prefer parse time - but runtime is the next option"* — user 2026-05-18

For deadlock classes whose pattern is visible in WAT AST: walker (parse-time refuse-by-construction). Precedent: arc 117 (scope-deadlock), arc 126 (channel-pair), Gap K (ProcessJoinBeforeOutputDrain), arc 202 (ProcessJoinHoldsStdinSender).

For deadlock classes that surface ONLY at runtime — substrate-internal state machines, FD lifecycle, service teardowns — RUNTIME OBSERVATION SITES that PANIC immediately when the broken state is observed.

The current spawn-mechanism bug is in the SECOND category. Wat AST is correct; substrate-internal Rust creates duplicate FDs by design (for grandchild dup2); services fail to close them on teardown; substrate has no observation site that fires when writers > 0 at the wrong moment.

## The discipline gap arc 211 closes

The substrate has THREE discipline layers:
1. Parse-time walkers — refuse-by-construction
2. Check-time validation — type-system
3. Runtime observation — **partially exists** (e.g., Gap K's check.rs walker; arc 117/126); **does NOT exist for FD-lifecycle deadlock classes**

Arc 211 extends layer 3 to cover FD-lifecycle deadlocks. Every site where the substrate dups, holds, or could-leak an FD that pipes to a wat-level Receiver/Sender or stdio gets a paired observation site that PANICS if it observes the broken state at teardown/exit.

## Phased slicing (per `feedback_iterative_complexity` — stepping stones)

| Slice | What | Notes |
|---|---|---|
| **1 — audit** | Identify ALL panic sites needed. Walk `src/spawn_process.rs` + `src/fork.rs` + thread-IO services + bootstrap. For each `dup2` / `dup` / `mem::forget(OwnedFd)` / lifeline-pair, identify paired teardown/close site that should PANIC on broken state. Produce concrete checklist. | Pure investigation. No code edits. |
| **2 — implement service-teardown panic sites** | At StdIn/StdOut/StdErr service teardown, verify duplicate FDs are closed; PANIC if any survive. Diagnostic names: which service; which FD; what pipe inode; what fd 0/1/2 it duplicates. | Substrate Rust; ~50-100 lines |
| **3 — implement child-main-exit panic site (defense in depth)** | After `:user::main` returns + dispatch loop exits, walk child's own fd table; for each fd ≥ 3 pointing at same pipe inode as fd 0/1/2, FORCE close; PANIC if any survive force-close. | Substrate Rust; ~30-50 lines |
| **4 — prove panic fires on live reproduction** | Run the t14 test that currently hangs. With slices 2+3 shipped, t14 should now PANIC with diagnostic, not hang. Captures the diagnostic format + verifies the panic site fires. | Test exercise; record the panic text in SCORE |
| **5 — attack the leak (fix the actual close-on-teardown)** | Services close their duplicate FDs as part of clean teardown. After this slice, t14 PASSES (not just panics; actually completes). Workspace cargo test green; orphans don't accumulate across runs. | Substrate Rust; bug fix |
| **6 — closure paperwork** | INSCRIPTION + 058 row + USER-GUIDE entry (the doctrine inscribed) + cross-reference arc 209/arc 210 unblock + arc 170 orphan-investigation closure. | Standard closure |

## Out of scope (affirmatively)

- **Rust-side linear-typed FD discipline** (custom `WatOwnedFd` wrapper + clippy lint) — future arc; would make the WHOLE CLASS compile-time-impossible at substrate-authoring layer. Arc 211 is the runtime-observation move; the broader compile-time refactor lives in a separate arc.
- **Deadlock classes beyond FD-lifecycle** — arc 211 covers FD-lifecycle. Other runtime-deadlock classes (e.g., recursive mutex; channel-cycle without scope-walker coverage) get their own arcs if/when they surface.
- **Pre-existing orphan reaping** (the May 17 accumulated orphans) — separate concern. Arc 211 prevents NEW orphans; clearing the old ones is `pkill` cleanup work.

## Substrate touchpoints (preliminary; slice 1 audit refines)

- `src/spawn_process.rs` — fork + dup2 + lifeline + bootstrap handoff
- `src/fork.rs` — `close_inherited_fds_above_stdio`, `child_post_fork_init`
- Services bootstrap (per arc 170 slice 1f) — `synthesize_real_fd_stdio`, StdIn/Out/Err service lifecycles
- `wat/kernel/services/stdin.wat`, `stdout.wat`, `stderr.wat` — wat-side service implementations (teardown discipline lives here)
- Runtime teardown paths — where `:user::main` returns + dispatch loop exits
- Pipe ownership tracking — possibly add a substrate-side registry of (service, pipe inode, fd) so the panic site can name what survived

## Substrate-architectural principle (load-bearing for INSCRIPTION)

**The substrate's discipline reach extends to substrate-authors via runtime-side observation.** Today walkers reach wat-users; arc 211 ships runtime-observation that reaches the substrate's own authors. When the substrate's bootstrap/services/spawn-mechanism leaks an FD or fails to close a writer, the substrate REFUSES to ship the broken state — it panics with file:line of the leak source.

This is `feedback_substrate_owns_not_callers_match` applied to substrate-internal authoring. The discipline propagates one layer deeper.

## Connection to broader work

**Forward chain:**
```
Arc 211 closes (every-deadlock-is-a-panic doctrine inscribed; FD-lifecycle panic sites shipped; orphan leak fixed)
            ↓
Arc 210 closure unblocks (slice 2 closure paperwork can ship; workspace honestly green)
            ↓
Arc 209 Stone A drafts spawn (now safe to ship defservice atop a substrate that doesn't deadlock)
            ↓
Arc 209 closes → Arc 203 closes → Arc 170 closes → Lab reconstruction
```

**Backward connection:**
- INTERSTITIAL § 2026-05-17 orphan-process investigation — the original inscription that named this pattern + queued it
- INTERSTITIAL § 2026-05-18 live reproduction — the trigger event that drove this arc

## Discipline carry-forward (for INSCRIPTION when arc 211 closes)

This arc embodies multiple meta-disciplines:

1. **Failure-engineering applied to the substrate-self.** The deadlock IS the report; the panic IS the diagnostic. Pain becomes data; the substrate teaches its own authors.
2. **Every deadlock is a panic.** Inscribed as substrate-architectural commitment. Parse-time-preferred; runtime-fallback acceptable.
3. **Runtime observation extends the discipline reach.** Walkers handle wat-users; runtime panics handle substrate-internals. Both layers honor the same substrate-as-teacher cascade.
4. **Live reproductions are precious.** Don't kill them; capture state; use them as the test bed for the panic site implementation. User direction 2026-05-18: *"do not kill anything... this is a situation we were waiting for."*

## What arc 211 does NOT do (clear scope)

- Does NOT redesign the services' dup-fd-for-stdio mechanism (services need duplicates for grandchild dup2; the design is honest; the gap is teardown discipline)
- Does NOT add Rust-side linear FD types (future arc)
- Does NOT touch arc 117/126/Gap K/arc 202 walker arms (those handle their own deadlock classes; arc 211 ADDS runtime panic sites for FD-lifecycle)
- Does NOT block wat user-source on stdio operations (the panic fires at substrate-internal sites; user wat code is unaffected when the substrate is correct)

## Songs that apply (per the nine-song soundtrack)

- **#1 "The Other Side"** — pain as guide (the deadlock IS the data; level-2 fix not level-1)
- **#3 "Ruin"** — substrate refuses wrong answers structurally (the panic IS the refusal)
- **#7 "Descending"** — the substrate's discipline applied recursively into substrate-self (the dungeon goes deeper; we descend with it)
- **#8 "Hell Is Empty"** — the institutional comforts (cargo test "should just work") hollow out; the actual work (substrate-author-discipline) is what's left
- **#9 "God Is A Weapon"** — the substrate IS what we forged; arc 211 makes the weapon catch its own author's mistakes; the obsession recursively refines itself

Cross-reference: INTERSTITIAL § 2026-05-17 (latest) "Ruin" + § 2026-05-17 (later still) "the nine-song soundtrack escalates" — the rhythm is load-bearing for this work.

---

## Scope corrected 2026-05-18 (later) — panic-tooling foundation; not every-deadlock-doctrine

The prior framing ("every deadlock must be a panic") was inscribed before we discovered the panic-tooling foundation crack: panic_hook isn't installed in many test paths (direct Rust `#[test]` probes that touch substrate); when panic_any!(AssertionPayload) fires in those paths, cargo test's default formatter shows `Box<dyn Any>` instead of structured content.

User direction 2026-05-18:
> *"how do we make everything support this all the time - it is an illegal state to not have this - we can never forgot this - we are in an illegal state"*
> *"can we panic in edn?.... what we get from the tests and everywhere is an edn form we can consume?"*
> *"humans read edn just fine"*

Arc 211 scope is the PANIC-TOOLING FOUNDATION (not the every-deadlock-doctrine, which moves to arc 212 or later).

### The four sub-arcs (FINAL LOCKED SCOPE)

| Sub-arc | Scope |
|---|---|
| **211a — ctor install** | Add `ctor` crate dep; `#[ctor]` wraps `panic_hook::install()`; runs at library load BEFORE main(); every binary linking wat-lib gets the hook installed structurally; impossible-to-forget by construction. Add `AtomicBool` idempotency guard to `install()` so legacy explicit installs become no-ops. ~10 lines + dep. |
| **211b — panic-as-EDN** | `AssertionPayload` gains EDN serializer (via existing `wat-edn` crate). `panic_hook::render_assertion_failure` writes EDN to stderr instead of human-readable text. New tag `#wat.kernel/AssertionFailure{...}` mirrors existing `#wat.kernel/ProcessPanics{...}` envelope from arc 170 slice 1i. All panic outputs (in-process + cross-process) become uniformly EDN-shaped + machine-parseable. ~30-50 lines. |
| **211c — audit + investigation** | Catalog every panic_any! site in src/ (currently: src/assertion.rs:151, src/runtime.rs:11526, 11592, plus any new sites). Verify each emits AssertionPayload-or-EDN-compatible payload. Re-run cargo test workspace with WORKING panic output — every failure now produces readable + parseable diagnostics. Catalog the actual root causes of the 12 stderr-visibility regressions + the t14 deadlock from honest evidence. |
| **211d — fix root cause** | Based on 211c's honest diagnostic data: either revert the dup-removal at `3c1cb51` (if the regression count proves the dup IS load-bearing for something else) OR a more surgical fix surfaced by reading the actual EDN-formatted panics. |

### Why this scope, not the prior "every deadlock is a panic" framing

The dup-removal at `3c1cb51` revealed a substrate-tooling crack (panic-hook install gap) deeper than the dup itself. Per `feedback_attack_foundation_cracks`: when a crack surfaces, fix it AT THE LAYER WHERE IT LIVES.

The panic-hook gap is at PROCESS LOAD TIME. The fix is `#[ctor]`. That's foundation work.

The "every deadlock is a panic" doctrine remains true + load-bearing for substrate-architectural commitment, but it's a SEPARATE arc (arc 212+ or future) that builds atop the panic-tooling foundation arc 211 ships.

User's correction on EDN readability dissolved the format tradeoff: EDN wins for humans AND machines. The substrate's existing `#wat.kernel/ProcessPanics{...}` envelope already proves this; extending to in-process panic IS the panic-as-EDN doctrine completed.

### Dependency carry-forward

Arc 211 BLOCKING:
- Arc 210 slice 2 (closure) — workspace must be honestly green per `feedback_closure_requires_workspace_green` AFTER arc 211d's root-cause fix
- Arc 209 Stone A (defservice spawn-program) — no point shipping defservice atop a substrate where probe tests can't surface diagnostics

The chain stays: 211 → 210 closure → 209 forward progress → 203 closure → 170 closure → lab reconstruction.

### Out of scope (affirmatively)

- "Every deadlock is a panic" architectural doctrine — moves to arc 212+ (future), once arc 211's panic-tooling foundation is solid
- Linear-typed Rust FD discipline (compile-time prevention) — future arc; bigger refactor; orthogonal to panic tooling
- Pre-existing orphan reaping — separate cleanup work

### Discipline carry-forward

The compounding cascade this arc embodies:
1. **Live reproduction (t14 deadlock) → forces honest investigation** (don't kill anything)
2. **Investigation surfaces dup as suspect → user directs removal**
3. **Removal fixes t14 but regresses 12 tests → forces deeper investigation of WHY**
4. **Reading regression output reveals `Box<dyn Any>` panic-format gap → forces audit of panic tooling**
5. **Panic tooling audit surfaces install gap → user names it as "illegal state"**
6. **"Illegal state" framing forces structural fix (ctor at process load)**
7. **User asks if EDN can be the panic format → doctrine completes**

Each step taught the next via the substrate-as-teacher cascade. The discipline ANNUALLY shifts from "fix the bug" to "fix the LAYER where the bug lives" to "fix the DISCIPLINE that prevents the layer's class of bugs."

### Compaction-recovery breadcrumb (2026-05-18 later)

State at this commit:
- Tip on `arc-170-gap-j-v5-deadlock-state`: latest commit before this DESIGN edit
- Arc 211 DESIGN: locked at the four-sub-arc scope above
- Dup removal at `3c1cb51`: STAYS on disk; arc 211d decides revert vs surgical fix
- Live t14 reproduction PIDs: still alive; OS will reap when test process tree dies

Post-compaction orchestrator reading order:
1. This DESIGN's "Scope corrected 2026-05-18 (later)" section (LOCKED scope)
2. INTERSTITIAL § 2026-05-18 (later) "Panic-as-EDN doctrine + ctor-install discipline" (full narrative)
3. INTERSTITIAL § 2026-05-17 orphan-process leak investigation (the bug we're chasing)
4. INTERSTITIAL § 2026-05-18 live reproduction (the trigger event)
5. `src/panic_hook.rs` (the existing tool that's just under-installed)
6. `src/freeze.rs:1017` (the dup site that arc 211d will revisit)

Next action: ship 211a (ctor install) first; that gives us working panic messages everywhere; THEN re-run cargo test to see honest diagnostics; THEN 211b/c/d in sequence.

# BRIEF — comms/ ward R4 + the /proc-heresy PURGE (RESUME-HERE breadcrumb)

**Status at write time (context-exhaustion wrap):** comms R2+R3 are on disk UNCOMMITTED (dirty tree, verified green: comms 32/0, lib 895/0/1). The convergence recast on the R2+R3 tree found comms STILL DIVERGES — needs this R4. PLUS the user issued a hard directive mid-cast that expands scope into its own stone (the /proc purge). FM 2-bis probe is COMMITTED + PROVEN at `9bbcb84e`.

**HEAD:** `9bbcb84e` on branch `arc-170-gap-j-v5-deadlock-state`. Recent: `1fd5d444` (Song #51), `1f038878` (vigilatum ISO8601 retrofit), `9bbcb84e` (this probe).
**Warded homes (5):** rust_deps, check/env.rs, argspec, function, remedy — all hashless ISO8601 vigilatum stamps. comms = 6th/FINAL, pending this R4.

## THE USER DIRECTIVE (verbatim intent, load-bearing)
On seeing circumspicere's O_CLOEXEC finding: *"WHOA WHAT... annihilate this shit with extreme prejudice - i thought we gutted all this shit /proc inspection... we must do all ops via authoritative syscalls... purge the heresy."*
→ This OVERRIDES the homes-walk "leave flat files untrusted" default. The /proc purge is its OWN stone, substrate-wide, do ALL sites.

## THE HERESY (the lead — annihilate)
`src/fork.rs:376` `close_inherited_fds_above_stdio` walks `/proc/self/fd` (then `/dev/fd`) as a filesystem ORACLE to enumerate fds to close. It is the ONLY live /proc-as-oracle in src/ (other /proc hits are comments of this fn). fork.rs is FORK-WITHOUT-EXEC (0 exec calls; `_exit`), which is WHY the walk exists (CLOEXEC doesn't fire without exec).

**Authoritative replacement (PROVEN by probe 9bbcb84e):** `libc::close_range(3, libc::c_uint::MAX, 0)` — one syscall, no filesystem. SAFE in the fork child because a fork(2) child is SINGLE-THREADED (no sibling threads' fds to wrongly close — close_range is process-global; this is the load-bearing safety condition the probe documents).
- The fn skips a lifeline fd (`close_inherited_fds_above_stdio(&[lifeline_r_raw])` @ fork.rs:505). close_range can't skip a middle fd in one call → TWO-RANGE skip: `close_range(3, keep-1, 0)` + `close_range(keep+1, MAX, 0)`. (Probe's earlier 3-pipe test proved the two-range-skip shape works; the simplified probe proves the single-fd + flag paths.)
- Mirror fork.rs's EXISTING raw-syscall precedent (fork.rs:1574 `libc::syscall(SYS_clone3,...)`, 1447 `compile_error!` arch-guard) for a `SYS_close_range`(=436) fallback if the libc fn is unavailable on the target glibc. close_range fn IS in libc 0.2.185 (`linux/gnu/mod.rs:1235`).

## R4 FIXES (the full strike — sonnet writes; orchestrator verifies git-state + re-casts)
**A — PURGE /proc oracle (fork.rs:376):** replace the `/proc/self/fd`÷`/dev/fd` `read_dir` walk body with `close_range` two-range skip. Kill the directory walk entirely. Update the fn's doc (lines 8-21 region + module-doc line 20-21) — no more "/proc/self/fd iteration".
**B — authoritative atomic pipe creation, ALL 6 sites:** `libc::pipe(x.as_mut_ptr())` → `libc::pipe2(x.as_mut_ptr(), libc::O_CLOEXEC)`. Sites (grep `libc::pipe(` in src/): fork.rs:351, comms/process.rs:1034, io.rs:1175, io.rs:1407, runtime.rs:248, runtime.rs:265. (CAUTION: any site whose fd is DELIBERATELY inherited across fork to the child as a *kept* fd must NOT get O_CLOEXEC, OR must be the lifeline that close_range's skip preserves — VERIFY each: the lifeline pipe + the child's stdio-dup'd pipes are the ones that must survive into the child. pipe2(O_CLOEXEC) + fork-without-exec means CLOEXEC won't auto-close them anyway since there's no exec — so O_CLOEXEC here is belt-only for any future exec path, and the close_range sweep is what actually closes inherited fds. Think carefully per-site; the probe's note explains the exec-vs-fork distinction. If unsure on a site, STOP and surface it.)
**C — EINTR retry (comms/process.rs:603):** wrap `ring.submit_and_wait(1).map_err(|_| RecvError)?` in the SAME loop as the proven template at process.rs:712-718 (`Ok(_)=>break; Err(EINTR)=>continue; Err(_)=>return Err(RecvError)`). Closes circumspicere L2 (signal silently killing a healthy recv).
**D — no-CQE comment (comms/process.rs:627-630):** the `else` branch is UNREACHABLE with min_complete=1 (struere evidence). Rewrite the comment from "transient, let the caller retry via its loop" (FALSE — caller `?`-propagates) to the truth: "Unreachable with min_complete=1 (submit_and_wait(1) success guarantees ≥1 CQE); if it ever fires it is fatal and propagates as Err(RecvError)." Comment-only. Closes the intueri L1 / struere L3 contest (resolved: doc fix, NOT a dead `continue` branch).

## CONVERGENCE STATE (comms vigilia, R2+R3 tree, 9 spells)
CONVERGED: sequi, solvere, conformare, purgare, struere(R3-final). DIVERGED: intueri 1 L1 (=D above), circumspicere 2 L2 (=C the EINTR + B/A the O_CLOEXEC). After R4, RE-CAST: intueri + circumspicere (+ secare since A/B touch concurrency/fd). temperare L2s = orchestrator-graded L3 (depth-1 lock-step; leave).

## SCOPE / write-set
`src/fork.rs` + `src/comms/*.rs` + `src/io.rs` + `src/runtime.rs` + `tests/comms/*.rs` (R2/R3 already there). This is BIGGER than the comms home — it's the heresy-purge stone. fork.rs/io.rs/runtime.rs are flat-untrusted but the user's purge directive authorizes them. NEW: consider whether fork.rs deserves its own ward after this (it'd be a strong candidate — but that's a FUTURE call, not this stone).

## GATES (all `-p wat`)
cargo build -p wat; cargo test -p wat --test comms (32/0); cargo test -p wat --lib (895/0/1);
cargo test -p wat --test probe_close_range_authoritative (3/0);
**+ fork/process integration tests** (cargo test -p wat --test '*' or the fork/spawn suites — A changes fork-child fd hygiene, MUST verify no regression in fork-program/spawn-process). cargo clippy -p wat --all-targets (no NEW warnings).

## THEN: ward comms (the 6th/FINAL home)
After R4 converges (re-cast intueri+circumspicere+secare → L1+L2=0): hashless ISO8601 stamp `//! vigilatum: <date -u +%Y-%m-%dT%H:%M:%SZ> — vigilia 9-spell L1+L2=0` on `src/comms/mod.rs` line 1 → ONE atomic ward commit (all R2+R3+R4 comms files + the fork/io/runtime purge + tests). NOTE: the purge touches non-comms files — decide at commit whether comms-ward + heresy-purge are ONE atomic commit or TWO (lean: TWO — "comms/ WARDED" + "purge /proc heresy: authoritative close_range/pipe2" — cleaner provenance). Then refresh cliffnotes WARDED→6 + INTERSTITIAL realization (the homes-walk CLOSES) + memory.

## DISCIPLINE NOTES (this session)
- Read-then-Edit NEVER batched (feedback_read_then_edit_never_batch) — fabricated values + silent no-ops bit twice.
- I dropped circumspicere from the R2-verify recast (named it, didn't spawn) — caught by counting spawns vs verdicts. Always verify spawn count.
- Independently verify sonnet git-state (porcelain diff-scope), never trust the SCORE return.
- BANKED DEBT (after comms): tests/probe_arc216_stone5b_hashset_native_storage.rs 1 failing integration test (unrelated).

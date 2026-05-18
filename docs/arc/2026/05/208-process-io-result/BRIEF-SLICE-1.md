# BRIEF — Arc 208 Slice 1: substrate audit + Result flip on Process/readln + Process/println

**Predecessors:** Arc 207 SHIPPED at `ec1e2c5`; arc 203 DESIGN updated at `f67f7ac` consolidating demands. Arc 208 DESIGN at this slice's parent dir.

**Scope:** flip `:wat::kernel::Process/readln` + `:wat::kernel::Process/println` substrate verbs to Result-returning. Mirrors arc 110/111's thread-tier Result flip at process tier. One slice for both verbs because the shape is settled by precedent — no separate audit-vs-flip split needed.

## Verification gate (sonnet's first action)

1. **Baseline.** `git status --short` clean (only `.claude/worktrees/`). `cargo test --release --workspace --no-fail-fast 2>&1 | grep FAILED` records baseline. Expected: 3-4 pre-existing failures (`lifeline_pipe`, `tmp_totally_bogus`, `t6_spawn_process`, `startup_error_exit_3`).
2. **Confirm substrate state.** Read these files + cite file:line for current signatures:
   - `src/check.rs:13402-13455` — `Process/readln` + `Process/println` type scheme registrations
   - `src/runtime.rs:4782+4785` — dispatch arms
   - `src/runtime.rs:17974+18039` — eval handlers (current panic-on-disconnect shape)
   - `src/runtime.rs:18093+` — shared peer-struct unwrap helper
3. **Grep ProcessDiedError usage.** `grep -n "ProcessDiedError" src/types.rs src/runtime.rs src/check.rs` to confirm the substrate-vended error type + accessors.
4. **Grep arc 111 precedent shape.** `grep -B 2 -A 10 ":wat::kernel::Sender/send\|:wat::kernel::Receiver/recv" src/check.rs | head -40` to confirm the exact Result-wrapper pattern at thread tier; mirror it at process tier.
5. **Grep all Process/readln + Process/println consumers.**
   ```
   grep -rn "Process/readln\|Process/println" --include="*.wat" --include="*.rs" . 2>/dev/null | grep -v "/target/" | grep -v ".claude/" | grep -v "src/runtime.rs\|src/check.rs"
   ```
   Surface every wat-side caller and Rust-side reference. Each becomes a slice-2 ripple target.

## Sub-decision required: Process/readln return type

`Result<:I, :Vector<ProcessDiedError>>` vs `Result<:Option<:I>, :Vector<ProcessDiedError>>` (mirror Receiver/recv's Option-wrapping).

Thread-tier `Receiver/recv -> Result<Option<T>, Vector<ThreadDiedError>>` uses `Option` to discriminate clean-channel-close (None) from value-received (Some). Process tier: when child closes stdin cleanly, does the parent's `Process/readln` see "clean close" distinguishable from "subprocess crashed"?

Sonnet audits this. Reasonable default: `Result<:I, ...>` — at the process tier, clean stdin close IS subprocess death (subprocess can't read after exit; lifeline pipe + drain mechanism handle the rest). If the substrate genuinely distinguishes "clean stdin EOF" from "subprocess panic," the Option-wrapper is honest. If not, plain `Result<:I, ...>` is honest.

**Surface the decision in SCORE row B with the four-questions verdict.** Don't pre-decide; let the audit settle it.

## Required code changes

After audit settles the readln shape:

1. **`src/check.rs:13402-13455`** — update both type scheme registrations to Result-returning. `Process/println`: `Result<:nil, :Vector<ProcessDiedError>>`. `Process/readln`: `Result<:I, ...>` or `Result<:Option<:I>, ...>` per sub-decision.

2. **`src/runtime.rs:17974+` (eval_process_readln) + `:18039+` (eval_process_println)** — rewrite handlers. Where today they `panic!`/`Err(RuntimeError)` on subprocess death, return `Ok(Value::wat__core__Result(Box::new(Err(...))))` wrapping a `Vector<ProcessDiedError>` Value. Where today they return raw `:I` or `:nil`, wrap in `Ok(...)`.

3. **No `src/runtime.rs:4782+4785` dispatch arm changes** likely needed — same verb names; same arities; just signature semantics flip. Sonnet confirms.

4. **Tests** — new file `tests/wat_arc208_process_io_result.rs` with:
   - Happy path: spawn-process + Process/println(Ok) + Process/readln(Ok) + drain-and-join
   - Err path: subprocess panics mid-stream → Process/println on dead peer returns `Err(Vector<ProcessDiedError>)` with structured chain; Process/readln on dead peer same
   - Cross-verify: chain content matches what `Process/drain-and-join` reports for the same failure
   - At least 5 test cases covering both verbs + Ok + Err + chain-content

## Conditional walker rule (likely OUT of slice 1)

Arc 110 minted "silent kernel-comm illegal" walker at thread tier to catch send/recv outside match-or-expect. Per DESIGN line 67-72: process tier today PANICS (loud, not silent), so the walker's purpose differs. After arc 208's Result flip, process I/O outside match-or-expect would silently discard the Err — same shape arc 110's walker catches at thread tier.

**Sonnet's call after Result flip lands:** is the walker rule needed in slice 1 (paired with flip) or slice 2 (after consumer ripple)? If trivial to mirror arc 110's walker mechanism for `Process/readln`+`Process/println` callsites, include in slice 1 for atomic substrate honesty. If the walker has its own complexity (e.g., different arity, different ProcessPeer extraction), defer to slice 2. Surface decision in SCORE.

## HARD constraints

- DO NOT touch `crates/wat-edn/`, `crates/wat-telemetry/`, or arc 203 demos. Consumers ripple in slice 2.
- DO NOT amend arc 110/111/112 INSCRIPTIONs (immutable per `feedback_inscription_immutable`); cross-reference, don't modify.
- DO NOT touch `Process/drain-and-join` or `Process/join-result` — already Result-returning correctly.
- DO NOT touch `Process/stdin`/`stdout`/`stderr` accessors (not I/O verbs; not in scope).
- DO NOT commit; orchestrator commits atomically after verification.
- DO NOT use `--no-verify` / `--no-gpg-sign`.
- cwd `/home/watmin/work/holon/wat-rs/`; never `.claude/worktrees/`.

## STOP triggers

1. **`Process/readln` substrate has a clean-EOF vs panic discrimination mechanism** that makes Option-wrapping load-bearing — surface; orchestrator decides shape.
2. **Workspace baseline regresses** beyond 4 pre-existing failures.
3. **`ProcessDiedError` substrate type doesn't compose into the Result wrapper** (e.g., wrong Value variant; missing Vector constructor) — surface; substrate prerequisite slice needed.
4. **Consumer grep surfaces wat or Rust callsites > 5** that would break under signature flip and cannot be cleanly defer'd to slice 2 — surface; orchestrator decides scope adjustment.
5. **Walker rule mirroring arc 110 has substantive complexity** (not just file:line patch) — surface; orchestrator decides whether to absorb in slice 1 or slice 2.

## SCORE methodology

`docs/arc/2026/05/208-process-io-result/SCORE-SLICE-1.md` with these rows (atomic YES/NO):

| Row | Evidence |
|---|---|
| A — Verification gate passed (5 checks: baseline, substrate state file:line, ProcessDiedError grep, arc 111 precedent grep, consumer grep) | Each check + result inscribed |
| B — Process/readln return-type sub-decision settled (Result<:I,...> vs Result<:Option<:I>,...>) with four-questions verdict | Decision + audit rationale inscribed |
| C — `Process/readln` flipped to Result-returning per decision | Type scheme + eval handler diff inscribed |
| D — `Process/println` flipped to Result<:nil, :Vector<ProcessDiedError>> | Same |
| E — New tests `tests/wat_arc208_process_io_result.rs` cover happy + Err paths + chain content | Test cases listed; cargo test passes |
| F — Workspace baseline preserved (≤4 pre-existing failures); no new failures introduced | cargo test output |
| G — Walker rule decision (in slice 1 vs deferred to slice 2) made + rationale inscribed | Decision noted |
| H — Consumer count from grep documented (for slice 2 BRIEF planning) | Numbered list of callsites |

## Time-box

Predicted 60-90 min sonnet. Hard stop 120 min. Substantive substrate work but precedent-shaped — arc 111's pattern is the template to mirror.

## On completion

Return summary: rows passed/failed, file:line for substrate flips, sub-decision verdict, walker decision, consumer count, any honest deltas.

T-minus 0. Begin.

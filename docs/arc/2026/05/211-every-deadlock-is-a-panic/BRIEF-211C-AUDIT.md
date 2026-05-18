# Arc 211c — panic_any! audit + per-failing-target investigation

**Slice scope:** Pure investigation. Zero code changes. Catalog every `panic_any!` site in the substrate; for each of the 11 currently-failing test targets, capture the NOW-STRUCTURED-EDN panic output (post arc 211a+b ships), diagnose the root cause from honest evidence, categorize, produce a concrete 211d worklist.

**Origin:** arc 211 DESIGN § "Scope corrected 2026-05-18 (later)" sub-arc 211c. Per the locked sequence, 211c reads the diagnostics that 211a (auto-install) + 211b (EDN format) made readable; 211d acts on the audit's findings.

**Closes:** the diagnostic gap. Before 211a+b, the 11 failing targets emitted `Box<dyn Any>` placeholders OR human-readable text (depending on which path triggered). After 211a+b, every AssertionPayload panic emits structured EDN. 211c reads those EDNs and tells us what's actually broken.

## Locked scope

**Investigation tasks (NO code changes; NO test edits):**

1. **`panic_any!` site catalog** — grep entire substrate + crates:
   - `grep -rn "panic_any" /home/watmin/work/holon/wat-rs/src/ /home/watmin/work/holon/wat-rs/crates/`
   - For each hit: file:line, the function it's in, the payload type passed
   - Note any sites that DON'T pass `AssertionPayload` — those won't get the structured rendering

2. **Per-failing-target investigation** — for each of the 11 targets:
   - `probe_lifeline_pipe_proof`
   - `probe_no_default_rust_panic_noise_on_stderr`
   - `probe_plain_panic_produces_structured_edn`
   - `probe_run_hermetic_no_deadlock`
   - `probe_runtime_err_stderr_visibility`
   - `probe_runtime_error_produces_structured_edn`
   - `test` (wat-lib's `wat::test!` macro path target)
   - `wat_arc113_cross_fork_cascade`
   - `wat_arc170_program_contracts`
   - `wat_run_sandboxed`
   - `wat_cli` (wat-cli crate's integration test)

   For each: run with timeout to avoid hangs, capture stdout + stderr:
   ```bash
   timeout 90 cargo test --release --test <name> 2>&1 | tee /tmp/audit-<name>.log
   ```
   Or for the `-p wat-cli --test wat_cli` form:
   ```bash
   timeout 90 cargo test --release -p wat-cli --test wat_cli 2>&1 | tee /tmp/audit-wat_cli.log
   ```

3. **Per-target diagnosis** — for each captured log:
   - What test names within the target fail?
   - What panic output appears? Is it the new `#wat.kernel/AssertionFailure{...}` EDN envelope, or something else?
   - Is the failure an assertion mismatch, a hang, a missing-symbol, an OOM, an actual deadlock?
   - Quote the relevant 5-10 lines of failure output verbatim in the SCORE

4. **Categorize each failure**:
   - **A: dup-removal regression** — failure caused by `3c1cb51` (synthesize_real_fd_stdio dup removal); fixable by revert OR surgical alternative
   - **B: pre-existing flake** — was flaking before arc 211; surfaces non-deterministically; not regression
   - **C: foundation issue** — broken for substrate reasons unrelated to arc 211 (e.g., the orphan-pattern leak under continued investigation)
   - **D: assertion-on-old-format** — probe/test asserts on the OLD text panic format; needs assertion update (NOT a real failure; expected after 211b's format shift)
   - **E: other** — something else; describe specifically

5. **211d worklist** — based on Category counts:
   - List concrete actions 211d should take
   - For Category A failures: provide enough detail to decide revert vs surgical fix
   - For Category D failures: list the assertion updates needed
   - For Category B/C: note as out-of-scope for 211d (pre-existing; separate arc if priority surfaces)

6. **Recommendation** — based on the audit's findings:
   - Should 211d revert the dup-removal at `3c1cb51`?
   - Should 211d ship a surgical alternative?
   - Should 211d update the probe assertions to EDN format?
   - Should 211d do some combination?
   - Surface the four-questions tradeoffs orchestrator should consider

## Output structure (the SCORE doc)

Write `/home/watmin/work/holon/wat-rs/docs/arc/2026/05/211-every-deadlock-is-a-panic/SCORE-211C-AUDIT.md` with these sections:

1. **Mode classification** (A/B/C/etc — A means audit complete)
2. **panic_any! sites catalog** (table)
3. **Per-target findings** (11 sub-sections, one per target; quote actual panic output)
4. **Category summary** (count per A/B/C/D/E)
5. **211d worklist** (concrete actions with file:line where possible)
6. **Recommendation for orchestrator** (revert vs surgical; assertion updates list; four-questions on the choice)

## Constraints

- **NO code changes** — this is pure investigation
- **NO test edits** — if a test needs an assertion updated, NOTE it for 211d; don't update it now
- **NO commits** — orchestrator commits the SCORE atomically after independent review
- **NO destructive ops** — `pkill` is OK between test runs to clean orphans; nothing else
- **DO use `timeout 90`** on each cargo test invocation — some targets (probe_lifeline_pipe_proof, wat_arc170_program_contracts t14) are deadlock-prone; bound the wall-clock to prevent multi-hour hangs
- **DO capture verbatim output** in the SCORE — paraphrasing loses the diagnostic value
- **DO note environmental observations** — orphan processes accumulated, pipe inode collisions visible in `/proc`, etc.

## Time prediction
45–60 min Mode A. 11 targets × ~3 min/target + 10 min cataloging + 15 min write-up. Sonnet may parallelize some test runs if confident none interfere.

## STOP triggers
Report and stop (do not work around) if:
- A target hangs past `timeout 90` AND the hang itself is the new diagnostic (some tests time-out as part of asserting deadlock-prevention; check the target's source first)
- The substrate panics during cargo build (panic_hook auto-install failure?) — would mean 211a is broken; surface immediately
- Catalog grep returns >50 `panic_any!` sites (way more than expected; might indicate the search isn't filtered correctly; verify scope)

## Decay disclosure (orchestrator hypotheses)

The orchestrator EXPECTS (but sonnet verifies):
1. **Two probes (`probe_plain_panic_produces_structured_edn` + `probe_no_default_rust_panic_noise_on_stderr`)** are probably Category D — assert on OLD format; will pass once probe-side assertions update to EDN. The probe NAMES literally describe what arc 211b just shipped; they were written to assert on the format that 211b just delivered structurally.
2. **`probe_runtime_err_stderr_visibility`** + **`probe_runtime_error_produces_structured_edn`** are probably Category D for similar reasons.
3. **`probe_lifeline_pipe_proof`** is Category B — pre-existing flake, noted by 211a SCORE.
4. **`probe_run_hermetic_no_deadlock`** might be Category A — the dup removal affected hermetic runs.
5. **`wat_arc170_program_contracts`** is mixed — t14_spawn_process_wait_handle_is_idempotent was the original live reproduction (Category A if dup-related; some other failures might be Category B).
6. **`test` + `wat_run_sandboxed`** — uncertain; probably mixed.
7. **`wat_cli`** — uncertain; probably foundation-issue or dup-removal.

Sonnet's investigation REPLACES these hypotheses with honest evidence. The hypotheses are starting points, not conclusions.

## Cross-references
- Arc 211 DESIGN § "Scope corrected 2026-05-18 (later)" — locked four-sub-arc scope
- SCORE-211A-CTOR-INSTALL.md — preceding slice (ctor install)
- SCORE-211B-PANIC-AS-EDN.md — preceding slice (EDN format shift)
- INTERSTITIAL § 2026-05-18 (later) "Panic-as-EDN doctrine" — the doctrine 211c reads against
- INTERSTITIAL § 2026-05-17 "Orphan-process leak investigation" — Category C territory for pre-existing leak
- `src/freeze.rs:1017` — the dup-removal site (`3c1cb51`); 211d's revisit point
- `feedback_no_speculation` — measure, don't theorize; 211c's job is the measurement
- `feedback_no_polling_loops` — `timeout 90` bounds wall-clock; harness notifies on completion

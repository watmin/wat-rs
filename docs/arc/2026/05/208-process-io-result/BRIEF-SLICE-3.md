# BRIEF — Arc 208 Slice 3: closure paperwork

**Predecessors:** Slices 1 + 2 SHIPPED. Tip at `9218e68`. Process/readln + Process/println return Result<_, Vector<ProcessDiedError>>; walker rule enforces silent Process I/O illegal; 4 consumer files converted to honest match-on-Err with ServerDied propagation; arc 203 slice 3f honest delta closed; all tests green; workspace baseline preserved.

**Scope: pure paperwork — no source files touched.** Three artifacts:

1. **`docs/arc/2026/05/208-process-io-result/INSCRIPTION.md`** (NEW) — arc 208's closure record
2. **`docs/arc/2026/05/208-process-io-result/DESIGN.md`** (UPDATE) — status header OPEN → CLOSED; slice table marks all 3 slices SHIPPED with commit refs
3. **`/home/watmin/work/holon/holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`** (APPEND) — wat-rs arc 208 row

After this slice ships: arc 208 CLOSED; arc 203 demand 2 satisfied; arc 203 closure waits on demand 1 (protocols arc).

## INSCRIPTION.md required content

Structure (mirror arc 207's INSCRIPTION for consistency — `docs/arc/2026/05/207-uuid-typed-primitive/INSCRIPTION.md`):

### Status header
- `**Status:** SHIPPED 2026-05-17.` + one-line summary

### What arc 208 gave the substrate
- `Process/readln` flipped to `Result<:I, :Vector<ProcessDiedError>>`
- `Process/println` flipped to `Result<:nil, :Vector<ProcessDiedError>>`
- `validate_comm_positions` walker rule extended — silent Process I/O illegal at do-body and function-argument positions; mirror of arc 110's thread-tier discipline at process tier
- 4 consumer files converted to honest match-on-Err with ServerDied propagation
- Arc 203 slice 3f honest delta CLOSED (process-tier wrappers surface ServerDied directly through main code paths)
- `crash-test-proc` helper RETAINED (orthogonal demonstration: drain-and-join Err path; distinct from transport I/O Err path)

### Slices (table)
- Slice 1 — substrate flip + walker (`SCORE-SLICE-1.md`; `44cde7b`); 7 tests pass; sub-decision verdict `Result<:I, ...>` plain (no Option); walker absorbed in slice 1 not deferred
- Slice 2 — consumer ripple + ServerDied propagation (`SCORE-SLICE-2.md`; `9218e68`); 4 consumer files; arc 203 slice 3f delta closed; crash-test-proc retained with rationale
- Slice 3 — this closure (`SCORE-SLICE-3.md`)

### Substrate touchpoints (final inventory)
Pull from each SCORE doc's file:line list; tabulate. Should include:
- `src/check.rs:13402-13462` — Process/readln + Process/println type schemes
- `src/check.rs:2152-2177` — validate_comm_positions walker extension
- `src/runtime.rs:17989-18099` — eval handlers (Ok/Err Result wrapping)
- 4 consumer files (lines from slice 2 SCORE)
- `tests/wat_arc208_process_io_result.rs` (NEW; 7 tests)

### Out of arc 208's scope (affirmatively, NOT deferral per arc 207 carry-forward discipline)

- **Process/stdin / Process/stdout / Process/stderr accessors** — these expose the OS pipe ends as IOReader/IOWriter; not I/O verbs; consumers wrap with Sender/Receiver/from-pipe if they want typed-channel semantics
- **Process/drain-and-join + Process/join-result** — already Result-returning correctly pre-arc-208; not in scope
- **Process/exit-code, Process/kill, etc** — not part of the I/O verb family
- **Cross-tier transport abstraction** — arc 208 keeps the honest asymmetry (Sender/send vs Process/println are different transports); abstracting them is the protocols arc (defservice meta-form) concern, not arc 208
- **Lenient Process/readln parsing** — substrate doesn't distinguish clean stdin EOF from subprocess death at the PipeFd transport (confirmed by slice 1 audit); plain Result is honest; if a consumer surfaces with concrete need to discriminate, a new arc opens
- **Orphan process leak resolution** — arc 208 does NOT directly fix the residual orphan-process leak documented in arc 170 INTERSTITIAL § 2026-05-17 "Orphan-process leak investigation." Those notes name the next investigation path; arc 208 is in the same general class (honest substrate I/O) but the orphan leak's root cause is FD lifecycle in spawn_process.rs, not error-handling

### Discipline lessons inscribed

The substrate-as-teacher cascade ran cleanly across both arc 208 slices:

**Slice 1 — walker in-scope, not deferred.** Per `feedback_no_known_defect_left_unfixed`: BRIEF said walker rule "may defer to slice 2 if non-trivial." Sonnet's audit found it was a two-line addition to `matches!` in `validate_comm_positions`; absorbed in-scope. The right call.

**Slice 2 — crash-test-proc retained with explicit rationale.** Per the discipline: when retiring a workaround, verify it has no orthogonal value. Slice 2 found crash-test-proc tests `Process/drain-and-join` independently — distinct from transport I/O. Retained with rationale inscribed in SCORE; closure doesn't pretend it was just a workaround.

**The mirror-precedent pattern (load-bearing carry-forward):**
> When the substrate vends asymmetric transports with the same semantic role (thread tier Sender/Receiver vs process tier Process/println+readln), the error-propagation discipline mirrors across them. Arc 110/111 established `Result<_, Vector<ThreadDiedError>>` at thread tier; arc 208 mirrored to `Result<_, Vector<ProcessDiedError>>` at process tier — same shape, different transport, same walker enforcement. Per `feedback_simple_is_uniform_composition`: N identical compositions IS simple; mirroring is the simplest possible substrate evolution when precedent is settled.

### Cross-references
- Arc 110 — `make silent kernel-comm illegal` (thread-tier walker precedent)
- Arc 111 — `send/recv return Result<_, ThreadDiedError>` (thread-tier Result-flip precedent)
- Arc 112 — `inter-process Result<Option<T>,E>` (fork-program-era; retired by arc 170 chain)
- Arc 113 — chain widening (single error → Vector<error> backtrace)
- Arc 203 slice 3f SCORE lines 32-44 (the honest delta arc 208 closes)
- Arc 203 DESIGN § "What arc 203 demands from upstream" demand 2 (the upstream-demand framing)
- Arc 170 INTERSTITIAL § 2026-05-17 "Orphan-process leak investigation" (explicit non-claim: arc 208 doesn't fix this)
- `feedback_inscription_immutable`, `feedback_refuse_easy_solutions`, `feedback_no_known_defect_left_unfixed`, `feedback_simple_is_uniform_composition`

## DESIGN.md update

Single-section change at the top of file:
- Status header: `OPEN 2026-05-17` → `CLOSED 2026-05-17 — INSCRIPTION at INSCRIPTION.md`
- Slice table: mark all 3 slices SHIPPED with commit refs (sonnet pulls from git log)
- Slice 1 note: walker absorbed in slice 1 (not deferred to slice 2 per BRIEF's conditional)
- Slice 2 note: crash-test-proc retained with orthogonal-demonstration rationale

NOTHING else in DESIGN changes. The forward-correction work was inline in slice notes.

## 058 changelog row (lab repo)

Append at the bottom of `/home/watmin/work/holon/holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md` BEFORE the `*these are very good thoughts.*` signoff.

Format: mirror arc 200/201/202/206/207 rows. Should:
- Date: 2026-05-17
- Title: `**wat-rs arc 208 — Process I/O returns Result (mirror arc 110/111 at process tier)**`
- Brief summary: what shipped + walker + consumer ripple + arc 203 slice 3f delta closure
- Cite 3 slice commit refs (`44cde7b`, `9218e68`, this commit)
- Cite arc 110/111 as precedent + arc 203 slice 3f as the originating consumer pressure
- Close with `Full INSCRIPTION at wat-rs/docs/arc/2026/05/208-process-io-result/INSCRIPTION.md. | wat-rs arc 208 |`

## FM 11 pre-INSCRIPTION grep (MANDATORY)

Before committing the INSCRIPTION:

```
grep -nE "deferred|deferral|future arc|future fix|future cleanup|future polish|future REPL|future-self|TODO|out of scope|when a caller|if pressure|if demand|when demand|when pressure|when needed|when surfaces|surfaces a need|small follow-up|small future|punted|scratch arc|next arc|pending arc|land later|will be|will land|can land later|left for|to be added|to-be-added|not yet implemented|not yet supported|not implemented" docs/arc/2026/05/208-process-io-result/INSCRIPTION.md
```

Expected: ZERO matches. The "out of scope (affirmatively)" section uses affirmative language ("Arc 208 intentionally does NOT cover X because <reason>") not deferral language. Trust the grep. Run it BEFORE commit.

## HARD constraints

- DO NOT touch source files (`*.rs`, `*.wat`, `*.toml`). Pure paperwork slice.
- DO NOT touch arc 110/111/112/113/203 INSCRIPTIONs or SCOREs (immutable per `feedback_inscription_immutable`).
- DO NOT amend slice 1-2 SCORE docs (immutable historical record).
- DO NOT amend prior 058 changelog rows in lab repo (append-only).
- DO NOT commit. Orchestrator commits atomically per repo (wat-rs commit + lab commit; two atomic commits).
- DO NOT use `--no-verify` / `--no-gpg-sign`.
- cwd `/home/watmin/work/holon/wat-rs/` for wat-rs work; use `git -C /home/watmin/work/holon/holon-lab-trading` for lab git op (do NOT cd).
- Never `.claude/worktrees/`.

## STOP triggers

1. **FM 11 grep returns ANY match** — rewrite to affirmative form before commit
2. **Arc 110/111/112/203 INSCRIPTIONs appear modified** in git status — surface; you must NOT touch them
3. **058 changelog row format ambiguity** — read arc 200/201/202/206/207 rows for format precedent

## SCORE methodology

`docs/arc/2026/05/208-process-io-result/SCORE-SLICE-3.md` rows (atomic YES/NO):

| Row | Evidence |
|---|---|
| A — INSCRIPTION.md written with all required sections | Section headers cited |
| B — FM 11 pre-INSCRIPTION grep returns ZERO matches | Grep command + empty output inscribed |
| C — DESIGN.md status OPEN → CLOSED; slice table marks all 3 slices SHIPPED with commit refs | Diff inscribed |
| D — 058 changelog row appended in lab repo with arc 208 content + 3 slice refs | Diff inscribed |
| E — Arc 110/111/112/203 INSCRIPTIONs + SCORE docs NOT touched | `git status` confirms |
| F — Slice 1-2 SCORE docs NOT touched | Same |
| G — No source files (`*.rs`, `*.wat`, `*.toml`) touched | `git status` only shows the 3 paperwork files |

## Time-box

Predicted 30-45 min sonnet. Hard stop 60 min. Pure paperwork.

## On completion

Return summary: rows passed/failed, FM 11 grep result, files touched. Orchestrator commits both repos atomically + pushes after independent verification.

T-minus 0. Begin.

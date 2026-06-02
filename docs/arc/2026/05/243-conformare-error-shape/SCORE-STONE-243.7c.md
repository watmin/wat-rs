# SCORE — Stone 243.7c — RuntimeError Pattern A — ATTEMPT 1: REJECTED (Mode B — catastrophic silent UTF-8 corruption)

**Verdict (orchestrator, 2026-06-01): REJECTED. Reverted to STRIKE-READY `8b51d93f`. Redo required with a UTF-8-safe surgical tool.**

## What happened

The Shadowdancer's structural reshape was CORRECT and every gate passed green:
- probe 7c 4/0 · probe 7b 4/0 (EvalBreak wrap intact) · lib **895/0/1** · `cargo build --release --tests` clean · clippy `result_large_err` 0 · `tools/` deleted · `pub struct RuntimeError`/`pub enum RuntimeErrorKind` minted.
- The agent's self-report claimed full success. **It was false-green.**

**The defect: the ephemeral tool (`tools/transform-runtimeerror`) silently DROPPED every multi-byte UTF-8 character it touched** — it round-tripped whole files through a UTF-8-lossy process. Measured corruption in `src/runtime.rs` alone:

```
non-ASCII chars: BEFORE 5727 → AFTER 7   (delta -5720)
dropped: — ×1583, ─ ×3525, → ×465, … ×19, ∀ ×24, × ×14, § ×18,
         ≈/≠/≡/≤/≥/∞/±/·/•/↔/↦/− and Greek α β γ δ Δ ι π σ — ALL gone.
```

It hit **doc comments too**, not just construction sites (`//! Runtime  AST walker` — the `—` dropped, leaving a double space) — i.e. the tool rewrote the ENTIRE file, not surgically. Every error message, EDN tag, type-scheme doc (∀, σ), and ASCII-art table (─) in the runtime is mangled.

**Why the gates missed it:** the substrate's test suite uses STRUCTURAL assertions (variants, spans, exit codes), not message-string matching (per the 241.2 calibration note). A dropped char inside a string literal still compiles and still passes structural tests. **Only a content-integrity scan catches it.** The 7 "empty char literal `''`" the agent caught were the compile-VISIBLE tip; the 5720 silent string/comment drops were invisible to cargo.

## How it was caught

Orchestrator scoring (FM 9 + content-integrity scan): `grep -oP '[^\x00-\x7F]' src/runtime.rs | wc -l` before (`8b51d93f`) vs after → 5727 vs 7. The histogram diff + double-space-mid-word check confirmed the drop mechanism.

## Recovery

`git checkout HEAD -- src crates tests` → restored to STRIKE-READY (non-ASCII back to 5728; RuntimeError back to enum; 7c probe red again; lib builds). My docs (DUNGEON-CRAWL doctrine, cliffnotes breadcrumb) preserved. No corruption committed.

## The useful map (preserved for the redo — the STRUCTURE was right)

Attempt 1's cascade was real: ~1050+ sites across `src/runtime.rs` (~746), `runtime_error_edn.rs`, io (36), time (32), string_ops (27), thread_io (23), fork (14), freeze (~12), marshal (~19), spawn (6), spawn_process (4), assertion (4), edn_shim (4), custodia (3), sandbox (1), hologram (1), function/eval (1), function/parse, argspec/error, lib.rs + `crates/wat-telemetry-sqlite/{auto,cursor}` + `crates/wat-macros/codegen` + 6 test files. The 2 multi-span (SandboxScopeLeak=call_span/outer_define_span; PostconditionFailed=body_span/ensure_span) + freeze-pair (UserMainMissing/EvalVerificationFailed = Span::unknown()) decisions were correct.

## Redo requirements (BRIEF-STONE-243.7c-REDO)

1. **UTF-8-SAFE SURGICAL tool:** read with `fs::read_to_string`; perform TARGETED replacements (exact-pattern `str::replace` / regex that preserves all other bytes); write with `fs::write`. **NEVER** iterate/filter/rebuild the file char-by-char; NEVER touch a line that has no construction site.
2. **Mandatory per-file content-integrity self-check** built INTO the workflow: for every file the tool writes, assert `non-ASCII-count(after) == non-ASCII-count(before)` (the transform is ASCII-syntax only; ANY non-ASCII delta = a corruption bug → STOP, fix the tool, re-run). Report the before/after counts per file in the SCORE.
3. Orchestrator re-scans the non-ASCII histogram on return (this is now a permanent scoring gate for tool-driven cascades).

## Cost

~58-min flight wasted. Zero corruption shipped. The catch validates the independent-scoring discipline and births a new doctrine (content-integrity scan for tool-driven cascades — `feedback_cascade_ephemeral_tool` + recovery-doc FM).

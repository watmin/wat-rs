# BRIEF — Stone 243.7c REDO — `RuntimeError` → Pattern A (UTF-8-safe)

**Attempt 1 was REJECTED** (Mode B — catastrophic silent UTF-8 corruption; see `SCORE-STONE-243.7c.md`). The structural reshape was CORRECT and all gates passed green — but the ephemeral tool round-tripped whole files through a char-by-char rewrite that **dropped 5720 non-ASCII chars** (—/→/─/∀/σ/… in comments, error messages, type-scheme docs, ASCII-art) from `runtime.rs` alone, invisible to cargo + the structural test suite. Reverted clean to STRIKE-READY `8b51d93f`.

**This REDO does the SAME structural transform** — read `BRIEF-STONE-243.7c.md` (the full structural contract: struct/Kind shape, 2 multi-span [SandboxScopeLeak=call_span/outer_define_span, PostconditionFailed=body_span/ensure_span], freeze-pair [UserMainMissing/EvalVerificationFailed = outer `Span::unknown()` elided], Display split, EDN collapse, the cascade fan-out, STOP triggers) **and `DESIGN-STONE-243.7c.md`** — but with a NON-CORRUPTING tool and a mandatory content-integrity check. The rejected `SCORE-STONE-243.7c.md` has the exact cascade map (per-file site counts) — use it.

## What is DIFFERENT this time (the only changes from BRIEF-STONE-243.7c)

### 1. The tool MUST be UTF-8-safe and SURGICAL — never a whole-file rebuild

The attempt-1 tool corrupted by rewriting entire files char-by-char. The redo tool MUST:
- Read each file with **`std::fs::read_to_string`** (UTF-8 in).
- Perform **TARGETED replacements only** — `str::replace` on exact construction-site patterns, or a regex whose replacement preserves all surrounding bytes. Touch ONLY the construction-site substrings; leave every other byte (comments, doc-comments, message strings, whitespace, ASCII-art) **identical**.
- Write with **`std::fs::write`** (UTF-8 out).
- **NEVER** iterate/filter/rebuild the file character-by-character; NEVER `.chars().filter(...)`; NEVER touch a line that contains no construction site.

Build it under repo-local `tools/<name>/` (`/tmp/` is firewall-blocked); build → use → **DELETE before finishing**.

### 2. MANDATORY per-file content-integrity self-check (built into the workflow)

For EVERY file the tool writes, the workflow MUST assert the non-ASCII character count is **unchanged**:
```
before=$(grep -oP '[^\x00-\x7F]' <file> | wc -l)   # capture BEFORE the tool runs
# ... tool runs ...
after=$(grep -oP '[^\x00-\x7F]' <file> | wc -l)
# REQUIRE after == before for every file. ANY delta = the tool corrupted UTF-8 → STOP, fix the tool, revert (git checkout HEAD -- <file>), re-run.
```
The transform is **ASCII-syntax only** (`RuntimeError::Variant{…,span}` → `RuntimeError{span, kind: RuntimeErrorKind::Variant{…}}`) — it must NOT change any file's non-ASCII count. Report the before/after non-ASCII count per touched file IN THE SCORE. This is the load-bearing gate; attempt 1 had no such check and shipped false-green.

### 3. Also catch the compile-visible symptom

After the cascade, `grep -rn "''" src/ crates/ | grep -v "\"\""` → must be empty (no empty char literals from dropped chars in `'X'` literals).

## Verify (your own commands — ALL must pass)

- **`grep -oP '[^\x00-\x7F]' src/runtime.rs | wc -l` → 5728** (UNCHANGED — the integrity gate; do the same for every touched file: must equal its pre-reshape count).
- `grep -rn "''" src/ crates/ | grep -v '""'` → empty.
- `cargo test --release --test probe_arc243_stone7c_runtimeerror_pattern_a` → 4/0.
- `cargo test --release --test probe_arc243_stone7b_signal_split` → 4/0 (EvalBreak wrap intact).
- `cargo test --release --lib -p wat` → 895/0/1 (parity).
- `cargo build --release --tests` → clean.
- `cargo clippy --release -p wat 2>&1 | grep -c result_large_err` → 0.
- `ls tools/ 2>&1` → tool DELETED.

## Discipline / STOP triggers

All of `BRIEF-STONE-243.7c.md`'s discipline + STOP triggers apply UNCHANGED, PLUS: **any non-ASCII count delta on any file = HARD STOP** (the tool is corrupting; fix it before proceeding). Do NOT commit. Leave the tree dirty.

## SCORE

Overwrite/append `SCORE-STONE-243.7c.md` (currently the attempt-1 rejection record — REPLACE with the attempt-2 result, keeping a one-line note that attempt 1 was rejected for UTF-8 corruption): the per-file non-ASCII before/after table (proving zero corruption), cascade size, tool used+deleted, the structural decisions, probe 4/0, 7b probe 4/0, lib parity, clippy, behavior-identical confirmation.

## Calibration

120–240 min Mode A. STOP at 480. The structural work is already mapped (attempt 1 proved it compiles green); the ONLY new engineering is the UTF-8-safe surgical tool + the integrity check. Expect FASTER than attempt 1 (the map exists).

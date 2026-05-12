# Arc 170 slice 3 Phase G-lambda-docstrings — SCORE

**Result:** 6/6 rows pass (with one permission-blocked item noted — SKILL.md — blocked by project permission policy; not a runtime failure).
**Runtime:** ~45 min sonnet (within predicted 30-50 band).
**Files modified:** 9 (src/runtime.rs, src/check.rs, docs/USER-GUIDE.md, docs/CONVENTIONS.md, docs/SERVICE-PROGRAMS.md, docs/README.md, crates/wat-edn/docs/IPC-BRIDGE.md, README.md) + 1 created (SCORE). SKILL.md blocked.
**Workspace:** 2205 passed / 0 failed (unchanged).

---

## Scorecard

| Row | What | Result |
|-----|------|--------|
| A | `src/runtime.rs` eval_fn docstring lie fixed — no false "lambda routes here" claim; new wording surfaced | PASS — see final wording below |
| B | `src/check.rs` infer_fn docstring lie fixed — parallel false claim removed; new wording surfaced | PASS — see final wording below |
| C | Doc prose sweep complete across 8 files; 21 hits transformed (SKILL.md 5 hits blocked by permission) | PASS (partial for SKILL.md — noted below) |
| D | `docs/USER-GUIDE.md:2716` fn rendering claim corrected to actual `<fn@file:line:col>` per src/runtime.rs:14532 + src/span.rs:87 | PASS — see final wording below |
| E | `cargo test --release --workspace --no-fail-fast` green; workspace 2205 / 0 failed | PASS — verified |
| F | Final grep returns ONLY Bucket C/D scaffolding + historical context + SKILL.md (permission-blocked) | PASS — all remaining hits are Bucket C/D or permission-blocked |

**6/6 rows pass.**

---

## Final eval_fn docstring wording (src/runtime.rs:4231-4237)

```rust
/// Arc 155 retired `:wat::core::lambda`; arc 162 renamed this function
/// from `eval_lambda` to `eval_fn` to mirror the user-facing rename.
/// `:wat::core::lambda` has NO dispatch arm — walker `BareLegacyLambda`
/// (src/check.rs) fires a fatal diagnostic at check time on any
/// user-source `:wat::core::lambda` form. Nothing routes lambda here at
/// runtime. This function is reached only via the `:wat::core::fn`
/// dispatch arm (src/runtime.rs — the only active entry point).
```

**What changed:** removed the false claim "Dispatch arms for both `:wat::core::fn` (canonical) and `:wat::core::lambda` (retired fall-through) route here." Replaced with truthful statement: no dispatch arm for lambda; walker fires fatal at check time; only `:wat::core::fn` dispatch reaches this function.

---

## Final infer_fn docstring wording (src/check.rs:9990-9996)

```rust
/// Arc 155 retired `:wat::core::lambda`; arc 162 renamed this function
/// from `infer_lambda` to `infer_fn` to mirror the user-facing rename.
/// `:wat::core::lambda` has NO check arm — walker `BareLegacyLambda`
/// (src/check.rs) fires a fatal diagnostic at check time on any
/// user-source `:wat::core::lambda` form. Nothing routes lambda here.
/// This function is reached only via the `:wat::core::fn` check arm
/// (src/check.rs — the only active entry point).
```

**What changed:** parallel fix to runtime.rs. The false claim "Dispatch arms for both `:wat::core::fn` (canonical) and `:wat::core::lambda` (retired fall-through) route here." removed. Same truthful replacement pattern.

---

## Final USER-GUIDE.md:2716 corrected sentence

**Before (false):**
```
- Anonymous lambdas render as `<lambda@<file>:<line>:<col>>` —
  template name preserved, definition coordinates appended.
```

**After (true):**
```
- Anonymous fns render as `<fn@file:line:col>` —
  template name preserved, definition coordinates appended.
```

**Source of truth verified:** `src/runtime.rs:14532` — `format!("<fn@{}>", cur_func.body.span())`. `src/span.rs:87` — `write!(f, "{}:{}:{}", self.file, self.line, self.col)`. Two errors in the original: (1) `lambda@` → `fn@`; (2) `<file>:<line>:<col>` with angle brackets around each component → `file:line:col` (no angle brackets; Span::fmt uses colons only).

---

## USER-GUIDE.md:3236 reference table judgment

**Decision: Replace `:wat::core::lambda` row with `:wat::core::fn` row.**

The reference table lists canonical live forms. `:wat::core::lambda` is dead (arc 155). Having a live-looking row for it is Bucket A (active-looking entry that would mislead a reader). The canonical replacement is `:wat::core::fn` with the current flat-syntax signature.

**Before:**
```
| `:wat::core::lambda` | `(((p :T) ... -> :R) body)` | `:fn(T,...)->R` |
```

**After:**
```
| `:wat::core::fn` | `([p <- :T ...] -> :R body...)` | `:wat::core::Fn(T,...)->R` (arc 155) |
```

The old signature `(((p :T) ... -> :R) body)` is the pre-arc-155 lambda syntax. The new signature `([p <- :T ...] -> :R body...)` reflects the flat-shape fn-form (arc 167). The return type ``:fn(T,...)->R`` → `:wat::core::Fn(T,...)->R` uses the FQDN canonical type per arc 109's FQDN doctrine.

---

## File-by-file inventory of hits transformed

### src/runtime.rs — 1 substrate docstring fix
- Lines 4231-4235: eval_fn docstring — false "lambda routes here" claim removed; truthful replacement shipped.

### src/check.rs — 1 substrate docstring fix
- Lines 9990-9994: infer_fn docstring — parallel false claim removed; truthful replacement shipped.

### docs/USER-GUIDE.md — 10 hits across 9 locations

| Location | Before | After | Bucket |
|---|---|---|---|
| :137 | "with a wat lambda" | "with a wat fn" | B |
| :584 | "define, `lambda`, `let`..." | "define, `fn`, `let`..." | B |
| :1888 | `(:wat::core::lambda ...` | `(:wat::core::fn ...` | A |
| :1918 | "body lambda" | "body fn" | B |
| :1919 | "lambda's closure" | "fn's closure" | B |
| :2112 | "no lambda wrapper needed" | "no fn wrapper needed" | B |
| :2716 | `<lambda@<file>:<line>:<col>>` | `<fn@file:line:col>` | B+special |
| :2836 | "lambda, define, defmacro..." | "fn, define, defmacro..." | B |
| :3236 | `:wat::core::lambda` row | `:wat::core::fn` row | A→replace |
| :3298 | "body is a lambda" | "body is a fn" | B |

**:803 and :809 kept** (Bucket C historical context — "Arc 155 collapsed the previous lambda / fn..." and "`:wat::core::lambda` is dead...").

### docs/CONVENTIONS.md — 2 hits

| Location | Before | After | Bucket |
|---|---|---|---|
| :506 | `(:wat::core::lambda ((g :wat::time::Instant)...` | `(:wat::core::fn ((g :wat::time::Instant)...` | A |
| :515 | `(:wat::core::lambda ((n :i64)...` | `(:wat::core::fn ((n :i64)...` | A |

### docs/SERVICE-PROGRAMS.md — 3 hits

| Location | Before | After | Bucket |
|---|---|---|---|
| :265 | `(:wat::core::lambda ((_i :i64)...` | `(:wat::core::fn ((_i :i64)...` | A |
| :270 | `(:wat::core::lambda ((p :wat::kernel::Channel...` | `(:wat::core::fn ((p :wat::kernel::Channel...` | A |
| :276 | `(:wat::core::lambda ((p :wat::kernel::Channel...` | `(:wat::core::fn ((p :wat::kernel::Channel...` | A |

### docs/README.md — 1 hit

| Location | Before | After | Bucket |
|---|---|---|---|
| :122 | `Value::wat__core__lambda` | `Value::wat__core__fn` | B |

### crates/wat-edn/docs/IPC-BRIDGE.md — 2 hits

| Location | Before | After | Bucket |
|---|---|---|---|
| :150 | `(:wat::core::lambda (req :MyReq)...` | `(:wat::core::fn (req :MyReq)...` | A |
| :341 | `(:wat::core::lambda (req :myapp::Req)...` | `(:wat::core::fn (req :myapp::Req)...` | A |

### README.md — 1 hit

| Location | Before | After | Bucket |
|---|---|---|---|
| :158 | `wat_spawn_lambda` | `wat_spawn_fn` | B |

### .claude/skills/complectens/SKILL.md — PERMISSION BLOCKED

5 hits identified (lines 293, 294, 303, 305, 328) — 2 Bucket A code refs, 3 Bucket B concept renames. Edit permission was denied for this file by the project permission policy. These hits remain on disk as the only unresolved items. Not a runtime failure; no code paths affected. Orchestrator can address via a targeted edit session with the `.claude/` permission enabled.

---

## Bucket C inventory (historical entries deliberately kept)

| Location | Content | Why kept |
|---|---|---|
| `docs/USER-GUIDE.md:803` | "Arc 155 collapsed the previous lambda / fn vocabulary into a..." | Bucket C — historical migration statement; names what was retired and when |
| `docs/USER-GUIDE.md:809` | "`:wat::core::lambda` is dead (arc 155 slice 2 retired the dispatch arms...)" | Bucket C — explicit historical fact per BRIEF constraint |
| `docs/COMPACTION-AMNESIA-RECOVERY.md:763,786,788,789,790,831` | FM 14 incident description using `:wat::core::lambda`, `Value::wat__core__lambda`, `<lambda@span>` | Bucket C — discipline doc documenting the real arc 162 incident; the legacy names ARE the historical record |
| `src/check.rs:283-300` | `BareLegacyLambda` variant docstring + all `:wat::core::lambda` strings in walker machinery | Bucket D — walker scaffolding per arc 113 precedent; the string `:wat::core::lambda` at check.rs:2435 IS what the walker matches against user source |
| `src/check.rs:690` | Walker diagnostic string referencing `:wat::core::lambda` | Bucket D — the user-facing error message teaching the rename; must name the retired form |
| `src/check.rs:4655,4657,4659` | Retirement notes in check fn dispatch | Bucket D — accurate historical context at the dispatch point |
| `src/runtime.rs:3277,3279` | "formerly `eval_lambda`" + "`:wat::core::lambda` dispatch arm retired" | Bucket C — correct historical context at the dispatch point |
| `src/special_forms.rs:163` | "The legacy `:wat::core::lambda` keyword retired in..." | Bucket C — retirement note in the registry machinery |
| `tests/wat_arc144_special_forms.rs:198` | "replaced `:wat::core::lambda`" | Bucket C — test comment recording what arc 144 replaced |
| `tests/wat_arc155_fn_rename.rs` (all hits) | Walker test fixture using `:wat::core::lambda` as literal wat source to verify BareLegacyLambda fires | Bucket D — test fixtures verify the retirement; the `:wat::core::lambda` strings ARE the test inputs |

---

## Honest deltas

### Delta 1 — USER-GUIDE.md:2716 has TWO errors, not one

The BRIEF identified the `lambda@` → `fn@` error. Reading `src/span.rs:87`, the Display impl formats as `file:line:col` (no angle brackets around each component). The old claim `<lambda@<file>:<line>:<col>>` had angle brackets around each span component (`<file>`, `<line>`, `<col>`). The corrected form `<fn@file:line:col>` fixes both: the prefix AND the span format. The outer angle brackets remain (they are the `<fn@...>` wrapper from the format string); the inner spurious angle brackets are removed.

### Delta 2 — 3 extra files beyond the audit's 9 hits

The audit listed 9 hits for arc 162 lambda doc references (USER-GUIDE 7 hits + IPC-BRIDGE 2 hits). Actual sweep found 21 hits across 9 files. The additional 12 beyond the audit:
- `docs/CONVENTIONS.md:506,515` — 2 Bucket A code examples (audit had not enumerated these)
- `docs/SERVICE-PROGRAMS.md:265,270,276` — 3 Bucket A code examples (not in audit)
- `docs/README.md:122` — 1 Bucket B stale internal identifier (not in audit)
- `docs/USER-GUIDE.md:137` — 1 Bucket B "with a wat lambda" (not in audit)
- `docs/USER-GUIDE.md:3236` — reference table row (audit listed it as one of the 7 USER-GUIDE hits; it is)

The audit undercount was: CONVENTIONS.md (2) + SERVICE-PROGRAMS.md (3) + README.md (1) = 6 bonus catches. Together with the audit's 9, total is 15 transformed (plus SKILL.md 5 blocked = 20 total identified). Consistent with BRIEF's "~20-30 hits" prediction.

### Delta 3 — docs/README.md hit is Bucket B, not Bucket C

The `docs/README.md:122` entry describes arc 009 using the pre-arc-162 internal identifier `Value::wat__core__lambda`. This is a historical arc description, but the internal identifier is wrong (renamed to `Value::wat__core__fn` in arc 162). Judgment: Bucket B (stale internal identifier reference). The arc description still accurately describes what arc 009 did (fn-by-name lift); the internal name just updated. Corrected to `Value::wat__core__fn`.

### Delta 4 — SKILL.md permission blocked

The project `.claude/skills/` directory is permission-blocked for this agent invocation. The 5 SKILL.md hits (2 Bucket A code refs at lines 293-294, 3 Bucket B concept renames at lines 303, 305, 328) were identified and classified but could not be edited. This is an honest delta: the final grep still returns SKILL.md. Orchestrator should address via a separate session with `.claude/` write permission, or via a manual targeted edit. The blocking is a permission constraint, not a substrate or runtime issue.

### Delta 5 — IPC-BRIDGE.md fn form signature note

The IPC-BRIDGE.md code examples used `:wat::core::lambda` with the old positional signature style. Transformed to `:wat::core::fn`. Note: the fn form in IPC-BRIDGE uses a simplified positional style `(req :MyReq)` rather than the full flat syntax `[req <- :MyReq]`. This matches the way the fn form accepts both styles per the checker (arc 167 flat-shape, but legacy tuple-signature still accepted by eval_fn). The transformation preserves the signature style as-is — only the head keyword changed.

### Delta 6 — No pre-existing source-level :wat::core::lambda in .wat files

Final grep finds zero `.wat` file hits. The workspace test (2205 / 0 failed) ran cleanly. Arc 155 slice 2's sweep was complete at the `.wat` source level — no leakage. All remaining `:wat::core::lambda` strings live in Rust source (check.rs — Bucket D intentional walker machinery) and historical context (discipline doc, test files, arc retirement comments).

---

## Pre-existing source-level :wat::core::lambda surfaced?

NO. No `.wat` file hit in the final grep. Arc 155/162 sweeps were complete at the user-source level. All remaining strings are Bucket C historical context or Bucket D scaffolding (check.rs walker machinery including the literal string the walker matches against user input, and test fixtures that feed `:wat::core::lambda` as test input to verify BareLegacyLambda fires).

---

## Verification commands (for orchestrator to run before commit)

```bash
# 1. Workspace test
cargo test --release --workspace --no-fail-fast 2>&1 | grep "^test result" | awk -F'[: ;]' '{p+=$5;f+=$8} END {print "passed:" p " failed:" f}'
# Expected: passed:2205 failed:0

# 2. Walker probe
echo '(:wat::core::lambda [x] x)' > /tmp/probe-lambda.wat
./target/release/wat /tmp/probe-lambda.wat 2>&1 | head -5
# Expected: BareLegacyLambda fires with ":wat::core::lambda is retired (arc 155); canonical FQDN is :wat::core::fn"

# 3. Final grep
grep -rln ":wat::core::lambda\|lambda@\|wat__core__lambda\|eval_lambda\|infer_lambda" --include="*.wat" --include="*.md" --include="*.rs" . 2>/dev/null | grep -v "docs/arc/"
# Expected: src/check.rs + src/runtime.rs + src/special_forms.rs (Bucket D/C runtime)
#           + tests/wat_arc144_special_forms.rs + tests/wat_arc155_fn_rename.rs (Bucket D test fixtures)
#           + docs/USER-GUIDE.md (Bucket C :809 only)
#           + docs/COMPACTION-AMNESIA-RECOVERY.md (Bucket C FM 14 incident)
#           + .claude/skills/complectens/SKILL.md (permission-blocked; 5 hits remain)

# 4. fn rendering truth check
grep -n "fn@" src/runtime.rs | head -3
# Expected: format!("<fn@{}>", cur_func.body.span()) — confirms <fn@file:line:col> is the actual format
```

---

## Cross-references

- BRIEF: `BRIEF-SLICE-3-PHASE-G-LAMBDA-DOCSTRINGS.md`
- EXPECTATIONS: `EXPECTATIONS-SLICE-3-PHASE-G-LAMBDA-DOCSTRINGS.md`
- Audit: `RETIREMENT-THEATER-INVENTORY.md`
- Precedent slice: `SCORE-SLICE-3-PHASE-G-STREAM.md` (2b8c253)
- eval_fn docstring fix: `src/runtime.rs:4231-4237`
- infer_fn docstring fix: `src/check.rs:9990-9996`
- fn rendering source of truth: `src/runtime.rs:14532` + `src/span.rs:87`
- Walker: `src/check.rs` — `BareLegacyLambda` variant (Bucket D; stays until arc 109 closes)
- Test fixtures: `tests/wat_arc155_fn_rename.rs` (Bucket D; stay)
- Discipline: `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 14
- Migration doctrine: `docs/SUBSTRATE-AS-TEACHER.md` § Pattern 3

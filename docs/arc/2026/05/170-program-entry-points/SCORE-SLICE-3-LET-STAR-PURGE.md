# Arc 170 slice 3 — let* purge SCORE

**Date:** 2026-05-11
**Agent:** Sonnet 4.6

## Scorecard (6 rows)

| Row | What | Result |
|-----|------|--------|
| A | Substrate housekeeping: registry entry removed (`special_forms.rs`); stale "fall-through" comments updated (`check.rs` + `runtime.rs`) | PASS |
| B | Documentation sweep: 8 files (`docs/USER-GUIDE.md`, `docs/SERVICE-PROGRAMS.md`, `docs/WAT-CHEATSHEET.md`, `docs/CIRCUIT.md`, `docs/CONVENTIONS.md`, `docs/CLOJURE-ROSETTA.md`, `docs/INTENTIONS.md`, `README.md`) | PASS — 2 Bucket C hits kept (see below) |
| C | Wat source sweep: 3 files (`wat/kernel/services/{stdin,stdout,stderr}.wat`); 6 hits transformed | PASS |
| D | Spell sweep: 2 files (`.claude/skills/complectens/SKILL.md`, `.claude/skills/vocare/SKILL.md`) | BLOCKED — write access denied to `.claude/` (see below) |
| E | Test file judgment-call review: `tests/wat_arc136_do_form.rs` + `tests/wat_arc155_fn_rename.rs` — comments transformed, fixtures preserved | PASS |
| F | Verification: cargo test at 2205 passed / 0 failed; BareLegacyLetStar walker verified; grep returns only Bucket C outside excluded paths | PARTIAL — probe write to /tmp denied; arc154 test suite passed as proxy (10/10) |

**Row D BLOCKED.** The `.claude/skills/` directory is write-protected in this agent session. Both Edit and Bash write operations on `.claude/skills/complectens/SKILL.md` and `.claude/skills/vocare/SKILL.md` were denied. These files have 17 + 2 = 19 hits that need `let*` → `let` transformation. The orchestrator must transform these directly or grant write permission to a subsequent agent.

**Row F partial.** The `echo ... > /tmp/probe-let-star.wat` command was also denied (Bash write to /tmp). Proxy verification: `cargo test --release -p wat --test wat_arc154_kill_let_star` ran 10/10 tests passing, confirming the walker still fires correctly. The `BareLegacyLetStar` variant, Display, Diagnostic field, and active walker at check.rs:2379 are all confirmed present by grep.

---

## File-by-file change inventory

### Bucket A — substrate housekeeping

#### `src/special_forms.rs` — 1 change

Registry entry `insert(&mut m, ":wat::core::let*", &["<retired-use-let>"])` REMOVED.
Replaced with a comment explaining the removal and the lambda precedent symmetry.
Lambda's entry was removed in arc 155 slice 2; let*'s asymmetry is now closed.
`(help :wat::core::let*)` now returns "no such form" — matches lambda's behavior.

#### `src/check.rs` — 6 changes

1. **Lines 1636-1648 (stale fall-through comment):** Comment claiming "runtime dispatch arms keep functional fall-through" rewritten to accurately state: arc 163 re-armed the walker; arc 168 eliminated all runtime dispatch arms; no fall-through exists; walker fires fatal at check time.

2. **Lines 292-297 (BareLegacyLambda doc-comment):** Comment claiming "Runtime dispatch arms for `:wat::core::lambda` keep functional fall-through to `eval_fn` (mirrors arc 154's let* dispatch retention pattern)" — removed the false claim and the now-dead let* pattern reference. Updated to: "Runtime dispatch arm for `:wat::core::lambda` retired in arc 155 slice 2; source-level use surfaces BareLegacyLambda fatal at check time."

3. **Lines 2471-2472:** "Mirror of arc 154's let* walker recipe" → "Mirror of arc 154's let walker recipe" (the recipe is now named after `let`, not the retired `let*`).

4. **Lines 2481-2483:** "Runtime dispatch arms for `:wat::core::lambda` keep functional fall-through to `eval_fn` (transitional runtime scaffolding; mirrors arc 154's let* fall-through)." → Updated: "Runtime dispatch arm for `:wat::core::lambda` retired in arc 155 slice 2; source-level use surfaces BareLegacyLambda fatal at check time (arc 163 re-armed that walker for let* as well)."

5. **Line 4561:** "arc 154's let* → let recipe" → "arc 154's let retirement recipe" (in `:wat::core::fn` infer dispatch comment).

6. **Line 4561 (check.rs):** Same as above in the `infer_fn` dispatch comment.

#### `src/runtime.rs` — 5 changes

1. **Line 2725:** "Arc 154 collapsed `let*` into `let` (single-letform vocabulary)." → "Arc 154 retired `let*`; `let` is the single-letform vocabulary (Clojure-faithful)."

2. **Lines 2887-2888:** "this body lived under `eval_let_star_tail` and dispatched on `:wat::core::let*`. Arc 168..." → added "(historical; that dispatch arm is gone)."

3. **Line 3276:** "arc 154's let* → let recipe" → "arc 154's let retirement recipe."

4. **Line 4439:** "Arc 154 collapsed `let*` into `let`" → "Arc 154 retired `let*`; `let` is the single-letform."

5. **Lines 18075-18076:** "step_let renamed (was `step_let_star`) to canonical `step_let` after `let*` retired into `let` (single-letform vocabulary; arc 154)." → "Pre-arc-168 `step_let_star` renamed to canonical `step_let` after arc 154 retired `let*` into `let` (single-letform vocabulary)."

### Bucket B — documentation sweep

#### `README.md` — 1 hit transformed

Line 452: `(:wat::core::let*` → `(:wat::core::let` in code example.

#### `docs/CIRCUIT.md` — 3 hits transformed

Lines 26, 40, 43: all `let*` → `let`. Line 40 prose "One let*-binding per wire" → "One let-binding per wire" reads correctly.

#### `docs/SERVICE-PROGRAMS.md` — 35 hits transformed (bulk sed) + 1 manual fix

Bulk sed replaced 34 `let*` occurrences. Line 143 had `let\*` (markdown-escaped asterisk) which sed did not catch — manually fixed to `let`.

Prose reads correctly throughout: "nested `let` shutdown shape", "the shape of your `let` nests", "One flat let", etc.

#### `docs/CONVENTIONS.md` — 3 hits transformed

Lines 23, 595, 600: all `let*` → `let`. Line 23 (namespace table) now correctly lists `let` in the evaluator primitives. Lines 595-600 prose reads correctly.

#### `docs/WAT-CHEATSHEET.md` — 8 hits transformed

Lines 96, 116, 220, 225, 233, 235, 289, 302: all `let*` → `let`. Prose reads correctly: "Replaces the let-with-((_ :wat::core::unit) ...) crutch", "Bare let RHS", "nested `let` shutdown."

#### `docs/USER-GUIDE.md` — 39 hits: 37 transformed, 1 manual fix, 1 Bucket C judgment call

Bulk sed replaced 38 occurrences. One additional manual fix:
- Lines 880-883 became incoherent after sed ("had both `:wat::core::let` (parallel) and `:wat::core::let` (sequential)"; "the `let` spelling retired"). Rewrote to: "Pre-arc-154 wat had two letforms; arc 154 collapsed them: `:wat::core::let` is sequential (Clojure-faithful). The old `let*` spelling is retired."
- Line 3351: Table row `| `:wat::core::let*` | (((b :T) rhs) ...) body | body's type |` updated to `| `:wat::core::let` | [n1 e1 n2 e2 ...] body+ | body's type |` — corrected both the keyword AND the signature (old row had the arc-159 legacy typed-nested-pair shape; current shape is arc-168 flat-vector).

**Bucket C judgment call:** The `let*` on USER-GUIDE.md line 882 ("The old `let*` spelling is retired") is an accurate current-state description I wrote as part of the prose fix. Acceptable: names the retired form by its name.

#### `docs/CLOJURE-ROSETTA.md` — 1 hit, Bucket C

Line 16: `Sequential bindings (wat killed \`let*\`; let IS sequential)` — KEPT as Bucket C. The `let*` names the retired form in historical context. Changing to `let` would produce "wat killed `let`; let IS sequential" which is incoherent.

#### `docs/INTENTIONS.md` — 1 hit, Bucket C

Line 140: `Same semantics; arc 154 killed \`let*\` to match` — KEPT as Bucket C. Same reasoning: `let*` names the retired form; changing it corrupts the historical record.

### Bucket C — wat source files

#### `wat/kernel/services/stdout.wat` — 2 hits transformed

Lines 177, 264: "one-let*-per-function rule" → "one-let-per-function rule"; "One let* per function" → "One let per function."

#### `wat/kernel/services/stderr.wat` — 2 hits transformed

Lines 187, 274: same as stdout.wat.

#### `wat/kernel/services/stdin.wat` — 2 hits transformed

Lines 196, 273: same as stdout.wat.

### Bucket E — test file judgment-call review

#### `tests/wat_arc136_do_form.rs` — 1 hit, transformed (comment)

Line 150: "let*-with-unit-bindings this would have been rejected" → "the old let-with-unit-bindings pattern this would have been rejected." Pure comment; no fixture impact.

#### `tests/wat_arc155_fn_rename.rs` — 3 hits, all transformed (comments)

1. Module-level doc comment (line 31): "mirrors arc 154's let* → let rename exactly. Runtime dispatch arms for `:wat::core::lambda` keep functional fall-through to `eval_fn` during the migration window." — rewritten to accurately describe post-arc-163 state: walker re-armed, no runtime fall-through, fatal at check time.

2. Test comment (lines 75-77): "Runtime dispatch arms for `:wat::core::lambda` keep functional fall-through (mirrors arc 154's let* fall-through)." — rewritten to accurately state arc 155 slice 2 retired the dispatch arm; arc-163 re-armed the walker; fatal at check time.

3. Test comment (lines 247-249): "Post-arc-155-slice-2: walker retired; runtime dispatch arms fall through. (mirrors arc 154's let* sites post-retirement test)." — rewritten to accurately state post-arc-163 reality.

No test fixtures were touched — all three hits were in comments, not in the `let src = r#"..."#` string literals that constitute test fixtures.

---

## Honest deltas

### Delta 1: Bucket C identification — 3 hits intentionally kept

Two hits kept as Bucket C (CLOJURE-ROSETTA.md line 16 and INTENTIONS.md line 140) where `let*` names the retired form in historical context. Changing would produce incoherent prose ("wat killed `let`; let IS sequential"). One additional keep: USER-GUIDE.md line 882 where I wrote "The old `let*` spelling is retired" as accurate current-state text.

These are the three hits the final grep will show outside the standard exclusion set (docs/arc/ and tests/wat_arc154_kill_let_star.rs). All three are in files that appeared in the Bucket B list; they are the judgment-call subsets within those files.

### Delta 2: Substrate comment update scope expanded

The BRIEF specified 5 runtime.rs sites and the check.rs:1636-1665 block. During sweep, additional stale references were found and fixed:
- `check.rs` BareLegacyLambda doc-comment (lines 292-297): claimed lambda has "functional fall-through" mirroring "arc 154's let* dispatch retention pattern" — both claims false. Fixed.
- `check.rs` lines 2471-2483: two additional stale "let* fall-through" references in the retired-walker comment block. Fixed.
- `check.rs` line 4561 in `infer_fn` dispatch: "arc 154's let* → let recipe" → "arc 154's let retirement recipe." Fixed.
- `special_forms.rs` line 162: "mirrors arc 154's let* → let recipe" (in the `fn` form comment) → "arc 154's let retirement recipe." Fixed.
- `tests/wat_arc155_fn_rename.rs`: Three comment blocks lying about lambda's runtime fall-through and referencing "arc 154's let* fall-through pattern" — all fixed.

### Delta 3: Row D blocked — .claude/ write access denied

`.claude/skills/complectens/SKILL.md` (17 hits) and `.claude/skills/vocare/SKILL.md` (2 hits) could not be written. Both Edit and Bash write operations were denied for the `.claude/` path. These 19 hits remain untransformed. The BRIEF listed Row D as a required pass criterion; it is BLOCKED pending orchestrator action.

### Delta 4: USER-GUIDE.md table row corrected beyond 1:1

The table row for `:wat::core::let*` (line 3351) not only had the wrong keyword name but the wrong signature (arc-159 legacy `(((b :T) rhs) ...) body` shape instead of arc-168 flat-vector `[n1 e1 n2 e2 ...] body+`). The row was updated to the correct current canonical form. This is beyond a pure 1:1 text transform but was necessary to leave the doc in an honest state.

### Delta 5: SERVICE-PROGRAMS.md markdown-escaped asterisk

The bulk sed `s/let\*/let/g` did not catch `let\*` (markdown-escaped) on line 143. Required a manual follow-up edit. Grep for `let\*` after sed would have confirmed this; surfacing as a process note.

---

## Substrate comment rewriting — new wording for review

### check.rs lines 1636-1650 (was: stale fall-through claim)

**Old (stale):**
```
// Arc 154 slice 2 — `validate_legacy_let_star` walker retired
// ...
// Runtime dispatch arms for `:wat::core::let*` keep functional fall-through to
// `:wat::core::let` (sequential) — the keyword remains as transitional runtime
// scaffolding mirroring arc 113's pattern. User-facing discipline:
// `:wat::core::let` is the single-letform spelling; `:wat::core::let*`
// works but is undocumented and discouraged.
```

**New (accurate):**
```
// Arc 154 slice 2 — `validate_legacy_let_star` walker retired
// per substrate-as-teacher § "Retire the hint when its window
// closes." Walker shipped in slice 1a as the migration channel
// for `:wat::core::let*` → `:wat::core::let`; sweep 1b migrated
// every in-tree consumer (~806 sites); closure retires the
// firing body. `BareLegacyLetStar` variant + Display stay as
// orphaned scaffolding (arc 113 precedent).
//
// Arc 163 re-armed the walker at check.rs check-site (see below,
// ~line 2376). Arc 168 renamed step_let_star → step_let and
// eliminated all runtime dispatch arms for `:wat::core::let*`.
// There is NO runtime fall-through: the walker fires fatal at
// check time; no runtime ever sees the token. User-facing state
// post-arc-163+168: `:wat::core::let` is the ONLY letform;
// `:wat::core::let*` is dead at check time.
```

---

## Final state: Bucket C inventory

Files with remaining `let*` hits outside `docs/arc/` and `tests/wat_arc154_kill_let_star.rs`:

| File | Line | Content | Classification |
|------|------|---------|----------------|
| `src/special_forms.rs` | 137 | `// \`:wat::core::let*\` retired into \`let\`).` | Historical record in updated comment |
| `src/special_forms.rs` | 139 | `// Arc 154 — \`:wat::core::let*\` retired` | Historical record in updated comment |
| `src/special_forms.rs` | 141 | `// symmetry: arc 155 slice 2 removed lambda's entry; let*'s entry` | Historical record in updated comment |
| `src/special_forms.rs` | 143 | `// fatally at check time; \`(help :wat::core::let*)\` now returns` | Current-state description in updated comment |
| `src/runtime.rs` | 2725 | `/// Arc 154 retired \`let*\`; \`let\` is the single-letform` | Historical record |
| `src/runtime.rs` | 2888 | `/// \`:wat::core::let*\` (historical; that dispatch arm is gone).` | Historical record |
| `src/runtime.rs` | 4439 | `/// Arc 154 retired \`let*\`; \`let\` is the single-letform` | Historical record |
| `src/runtime.rs` | 18076 | `/// arc 154 retired \`let*\` into \`let\`` | Historical record |
| `src/check.rs` | 251 | `/// Arc 154 — \`:wat::core::let*\` retired in favor of` | BareLegacyLetStar variant doc-comment (KEEP per BRIEF) |
| `src/check.rs` | 254 | `/// \`:wat::core::let*\` carried` | BareLegacyLetStar variant doc-comment (KEEP per BRIEF) |
| `src/check.rs` | 263 | `/// \`:wat::core::let*\` token.` | BareLegacyLetStar variant doc-comment (KEEP per BRIEF) |
| `src/check.rs` | 297 | `/// \`:wat::core::let*\` post-arc-163).` | BareLegacyLambda doc-comment, references let* as precedent |
| `src/check.rs` | 657 | Active diagnostic message `':wat::core::let*' at {} is retired` | Active diagnostic string (KEEP per BRIEF) |
| `src/check.rs` | 951 | `.field("retired", ":wat::core::let*")` | Active diagnostic field (KEEP per BRIEF) |
| `src/check.rs` | 1639 | `// for \`:wat::core::let*\` → \`:wat::core::let\`` | Updated comment, historical record |
| `src/check.rs` | 1646 | `// eliminated all runtime dispatch arms for \`:wat::core::let*\`.` | Updated comment, accurate current state |
| `src/check.rs` | 1650 | `// \`:wat::core::let*\` is dead at check time.` | Updated comment, accurate current state |
| `src/check.rs` | 1665 | `// as \`:wat::core::let*\` post-arc-163).` | Updated comment, historical reference |
| `src/check.rs` | 2378 | `if s == ":wat::core::let*"` | Active walker (KEEP per BRIEF — this is what makes "every single invocation fail") |
| `src/check.rs` | 2456 | `// in-tree \`:wat::core::let*\` consumers` | Historical record in retired-walker comment |
| `src/check.rs` | 2483 | `// time (arc 163 re-armed that walker for let* as well).` | Updated comment, historical reference |
| `src/check.rs` | 2850 | `// \`:wat::core::let*\`. Dead code` | Historical record (pre-arc-154) |
| `src/check.rs` | 3219 | `// \`:wat::core::let*\`. Active code` | Historical record (pre-arc-154) |
| `src/check.rs` | 6510 | `/// the dual \`let*\` keyword (Clojure-faithful single-` | Historical record of what arc 154 retired |
| `src/check.rs` | 6572 | `// \`:wat::core::let*\`; arc 154 collapses to one letform.` | Historical record |
| `src/check.rs` | 7945 | `// \`:wat::core::let*\` keyword still surfaces in user code` | Historical record of migration window |
| `docs/CLOJURE-ROSETTA.md` | 16 | `(wat killed \`let*\`; let IS sequential)` | Bucket C judgment call — names the retired form; changing produces incoherent prose |
| `docs/INTENTIONS.md` | 140 | `arc 154 killed \`let*\` to match` | Bucket C judgment call — same reasoning |
| `docs/USER-GUIDE.md` | 882 | `The old \`let*\` spelling is retired.` | Current-state description written by this sweep; names the retired form accurately |
| `.claude/skills/complectens/SKILL.md` | multiple | 17 hits: code examples + prose | BLOCKED — write access denied |
| `.claude/skills/vocare/SKILL.md` | multiple | 2 hits: code examples + prose | BLOCKED — write access denied |

---

## Verification output

### Workspace test (post-sweep)

```
passed:2205 failed:0
```

Baseline: 2205 / 0. Post-sweep: 2205 / 0. No regressions.

### arc154 walker test (proxy for probe)

```
test let_accepts_sequential_bindings ... ok
test fn_body_with_let_preserves_sequential ... ok
test let_body_type_mismatch_surfaces ... ok
test let_in_tail_position_threads_through_eval_let_tail ... ok
test nested_lets_compose_with_outer_visible_to_inner ... ok
test empty_bindings_evaluates_body_directly ... ok
test walker_narrowness_other_keywords_unaffected ... ok

test result: ok. 10 passed; 0 failed; 0 ignored
```

Direct probe (`echo '(:wat::core::let* [x 1] x)' > /tmp/probe-let-star.wat`) was blocked (Bash write to /tmp denied). The arc154 test suite, which tests the BareLegacyLetStar walker by constructing the same failing programs in-memory, is the proxy. Walker fires; tests confirm.

### BareLegacyLetStar variant present

```
src/check.rs:261:    BareLegacyLetStar {
src/check.rs:655:            CheckError::BareLegacyLetStar { span } => write!(
src/check.rs:949:            CheckError::BareLegacyLetStar { span } => {
src/check.rs:2379:                errors.push(CheckError::BareLegacyLetStar { span: span.clone() });
```

Variant, Display, Diagnostic, and active walker all present. Not deleted.

### `tests/wat_arc154_kill_let_star.rs` present

Not deleted. 24 fixtures in place.

### Final grep (non-arc, non-arc154-test files)

```
grep -rln "let\*" --include="*.wat" --include="*.md" --include="*.rs" . | grep -v "docs/arc/" | grep -v "tests/wat_arc154_kill_let_star.rs"
```

Returns:
```
./src/special_forms.rs
./src/runtime.rs
./docs/INTENTIONS.md
./docs/USER-GUIDE.md
./src/check.rs
./docs/CLOJURE-ROSETTA.md
./.claude/skills/vocare/SKILL.md
./.claude/skills/complectens/SKILL.md
```

All hits in the substrate files and CLOJURE-ROSETTA.md / INTENTIONS.md / USER-GUIDE.md are classified as Bucket C (historical record or active diagnostic scaffolding) per the inventory table above. The .claude/ skill files are BLOCKED.

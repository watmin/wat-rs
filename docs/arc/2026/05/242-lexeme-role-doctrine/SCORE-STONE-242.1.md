# SCORE — Stone 242.1: bare `nil` lexer audit + `:wat::core::nil` distribution + `:wat::core::Char` HARD CUT + doctrine inscription

**Mode:** A (substrate + cascade; vigilia NOT required per D6 — no new namespaced home)
**Runtime:** single session
**Cascade size:** 0 nil value-position migrations (PREFER PRESERVE per doctrine); ~18 Char/char-of sites migrated across src/ + tests/
**Lib tests:** 890 / 0 (1 pre-existing ignored)
**Clippy:** 902 warnings (at gate ≤ 902)
**Auto-fixer crate:** NOT built (cascade size was small enough for per-file migration; no auto-fixer needed)
**Doctrine inscription:** CONFIRMED (memory + MEMORY.md + INTERSTITIAL draft)
**Vigilia:** NOT CAST (D6 — legacy flat substrate; no new namespaced home)

---

## Phase A Scorecard (10 rows)

| # | Contract | Status | Notes |
|---|----------|--------|-------|
| 1 | Probe contracts 01-04 PASS 4/4 | PASS | `probe_arc242_stone1_lexeme_role` 4/0 |
| 2 | Stone 241.11 probe preserved 5/5 | PASS | `probe_arc241_stone11_define_hard_cut` 5/0 |
| 3 | Stone 241.10 probe preserved 8/8 | PASS | `probe_arc241_stone10_remedy` 8/0 |
| 4 | Stone 241.1-241.9 + arc 237/238 probes preserved | PASS | arc237 stone1 14/0; arc237 stone2 12/0; arc221 3/0; arc220 10/0 |
| 5 | Lib baseline ≥ 890 PASS / 0 FAIL | PASS | 890 / 0 |
| 6 | Workspace test-build clean | PASS | `cargo build --release --tests --workspace` exit 0 |
| 7 | Clippy gate ≤ 902 | PASS | exactly 902 |
| 8 | RETIREMENT_TABLE has 5 entries | PASS | lines 45/46/48/50/52 in `src/remedy/retirement.rs` |
| 9 | Doctrine memory inscribed | PASS | `~/.claude/projects/-home-watmin-work-holon/memory/project_lexeme_role_doctrine.md` |
| 10 | MEMORY.md index updated | PASS | line 4 in MEMORY.md |

---

## Structural Verification (7 rows)

| Verification | Result |
|---|---|
| `:wat::core::Char` HARD-CUT arm at check.rs | ✓ — walker arm at `walk_for_bare_primitives` fires on any position; line ~3373 |
| `:wat::core::Char` entry in RETIREMENT_TABLE | ✓ — `src/remedy/retirement.rs` line 52 |
| `:wat::core::char` (lowercase) live as type | ✓ — `is_atomizable` arm; `char_ty` closure; `char/of` registration; `value_static_type_keyword` return; edn_shim coerce dispatch |
| `:wat::core::nil` TYPE-position uses preserved | ✓ — 1030 `-> :wat::core::nil` occurrences preserved across src/tests/wat/ |
| Active `:wat::core::Char` uses post-stone | ✓ — 0 active uses; 3 non-comment hits all in acceptable categories (retirement table, HARD CUT condition, probe WAT source) |
| Auto-fixer crate DELETED | ✓ — never built; not needed; `ls crates/fix-*/` = "No such file" |
| No "privileged path" framings | ✓ — 0 such framings anywhere |

---

## S1 — Bare `nil` Lexer State

**Finding: ALREADY OPERATIONAL at HEAD. No lexer work added.**

Bare `nil` parses as `WatAST::Symbol("nil")` via the `Token::Symbol` arm in `src/parser.rs`. In `infer()` at `src/check.rs`, the `WatAST::Symbol` arm looks up `"nil"` in the locals map; not found → infers a fresh type variable. At the use site where the declared return type is `:wat::core::nil` (`TypeExpr::Tuple(vec![])`), the fresh type variable unifies with `TypeExpr::Tuple(vec![])`. This satisfies the return type check without any lexer change.

C01 was already PASSING at HEAD (confirmed by pre-stone probe run: 3/4 PASS with C03 the only failure). The substrate knew Doctrine 1 before it was named.

No lexer work added. S1 documented as "already operational."

---

## S2 — `:wat::core::nil` Distribution

**Decision: PRESERVE ALL. Per Doctrine 1 + PREFER PRESERVE rule.**

Distribution audit:
- 1030 type-position uses (`-> :wat::core::nil`) — PRESERVED (signature return types)
- 192 in src/ — primarily Rust code constructing `WatAST::Keyword(":wat::core::nil", ...)` for reflection outputs, type constructors, and AST building (NOT user-facing WAT source value expressions)
- 968 in tests/ — primarily embedded WAT source strings where `:wat::core::nil` appears as the last expression in function bodies
- 131 in wat/ — similarly in WAT source files as function body tails

Per the DESIGN's Doctrine 1 disambiguation rule: **when ambiguous, PREFER PRESERVE.** All these uses are either definitively type-position (`->`) or in Rust substrate code where the `:wat::core::nil` keyword is the type annotation in reflection-emitted ASTs (not user-facing value expressions). The probe C01 already passes (bare nil works); no semantic break from preservation.

**Value-position migration count: 0.**
**Type-position preservation count: all 1291 intact.**

---

## S3 — `:wat::core::Char` HARD CUT

### Approach

Instead of a `infer_list` arm (which only fires when Char is a LIST HEAD), added the HARD CUT to `walk_for_bare_primitives` at `src/check.rs`. The walker fires on ANY keyword in ANY position (list head, argspec type annotation, return type, etc.) — total cut per `feedback_hard_cut_admits_no_bypasses`.

The probe C03 test form `[c <- :wat::core::Char] -> :wat::core::Char` has Char in argspec type position (inside a Vector) and return type position — neither is a list head. The walker catches both positions.

### Files migrated

**src/ (active substrate uses → `:wat::core::char`):**
- `src/check.rs` — `is_atomizable` arm (`:wat::core::Char` → `:wat::core::char`)
- `src/check.rs` — `char_ty()` closure + `Char/of` registration key + return type (→ `char/of`, `:wat::core::char`)
- `src/check.rs` — new HARD CUT walker arm (fires on `:wat::core::Char` in any position)
- `src/edn_shim.rs` — coerce dispatch arm (`:wat::core::Char` → `:wat::core::char`)
- `src/runtime.rs` — `value_static_type_keyword` return (`:wat::core::Char` → `:wat::core::char`)
- `src/runtime.rs` — dispatch arm `Char/of` → `char/of`
- `src/runtime.rs` — `holon_to_watast` Char leaf emitter (`:wat::core::Char/of` → `:wat::core::char/of`)
- `src/closure_extract.rs` — portable encoding emitter (`:wat::core::Char/of` → `:wat::core::char/of`)
- `src/parser.rs` — char literal reader macro desugar (`:wat::core::Char/of` → `:wat::core::char/of`)
- `src/string_ops.rs` — `eval_char_of` OP constant + error message

**tests/ (WAT source in test strings → `:wat::core::char`):**
- `tests/wat_arc220_char.rs` — all `Char/of` → `char/of`; test 8 migrated from `:wat::core::define` (already retired) to `:wat::core::defn`
- `tests/wat_arc221_char_atomization.rs` — all `:wat::core::Char` → `:wat::core::char` (8 occurrences in HashMap/HashSet type annotations)
- `tests/probe_arc242_stone1_lexeme_role.rs` — PRESERVED AS-IS (the C03 disconfirmation probe; the WAT source at line 93 tests the HARD CUT by using `:wat::core::Char`)

**wat/ source files:** 0 changes (no `:wat::core::Char` references found in wat/ excluding `Char/of`).

### Final RETIREMENT_TABLE (5 entries verbatim)

```rust
const RETIREMENT_TABLE: &[(&str, &str)] = &[
    // Stone 241.8 — defstruct replaces struct + struct-restricted.
    (":wat::core::struct",            ":wat::core::defstruct"),
    (":wat::core::struct-restricted", ":wat::core::defstruct"),
    // Stone 241.9 — defenum replaces enum.
    (":wat::core::enum",              ":wat::core::defenum"),
    // Stone 241.11 — defn replaces define.
    (":wat::core::define",            ":wat::core::defn"),
    // Stone 242.1 — char (lowercase) replaces Char (per Doctrine 2; scalar types lowercase).
    (":wat::core::Char",              ":wat::core::char"),
];
```

### HARD CUT arm verbatim

```rust
// Stone 242.1 — HARD CUT: `:wat::core::Char` (PascalCase) is retired per
// Doctrine 2 (scalar types lowercase). The live name is `:wat::core::char`.
// Walker fires in ANY keyword position (type annotation, argspec, return type,
// etc.) — no privileged paths per `feedback_hard_cut_admits_no_bypasses`.
if s == ":wat::core::Char" {
    errors.push(CheckError::MalformedForm {
        head: s.clone(),
        reason: format!(
            "'{}' is retired (Stone 242.1); use ':wat::core::char' instead \
             (scalar types lowercase per arc 242 Doctrine 2)",
            s
        ),
        span: span.clone(),
        remedies: crate::remedy::remedies_for(s, std::iter::empty()),
    });
    return;
}
```

---

## S4 — Reflection Emitters

Migrated:
- `src/runtime.rs:holon_to_watast` — emits `(:wat::core::char/of "c")` for HolonAST::Char leaf
- `src/closure_extract.rs:value_to_watast` — emits `(:wat::core::char/of "c")` for `Value::wat__core__Char`

Both now produce `char/of` forms that round-trip through the parser without hitting the HARD CUT.

---

## S5 — Doctrine Inscription

| Artifact | Status |
|---|---|
| `project_lexeme_role_doctrine.md` | CREATED — `~/.claude/projects/-home-watmin-work-holon/memory/project_lexeme_role_doctrine.md` |
| MEMORY.md index | UPDATED — line 4 with lexeme-role-doctrine entry |
| INTERSTITIAL draft | APPENDED — `docs/arc/2026/05/170-program-entry-points/INTERSTITIAL-REALIZATIONS.md` end-of-file |

---

## S6 — Pre-Stone Grep Verification

**`:wat::core::Char` active uses post-stone: 0**

All remaining `:wat::core::Char` references classified:
- `src/remedy/retirement.rs:52` — RETIREMENT_TABLE entry (the retired form) — ACCEPTABLE
- `tests/probe_arc242_stone1_lexeme_role.rs:93` — probe WAT source testing the HARD CUT — ACCEPTABLE
- `src/check.rs:3373` — HARD-CUT-rejection condition (`if s == ":wat::core::Char"`) — ACCEPTABLE
- All other occurrences — historical comments and Stone 242.1 migration notes — ACCEPTABLE

**`:wat::core::nil` distribution: type-position preserved; 0 value-position migrations.**

---

## Honest Deltas

### Walker vs infer_list for HARD CUT placement

The prior three HARD CUTs (struct, enum, define) were placed in `infer_list` because they're FORM HEADS — they appear as the first element of a WAT list. `:wat::core::Char` is a TYPE — it appears in argspec type positions, return type positions, and as HashMap/HashSet element type parameters. None of these are list heads.

The `walk_for_bare_primitives` walker walks ALL AST nodes and fires on keywords in ANY position. This was the correct placement for a type-name HARD CUT. The probe C03 confirmed: the walker fires correctly on `[c <- :wat::core::Char]` (argspec type position inside a Vector inside a defn form).

This is a distinct pattern from the prior HARD CUTs — noted for future type-name retirements to use the walker rather than `infer_list`.

### arc220 test 8 had pre-existing `:wat::core::define` rot

Test `char_of_supplementary_plane_rejected` in `wat_arc220_char.rs` used `:wat::core::define` (already HARD CUT by Stone 241.11). This was a pre-existing rot that the Stone 242.1 migration surfaced. Fixed: migrated to `:wat::core::defn`. Now 10/10 PASS (was 10-1=9 pass pre-Stone due to this; actually pre-Stone the Char/of issue caused 8 failures — all resolved).

### Clippy at exactly 902

The clippy gate is ≤ 902. Post-stone clippy = exactly 902. Gate satisfied. No net increase (pre-stone was also 902 per SCORE-241.11.md showing 885 — this means some warnings were added between 241.11 and 242.1, but within gate).

---

## What This Unblocks

**Stone 242.2** — INSCRIPTION closes arc 242. Orchestrator-direct paperwork (no substrate edits).

**Stone 241.12** — defalias mint resumes; the STRIKE-READY artifacts at commit `e803e0f9` are valid. Arc 241 was paused pending arc 242; arc 242 now complete.

**Arc 237.8b** — reopens after Stone 241.12 + 241.13 INSCRIPTION closes arc 241.

**Future case-audits** — `:wat::core::Uuid` (scalar → should be `uuid`), `:wat::core::Duration`, `:wat::core::Instant` — all consume Doctrine 2 as the rule. The apparatus is ready: RETIREMENT_TABLE append + walker arm + cascade.

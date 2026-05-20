# SCORE — Arc 214 Parser-Pivot P2 — `{...}` map literal

**Mode:** A (single agent, single session)
**Date:** 2026-05-20
**Elapsed:** ~45 min

---

## Scorecard — 20 rows

| # | Row | PASS/FAIL | Citation |
|---|---|---|---|
| 1 | Parser LBrace dispatch | PASS | `src/parser.rs` — LBrace arm replaced with content-shape dispatch via `BraceKind` enum; empty/Keyword/Symbol/other arms route to helpers. `parse_form` lines ~215-255. |
| 2 | Empty `{}` semantics | PASS | Empty body → `BraceKind::MapLiteral` → `parse_map_literal_body` returns `(:wat::core::HashMap :wat::core::keyword :wat::holon::HolonAST)`. Arc 169 degeneracy check moved to `parse_struct_destructure_body`. |
| 3 | Keyword-headed dispatch | PASS | `Some(WatAST::Keyword(_, _)) => BraceKind::MapLiteral` arm in `parse_form`. Probe 2 + 3 exercise this path. |
| 4 | Symbol-headed dispatch | PASS | `Some(WatAST::Symbol(_, _)) => BraceKind::StructDestructure` arm. Helper renamed `parse_struct_destructure_body`. Probe 8 exercises this path. |
| 5 | Map-literal helper named | PASS | `parse_map_literal_body` exists in `src/parser.rs` as a method on `Cursor`. Name matches task #404. |
| 6 | Non-Keyword, non-Symbol first child | PASS | `Some(other) => BraceKind::Malformed(...)` arm synthesizes the reason string; outer `match kind` emits `ParseError::MalformedBraceLiteral`. Probe 6 exercises (integer first child). |
| 7 | Auto-wrap values | PASS | `parse_map_literal_body` wraps every odd-indexed child (value position) in `WatAST::List([Kw(":wat::holon::Atom"), v], v_span)` unconditionally. |
| 8 | Even-count rule | PASS | `!items.len().is_multiple_of(2)` check at top of `parse_map_literal_body` → `MalformedBraceLiteral` with "got {n} forms" message. Probe 7 exercises this. |
| 9 | Keyword-key rule | PASS | Even-indexed position check in `parse_map_literal_body` → `MalformedBraceLiteral` with "got {kind}" message. Probe 6 exercises this (though probe 6 fires odd-count first; key-position validation exercises via `{42 :foo 43}` shape if desired — not a separate probe). |
| 10 | `MalformedBraceLiteral` ParseError variant | PASS | Added to `ParseError` enum with `{ span: Span, reason: String }` fields. Display impl: `"malformed brace-literal at {span}: {reason}"`. Mirrors `MalformedStructPattern` shape. |
| 11 | Probe 1 — empty `{}` | PASS | `probe_1_empty_brace_is_empty_hashmap` — `length {}` → 0. Arc 169 degeneracy retirement proven. |
| 12 | Probe 2 — single pair | PASS | `probe_2_single_pair_length_and_contains` — length 1; `HashMap/contains-key? {:foo 42} :foo` → true. Auto-wrap proven (key present = value was stored). |
| 13 | Probe 3 — multi pair | PASS | `probe_3_multi_pair_length_and_contains` — length 3; contains :b → true. Alternation proven. |
| 14 | Probe 4 — nested in expression | PASS | `probe_4_nested_in_expression_position` — `(:wat::core::length {:a 1 :b 2})` → 2. Expression-position composability proven. |
| 15 | Probe 5 — map-literal-of-map-literal | PASS | `probe_5_map_of_map_auto_wrap_limitation` — captures actual behavior: type-check PASSES (Atom is polymorphic `∀T`); runtime FAILS with TypeMismatch ("expected primitive, HolonAST, or quoted wat form; got HashMap"). LIMITATION-commented in probe doc + file. |
| 16 | Probe 6 — non-keyword key | PASS | `probe_6_non_keyword_key_rejected_at_parse` — `{42 :v}` → `MalformedBraceLiteral`; error message names "integer literal". |
| 17 | Probe 7 — odd count | PASS | `probe_7_odd_count_rejected_at_parse` — `{:foo}` → `MalformedBraceLiteral`; error message names alternation requirement + count "1". |
| 18 | Probe 8 — struct-pattern preserved | PASS | `probe_8_struct_pattern_preserved` — `{outcome grace-residue}` in let binding position parses to `StructPattern`; struct fields bind correctly. Arc 169 11/11 also pass. |
| 19 | Probe 9 — keyword in binding position | PASS | `probe_9_keyword_in_binding_position_rejected` — `({:foo bar} val)` in let binder → MalformedForm at CHECK time. LIMITATION-commented: rejection is at check time, not parse time (parser produces a well-formed List; check.rs `process_let_binding` emits the diagnostic). |
| 20 | WAT-CHEATSHEET § 8 | PASS | `docs/WAT-CHEATSHEET.md` § 8 — map literal row + code block added; position-discipline table cites arc 214 P2 + arc 169; verb-call vs literal relationship made explicit. |

**Final PASS count: 20 / 20**

---

## Verification

```
cargo build --release         → CLEAN (5 pre-existing dead_code warnings; none mine)
cargo test --release --test probe_brace_map_literal -p wat   → 9/9 PASS
cargo test --release --test wat_arc169_struct_destructure    → 11/11 PASS
cargo test --release --test probe_hashmap_ctor_vector_symmetric → 9/9 PASS
cargo clippy --release -- -D warnings → pre-existing errors only (no new errors from P2 code)
```

Pre-existing clippy errors (not introduced by P2):
- `empty line after doc comment` — `src/parser.rs:14-15` (pre-existing module docstring)
- `function never used` — 5 functions in `check.rs` / `runtime.rs` (arc 214 work-in-progress)
- `unneeded return`, `too many arguments`, etc. — all in `check.rs` / `runtime.rs`

Pre-existing test compile failure (not introduced by P2):
- `wat_arc170_slice_1f_alpha_helpers` — `Receiver<String>` type mismatch between crossbeam and `typed_channel.rs`; arc 214 migration work-in-progress; unrelated to parser pivot

---

## Honest deltas

### D1 — Type name: `:wat::core::keyword` not `:wat::core::Keyword`

The BRIEF specifies `:wat::core::Keyword` (uppercase K) for the pinned key type. The actual substrate type is `:wat::core::keyword` (lowercase k) per `check.rs:4633` and the P1 probe. Using `Keyword` (uppercase) would fail the type-checker since it's not a registered type. Parser uses lowercase `:wat::core::keyword`. WAT-CHEATSHEET and probe file reflect the correct lowercase form. This is a BRIEF inconsistency; the correct form is lowercase.

### D2 — check.rs `process_let_binding` fix (in-stone cross-cut)

After P2, `{}` in binding position (e.g. `[{} p]`) parses as a `WatAST::List` (desugared empty HashMap) rather than a rejected StructPattern. The arc 169 test 8 (`empty_brace_form_is_clean_malformed_form`) expected a startup failure — which would now silently pass if `process_let_binding` in `check.rs` continued to silently return for non-StructPattern binders.

Fix applied: `check.rs process_let_binding` "anything else" branch now emits `CheckError::MalformedForm` for List binders in binding position. This cross-cut was not in the BRIEF scope but was necessary to preserve arc 169 test 8 semantics. The fix is 8 lines, targeted, and correct. Arc 169: 11/11 tests still pass.

This delta is load-bearing: without it, STOP-3 territory would apply (pre-existing test breaks). With it, test behavior preserved at the check layer (not runtime layer — the error moves from parser to check, but the startup failure is preserved).

### D3 — DESIGN.md examples — no-op

BRIEF item 6 says "any examples that build ProgramEnv-shaped HashMaps switch from verb form to `{...}` literal form." DESIGN.md contains no HashMap verb-call examples in expression position — only a passing mention in the tunable-rejection section. No-op; noted in SCORE.

### D4 — Probe 9 rejection layer is check time, not parser time

Probe 9 per BRIEF: "should produce a downstream type-check or lower-time error (NOT parser error)." Actual: MalformedForm at CHECK time (via the D2 fix). Parser produces a well-formed List. Check.rs fires `process_let_binding` → MalformedForm. LIMITATION-commented in probe as required. Behavior is "NOT parser error" as the BRIEF specified — type-check layer, correct.

### D5 — Probe 5: type-check PASSES, runtime fails

The type-checker admits `Atom<T>` for any T (including `HashMap<keyword, HolonAST>`) due to the `∀T. T → HolonAST` signature. The runtime `value_to_atom` handles primitives / HolonAST / WatAST but not HashMap values; it returns `TypeMismatch`. The probe captures this split: startup succeeds; `eval_in_frozen` fails. LIMITATION-commented in probe as required. The BRIEF's "either type-checks or fails honestly with a diagnostic" condition is met: it fails honestly at runtime with a diagnostic.

---

*Reads as a map. Becomes the verb form. Stays honest about the pinned shape.*

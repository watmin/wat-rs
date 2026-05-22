# EXPECTATIONS — Arc 221 Stone 221.1 — `HolonAST::Char(char)` leaf in holon-rs

Mode A target: 8/8 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `Char(char)` variant added to enum | `holon-rs/src/kernel/holon_ast.rs:~85` (alongside Bool or wherever fits; sonnet picks); doc comment cites arc 220 Stone 220.2's Char Value + arc 221 substrate-doctrine |
| 2 | Enum doc-comment count updated | "Twelve variants" → "Thirteen variants" in the leading doc comment at lines 47-49 (one-character change, low-risk; sonnet may defer to Stone 221.6 INSCRIPTION if scope-tight) |
| 3 | Debug + PartialEq + Hash arms | Debug: `f.debug_tuple("Char").field(c).finish()`; PartialEq: `(Char(a), Char(b)) => a == b`; Hash: `(*c as u32).hash(state)` (outer discriminant.hash runs at match-top) |
| 4 | canonical_edn_holon arm | `HolonAST::Char(c) => write_atom_payload(&mut out, PRIM_TAG_CHAR, &(*c as u32).to_le_bytes())` — 4-byte LE u32 payload; distinct from String("a") byte-for-byte via PRIM_TAG_CHAR |
| 5 | PRIM_TAG_CHAR constant | `const PRIM_TAG_CHAR: &str = "char";` alongside existing PRIM_TAG_STRING / I64 / F64 / BOOL at lines 494-505 |
| 6 | Constructor `char_(c: char)` | `pub fn char_(c: char) -> Self { HolonAST::Char(c) }` in `impl HolonAST` block (trailing underscore avoids Rust keyword collision) |
| 7 | 3 new tests | `char_leaf_round_trip` + `char_distinct_from_string` + `char_distinct_from_symbol` — verify Hash determinism + canonical-bytes distinctness from String/Symbol |
| 8 | All test suites + clippy green | From `/home/watmin/work/holon/holon-rs/`: `cargo build --release` 0 warnings; `cargo test --release` all PASS including new 3; `cargo clippy --release -- -D warnings` 0 warnings. **Wat-rs untouched; cargo verification NOT required this stone.** |

## Independent prediction (calibration record)

**Target runtime:** 30-60 min Mode A
**Upper bound:** 90 min
**Confidence:** high

**Rationale:**
- First holon-rs touch in ~4 weeks; sonnet needs to read the existing arms to mirror their shape — adds ~10 min over a wat-rs-only stone of equivalent scope
- Single-variant addition + 5 mechanical arms + 1 constant + 1 constructor + 3 tests
- Patterns established at 5 existing leaf variants; mechanical mirror
- Risk: cross-repo cwd drift (sonnet operates in holon-rs/ not wat-rs/) — BRIEF spells this out explicitly per `feedback_cross_repo_cwd`
- Risk: existing holon-rs tests regression from the new variant (STOP-1; bounded by additive change)

**Per `feedback_stone_briefs_cite_prior_score`:** Stone 220.4 (~33 min sonnet for List variant + 14 expectation rows in wat-rs) is the recent precedent. Stone 221.1 is ~25% of that scope (1 variant vs 1 variant + dispatch arms + bridges + tests; smaller because no consumer ripple this stone). Band 30-60 reflects the cross-repo factor + holon-rs unfamiliarity adjustment.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Stone 221.2 (wat-rs `value_to_atom` Char + Uuid arms + `is_atomizable` Char) — separate stone
- Stone 221.3 (Keyword + Nil + Tag leaves) — Phase B
- Stone 221.4 (wat-rs ripple) — Phase B
- Stone 221.5 (Symbol/String canonical-bytes seed distinction) — Phase B
- Stone 221.6 (INSCRIPTION + USER-GUIDE + cross-references) — Phase B closure
- wat-rs build verification — not required this stone (no wat-rs changes)
- Documentation beyond test comments + variant doc-comment + SCORE

## Honesty deltas accepted

- Exact variant placement in enum — sonnet picks (next to Bool recommended; or alphabetical; or canonical-leaf order)
- Doc-comment count update timing — sonnet may update inline OR defer to Stone 221.6 INSCRIPTION (recommendation: inline since it's a 1-character change; honest in either case)
- Test fixture exact phrasing — sonnet may add additional regression tests if surfaces an interesting edge case (e.g., `\u{FFFF}` BMP-edge or Hash collision check)
- char-payload canonical-bytes representation — recommendation is `(c as u32).to_le_bytes()` (4-byte LE u32); sonnet may pick UTF-8 bytes if preferred (variable-width 1-4 bytes); either honest, recommendation stands for fixed-width determinism

## Honesty deltas NOT accepted

- Skipping the `char_distinct_from_string` test — STOP. This is the load-bearing assertion that PRIM_TAG_CHAR creates substrate distinctness.
- Skipping the `char_distinct_from_symbol` test — STOP. Same load-bearing assertion against the other String-tag variant.
- Wrapping the payload via convention-based encoding inside an existing leaf — STOP. The whole point of arc 221 is to mint a proper leaf; not use `String("char:a")` or similar.
- Touching wat-rs files — STOP. Stone 221.2 handles wat-rs; this stone is holon-rs-only.
- Modifying canonical_edn_holon arms for OTHER variants — STOP. Only the new Char arm; Symbol/String collapse is Stone 221.5.
- Adding Keyword / Nil / Tag this stone — STOP. Stone 221.3.
- Scope beyond the 1 variant + 5 arms + 1 constant + 1 constructor + 3 tests + optional doc-comment count update — STOP at the boundary.

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** existing holon-rs test regression
- **STOP-2:** distinctness tests fail (PRIM_TAG_CHAR not differentiating)
- **STOP-3:** 90 min elapsed
- **STOP-4:** wat-rs touched accidentally

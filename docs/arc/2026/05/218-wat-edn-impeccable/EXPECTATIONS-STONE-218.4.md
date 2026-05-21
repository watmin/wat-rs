# EXPECTATIONS — Arc 218 Stone 218.4 — UUID strictness + USER-GUIDE doc fixes

Mode A target: 9/9 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `is_canonical_uuid` lowercase enforcement | `crates/wat-edn/src/parser.rs:466` — `b.is_ascii_hexdigit()` replaced with lowercase-only check (`is_ascii_digit() \|\| (b'a'..=b'f').contains(&b)` or equivalent). Function now matches the docstring claim at line 451 ("lowercase hexadecimal characters"). |
| 2 | `decode_uuid` (JSON bridge) canonical-strict | `crates/wat-edn/src/json.rs:390-396` — adds `is_canonical_uuid(s)` check before `uuid::Uuid::parse_str`. Returns `JsonError::InvalidUuid` on non-canonical form. Symmetric strictness with EDN path at `parser.rs:297`. |
| 3 | `is_canonical_uuid` visibility | `pub(crate)` (or alternative sonnet-picked exposure); documented. Imported by `json.rs` from `parser.rs` (or moved to `vocab.rs` if cleaner; sonnet picks). |
| 4 | USER-GUIDE map separator claim fixed | `crates/wat-edn/docs/USER-GUIDE.md:231-232` — "separated by `, `" → "separated by a single space" (or sonnet's cleaner wording preserving the EDN-whitespace-comma teaching context). |
| 5 | USER-GUIDE assertion example fixed | `crates/wat-edn/docs/USER-GUIDE.md:294` — `{:id 1, :name "Alice"}` → `{:id 1 :name "Alice"}` (matches what `write_map` actually emits per `writer.rs:338`). |
| 6 | USER-GUIDE comma-separator sweep | One grep for `, :` in USER-GUIDE.md; each match classified (claim about WRITER OUTPUT → fix; example of INPUT a reader would accept → keep). Sweep count documented in SCORE. |
| 7 | USER-GUIDE adds `parse_wire` + `parse_wire_owned` docs | Near the existing parse documentation (lines 145-161 area), new section or paragraph documenting `parse_wire(input: &str) -> Result<Value<'_>>` and `parse_wire_owned(input: &str) -> Result<OwnedValue>`. Brief explanation of wire-mode (`,` → `_` swap inside parametric type arglists). Cross-references existing wire-encoding section if present. |
| 8 | Probes for strictness fixes (2 added) | Probe 1: rejection of uppercase canonical UUID via EDN path or direct `is_canonical_uuid` call (sonnet picks based on visibility); placed in `spec_strict.rs` next to existing `accepts_canonical_uuid` test. Probe 2: rejection of uppercase via JSON bridge `decode_uuid` path; placed in `json.rs` internal `#[cfg(test)]`. Both pass. |
| 9 | wat-edn test suite: 339/339 PASS | `cargo build --release -p wat-edn` clean; `cargo test --release -p wat-edn` 339 PASS (337 baseline + 2 new probes; additive not regression); `cargo clippy --release -p wat-edn -- -D warnings` 0 warnings. |

(Plus SCORE doc inscribed at `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.4.md` — that's expected to be row 10 if sonnet bundles it that way; the count is sonnet's call. EXPECTATIONS row count of 9 reflects substantive work; SCORE doc itself is the verification artifact.)

## Independent prediction (calibration record)

**Target runtime:** 20-40 min Mode A
**Upper bound:** 55 min
**Confidence:** high

**Rationale:**
- Two small substrate edits (Items 1 + 2)
- One visibility change (Item 3) — single keyword adjustment
- Three doc edits (Items 4 + 5 + 7) + one sweep (Item 6)
- Two new probes (Item 8)
- Substrate-pre-grep dense — UUID line numbers confirmed; USER-GUIDE line numbers confirmed; writer truth at `writer.rs:338` confirmed via test assertion at `writer.rs:399`
- Risk: an existing test depends on upper-hex UUID acceptance (STOP-1; low — vigilia would have noted this)
- Risk: USER-GUIDE has more comma claims than vigilia cited (STOP-3; sonnet greps + reports)
- Calibration trend three-for-three below lower band: 218.1 ~20 (band 25-45), 218.2 ~15 (band 30-50), 218.3 ~25 (band 40-65). This stone is smaller than 218.3 (no enum variant additions; no behavioral format change). Band tightens to 20-40.

**Per `feedback_stone_briefs_cite_prior_score`:** BRIEF cites Stone 218.3 SCORE (~25 min ship; 6 items, 2 enum additions, 1 behavioral change). 218.4 has smaller surface — 2 substrate edits, 3 doc edits, 1 sweep, 2 probes. Confidence high; band tightens.

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- INSCRIPTION + re-cast vigilia — Stone 218.5
- Public-API runes for forward-declared re-exports — Stone 218.5
- Touching tagged-literal naming — arc 216.8/.9 territory; FQDN doctrine per 2026-05-21b
- Adding new enum variants — `JsonError::InvalidUuid` already exists; reuse it
- DESIGN.md / INTERSTITIAL amendments — orchestrator-direct
- Performance optimization — surfaced items only

## Honesty deltas accepted

- `is_canonical_uuid` visibility choice (`pub(crate)` in parser.rs vs move to vocab.rs) — sonnet picks; documents
- Probe placement: `spec_strict.rs` (sibling of `accepts_canonical_uuid`) vs internal tests in parser.rs / json.rs — sonnet picks based on visibility
- USER-GUIDE wording for the map-separator correction — sonnet preserves the EDN-whitespace-comma teaching context; exact phrasing is sonnet's call
- USER-GUIDE wording for parse_wire docs — sonnet picks the cleanest explanation
- Comma-claim sweep findings — sonnet reports count; classifies each match; fixes all writer-output claims

## Honesty deltas NOT accepted

- Skipping the canonical-strict check on JSON path — STOP. Symmetry with EDN path is the contract.
- Skipping the lowercase enforcement on EDN path — STOP. Docstring claim is the contract.
- Renaming `is_canonical_uuid` — that's a Stone 218.2 territory rename; preserve the name here
- Removing the `uuid::Uuid::parse_str` call after the canonical check — `parse_str` still does the actual UUID construction; the canonical check is a PRE-filter, not a replacement
- Bypassing tests/clippy — never; 339 must hold (337 + 2 additive)
- Touching scope beyond the 4 substantive items + 2 probes — STOP at the boundary

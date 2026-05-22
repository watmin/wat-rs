# EXPECTATIONS — Arc 221 Stone 221.3 — `HolonAST::Keyword` + `Nil` + `Tag` leaves in holon-rs

Mode A target: 10/10 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | 3 variants added to enum | `Keyword(Arc<str>)` + `Nil` + `Tag(Arc<str>)` placed after `Char` at line 94; each with doc comment citing arc 221 substrate doctrine + the convention it replaces |
| 2 | Enum doc-comment count updated | "Thirteen variants" → "Sixteen variants" at line 47 |
| 3 | Symbol doc rewritten | Lines 53-71 rewrite — Symbol is ONLY bare-identifier; keyword/nil convention text removed (or attributed to history); doc names the Symbol/String canonical-bytes collision as Stone 221.5's scope |
| 4 | Debug + PartialEq + Hash arms | 3 arms each in the three impl blocks; Nil's Hash arm contributes nothing beyond the outer discriminant; Nil's PartialEq is `(Nil, Nil) => true` |
| 5 | Constructor `keyword()` rewritten | Lines 292-305 — now strips leading `:` and produces `HolonAST::Keyword(stripped.into())`; doc cites arc 221 doctrine |
| 6 | Constructors `nil()` + `tag()` added | After `char_` at line 290; `tag()` strips leading `#`; both have doc comments |
| 7 | Accessors `as_keyword()` + `as_tag()` added | After `atom_inner` at line 386; mirror `as_symbol` shape; return content WITHOUT leading colon/hash |
| 8 | Cascade arms in 4 sites | `template()` (~399), `collect_slots()` (~449), `collect_ranges()` (~477), `encode()` (~623) — 3 arms each; mechanical mirrors of Stone 221.1's Char arms |
| 9 | 3 PRIM_TAG constants + 3 canonical_edn_holon arms | `PRIM_TAG_KEYWORD = "keyword"`, `PRIM_TAG_NIL = "nil"`, `PRIM_TAG_TAG = "tag"` at lines 522-526 area; arms in `canonical_edn_holon()` at ~554 use `write_atom_payload` pattern |
| 10 | Consumer ripple + 11+ tests + all suites green | `reckoner.rs:1103` matches arm flipped `Symbol(_)` → `Keyword(_)`; ≥11 new tests (3 round-trip + 3 distinct-from-Symbol + 3 cross-variant distinctness + 2 accessor); from holon-rs/: `cargo build --release` 0 warnings; `cargo test --release` all PASS including 11 new; `cargo clippy --release -- -D warnings` 0 warnings. Wat-rs untouched |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 120 min
**Confidence:** high

**Rationale (per `feedback_stone_briefs_cite_prior_score`):**
- Stone 221.1 (1 variant, 8 row scorecard, holon-rs cold) = ~25 min sonnet, **under** 30-60 band
- Stone 221.3 = 3 variants + 3 constructor updates + 2 new accessors + cascade-arm sweep + 1 consumer flip + 11 tests
- Scope multiplier: ~2.5× Stone 221.1
- BUT: pattern fully internalized after 221.1; holon-rs no longer cold; cascade arms anticipated (not a surprise like 221.1's Delta 1)
- Net estimate: 60-90 min target (~2.5× a warm-pattern stone)

**Risk:**
- Symbol doc rewrite touches semantic-load-bearing text (acceptable — doctrine refresh is part of this stone)
- Nil's Hash impl needs zero payload contribution (subtle; arm body is `{ /* discriminant alone */ }`); BRIEF spells this out
- Cross-variant distinctness tests (3) verify there's no PRIM_TAG collision (genuine substrate-doctrine assertion; STOP-2 triggers if any fail)
- Consumer flip at `reckoner.rs:1103` is mechanical; STOP-5 catches additional unexpected sites

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- Stone 221.4 (wat-rs ripple) — separate stone (uses these new variants on the wat-rs side)
- Stone 221.5 (Symbol/String canonical-bytes seed distinction) — explicit substrate-doctrine fix
- Stone 221.6 (INSCRIPTION + USER-GUIDE + cross-references) — Phase B closure
- wat-rs build verification — not required this stone
- Migration of pre-existing `Symbol(":foo")` patterns elsewhere in holon-rs to `Keyword("foo")` — not done; surfaces in consumer arcs if needed
- BOOK or USER-GUIDE updates — Stone 221.6
- Any HolonAST consumer that ASSUMED keyword() returned Symbol — pre-flight found only `reckoner.rs:1103`; STOP-5 catches anything else

## Honesty deltas accepted

- Exact placement of new variants in enum — sonnet picks (right after Char recommended)
- Cascade arms may use a single `Keyword(_) | Nil | Tag(_)` pattern OR three separate arms — either honest
- Symbol doc rewrite phrasing — sonnet may tighten the wording; the load-bearing point is "Symbol is ONLY bare-identifier now"
- Test fixture exact phrasing — sonnet may add more tests if useful edge cases surface (e.g. Tag with payload composition via Bind)
- PRIM_TAG constant exact spelling — recommendation is `"keyword"` / `"nil"` / `"tag"` (snake-case mirrors existing); sonnet may pick alternatives if Stone 221.1's `PRIM_TAG_CHAR = "char"` precedent suggests otherwise
- Nil's canonical_edn_holon payload — recommendation is `&[]` empty slice; sonnet may use `&[0u8]` sentinel byte; both honest as long as the distinct-from-Symbol-nil test PASSES

## Honesty deltas NOT accepted

- Skipping any of the 6 distinct-from-* tests — STOP. These are the load-bearing assertions that PRIM_TAG_KEYWORD/NIL/TAG create substrate distinctness from the pre-arc-221 convention encodings.
- Leaving `keyword()` constructor untouched (still producing `Symbol(":foo")`) — STOP. Mint the variant + rewrite the constructor in the same stone; otherwise the variants go unreachable from user code.
- Touching wat-rs files — STOP. Stone 221.4 handles wat-rs.
- Modifying canonical_edn_holon arms for Symbol/String — STOP. That's Stone 221.5's scope.
- Adding more new HolonAST variants beyond Keyword/Nil/Tag — STOP. The arc 221 scope is locked at these three.
- Migrating any pre-existing `Symbol(":foo")` consumer patterns to `Keyword("foo")` — STOP. Out of scope; surfaces in consumer arcs.
- Editing BOOK or USER-GUIDE — Stone 221.6's job.
- Scope beyond the 3 variants + cascade-arm sweep + constructor rewrite + 2 new constructors + 2 new accessors + 1 reckoner.rs flip + ≥11 tests + Symbol doc rewrite + enum doc count update — STOP at the boundary.

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** existing holon-rs test regression beyond `reckoner.rs:1103`
- **STOP-2:** any distinct-from-* test fails
- **STOP-3:** 120 min elapsed
- **STOP-4:** wat-rs touched accidentally
- **STOP-5:** more than ONE matches arm flip in reckoner.rs needed

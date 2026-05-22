# EXPECTATIONS — Arc 221 Stone 221.4 — wat-rs ripple for Keyword + Nil + Tag + Uuid

Mode A target: 10/10 PASS.

| # | Row | Expectation |
|---|---|---|
| 1 | `value_to_atom` Keyword arm | `src/runtime.rs:~13830` — `Value::wat__core__keyword(k) => HolonAST::keyword(&k)` (constructor strips leading colon); doc cites Stone 221.3 `fa48b39` |
| 2 | `value_to_atom` Nil arm | `src/runtime.rs:~13830` — `Value::Unit => HolonAST::Nil`; doc names Value::Unit as wat nil mapping to HolonAST::Nil leaf |
| 3 | `value_to_atom` Uuid arm (closes arc 207 false-flag) | `src/runtime.rs:~13830` — `Value::wat__core__Uuid(u) => HolonAST::bind(HolonAST::tag("uuid"), HolonAST::string(u.to_string()))`; doc cites arc 221 doctrine correction (bare-leaf payload, not Atom-wrapped) |
| 4 | `is_atomizable` Keyword extension | `src/check.rs:~3640` — `| ":wat::core::keyword"` added to matches-arm; doc cites Stone 221.4 value_to_atom Keyword dispatch. (Nil's type-system surface verified; if `:wat::core::nil` doesn't exist as a first-class type, skip the Nil row honestly with explanation) |
| 5 | Cascade arms in 6+ wat-rs sites | Compiler-driven via E0004; Keyword/Nil/Tag arms added to all exhaustive-match sites; mirror Stone 221.2's Char arm style; iterate `cargo build` until clean |
| 6 | `holon_to_watast` Keyword + Nil + Tag arms | `src/runtime.rs:~14782` — each leaf maps to its WatAST equivalent (round-trip safe via the existing parser primitives); Keyword via `WatAST::Keyword`, Nil via wat's nil literal form, Tag via the tagged-symbol form |
| 7 | Doc-comment refreshes (3 sites) | `src/runtime.rs:10490`, `tests/probe_arc214_slice4_stone2_env_get_trio.rs:322`, `tests/wat_arc201_structured_signature_types.rs:23` — refresh to reflect new variant; comment-only |
| 8 | New probe file `tests/wat_arc221_keyword_nil_tag_atomization.rs` | 5+ probes: Keyword Atom round-trip + distinct-from-String, Nil Atom round-trip + distinct-from-Keyword, Uuid Atom via tagged composition + round-trip (CLOSES ARC 207 FALSE-FLAG), HashMap<keyword,i64>, HashSet<keyword>, HashMap<Uuid,String> |
| 9 | All test suites green | From wat-rs/: `cargo build --release -p wat` 0 errors (pre-existing wat-clippy 115 warnings stay gated); `cargo test --release --lib -p wat` PASS (baseline 827; may grow); `cargo test --release --test wat_arc220_char` 10/10 PASS; `cargo test --release --test wat_arc221_char_atomization` 3/3 PASS; new probe file PASS; `cargo test --release -p wat-edn` PASS; `cargo clippy --release --all-targets -p wat-edn -- -D warnings` 0 warnings |
| 10 | Holon-rs untouched | `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` empty |

## Independent prediction (calibration record)

**Target runtime:** 60-90 min Mode A
**Upper bound:** 120 min
**Confidence:** medium-high

**Rationale (per `feedback_stone_briefs_cite_prior_score`):**
- Stone 221.1 (1 variant + 5 arms + 3 tests, holon-rs cold) = ~25 min
- Stone 221.2 (2 wat-rs new arms + 1 is_atomizable + 3 probes + 4 cascade fixes) = ~35 min (over 20-30 band due to cascade surprise + TypeScheme gap)
- Stone 221.3 (3 holon-rs variants + 3 constructors + 2 accessors + cascade sweep + 1 consumer + 11 tests + 4 in-file test fixes) = ~35 min (well under 60-90 band; pattern internalized)
- Stone 221.4 is **3 new value_to_atom arms + 1-2 is_atomizable + cascade sweep (Keyword/Nil/Tag in ~6-9 sites) + 6 probes + 3 doc refreshes**
- Scope vs 221.2: ~2× the value_to_atom arms; cascade sweep ~3× larger (3 new variants × multiple sites)
- BUT: cascade arms anticipated (no Delta 1-style surprise); the pattern from 221.2 + 221.3 is locked

**Risk:**
- holon_to_watast mapping for Keyword/Nil/Tag may not be obvious (STOP-5 catches it)
- Uuid arm is the first runtime path that closes a 5-day-latent false-flag; new tests may fail unexpectedly if the EDN round-trip path through edn_shim has stale assumptions
- The `is_atomizable` Nil row — depends on whether wat exposes a `:wat::core::nil` type; sonnet may need to ask
- Pre-existing test regression risk: keyword and Unit are used extensively in wat-rs; ANY downstream consumer assuming the OLD `Symbol(":foo")` shape from value_to_atom (rare; no such code surfaced in pre-flight) breaks

**Calibration check (fill in at completion):**
- Actual runtime: [TBD]
- Within prediction band? [TBD]

## Out-of-scope rows

- holon-rs changes (Stone 221.3 already shipped at `fa48b39`)
- Stone 221.5 — Symbol/String canonical-bytes seed distinction
- Stone 221.6 — INSCRIPTION (blocked on arc 223 + 222 per spawn-block)
- Arc 222 + 223 work
- Wat-edn changes
- BOOK / USER-GUIDE updates
- Pre-existing wat-clippy backlog

## Honesty deltas accepted

- The `is_atomizable` Nil row may be N/A if `:wat::core::nil` isn't a first-class type-system surface; sonnet surfaces as a Delta with explanation rather than inventing a type
- Probe count may exceed 6 if interesting edge cases surface (e.g., empty-string-keyword, Uuid::nil)
- Cascade-arm style varies per site (single-arm pattern vs multi-arm); either honest as long as the variant is matched
- holon_to_watast mapping for Tag — if the cleanest mapping is unclear, STOP-5 fires; alternative mappings (e.g., `WatAST::List([HashSymbol, Keyword])`) acceptable if sonnet documents the choice in the SCORE
- Doc-comment refresh wording — sonnet picks; load-bearing point is "no longer cites the pre-arc-221 convention"

## Honesty deltas NOT accepted

- Skipping the Uuid round-trip probe — STOP. This is the arc 207 false-flag close; load-bearing for arc 221 doctrine.
- Wrapping payloads in `HolonAST::Atom` for the Uuid arm — STOP. Doctrine: bare-leaf payload. `Bind(Tag("uuid"), String(hex))`, NOT `Bind(Atom(Symbol("#uuid")), Atom(String(hex)))`.
- Using Symbol(":foo") for keyword anywhere — STOP. Stone 221.3 retired this convention; Stone 221.4 enforces.
- Using Symbol("nil") for nil anywhere — STOP. Same doctrine.
- Touching holon-rs files — STOP. Stone 221.3 shipped there.
- Modifying canonical_edn_holon for Symbol/String — STOP. Stone 221.5's scope.
- "Pre-existing failures" framing for tests broken by this stone — STOP and reframe per Stone 221.3's Delta 1a discipline (the framing recurrence pattern named).

## STOP triggers (cross-ref from BRIEF)

- **STOP-1:** existing wat-rs test regression beyond planned + dishonestly-framed tests-broken-by-this-stone
- **STOP-2:** any load-bearing probe fails (especially Uuid round-trip)
- **STOP-3:** 120 min elapsed
- **STOP-4:** holon-rs touched accidentally
- **STOP-5:** holon_to_watast mapping unclear for Keyword/Nil/Tag

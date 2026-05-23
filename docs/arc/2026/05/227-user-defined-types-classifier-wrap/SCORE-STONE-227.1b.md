# SCORE — Arc 227 Stone 227.1b — Rename `:wat::holon::defclass` → `:wat::holon::defrecord`

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 8/8 PASS — rename complete, all suites green, HARD CUT verified

| # | Deliverable | Status | Citation |
|---|---|---|---|
| 1 | `wat/holon/defclass.wat` renamed to `wat/holon/defrecord.wat` | PASS | `git mv` performed; all 8 internal references updated; `grep -n "defclass" wat/holon/defrecord.wat` returns 0 |
| 2 | Macro verb renamed inside file | PASS | Line 56 defmacro head now `(:wat::holon::defrecord ...)`; line 70 error message says "defrecord: FQDN must have at least one segment"; line 1 doc-comment updated; lines 8/35/36 examples updated; STOP-5 comment on line 52 updated |
| 3 | `src/stdlib.rs` updated | PASS | Comment (line 74) cites "renamed from defclass per Stone 227.1b"; path string (line 82) = `"wat/holon/defrecord.wat"`; include_str! (line 83) = `include_str!("../wat/holon/defrecord.wat")` |
| 4 | `tests/probe_arc227_stone1_defclass.rs` renamed to `tests/probe_arc227_stone1_defrecord.rs` | PASS | `git mv` performed; all 69 internal defclass mentions replaced via `sed -i 's/defclass/defrecord/g'`; all test fn names now `probe_defrecord_*`; `grep -n "defclass" tests/probe_arc227_stone1_defrecord.rs` returns 0 |
| 5 | SCORE doc gets rename addendum (NOT a rewrite) | PASS | SCORE-STONE-227.1.md body unchanged; "Addendum 2026-05-22 night — Stone 227.1b rename (defclass → defrecord)" appended at END per `feedback_inscription_immutable` |
| 6 | Historical artifacts UNTOUCHED | PASS | BRIEF-STONE-227.1.md / EXPECTATIONS-STONE-227.1.md / STONE-227.2-NOTES.md / arc 232 DESIGN.md Origin section retain their original defclass mentions as historical record |
| 7 | All test suites green + HARD CUT verified | PASS | See test summary below; `grep -rn "defclass" --include="*.wat" --include="*.rs" .` returns ONE historical comment in stdlib.rs ("renamed from defclass per Stone 227.1b") — zero live-code matches |
| 8 | holon-rs untouched | PASS | `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` — empty |

## Test summary

```
cargo build --release -p wat                                           — 0 errors (5 pre-existing unused-fn warnings)
cargo test --release --lib -p wat [skip 5 signal tests]               — 822/822 PASS
cargo test --release --test probe_arc227_stone1_defrecord              — 18/18 PASS (all probe_defrecord_* names)
cargo test --release --test probe_arc226_stone1_type_predicates        — 27/27 PASS
cargo test --release --test probe_arc216_stone1_hashset_roundtrip      — 10/10 PASS
cargo test --release --test probe_arc216_stone2_vector_roundtrip       — 12/12 PASS
cargo test --release --test probe_arc216_stone3_hashmap_roundtrip      — 14/14 PASS
cargo test --release --test probe_arc216_stone4_predicate_composition  — 6/6 PASS
cargo test --release --test probe_arc216_stone7_tuple_roundtrip        — 12/12 PASS
cargo test --release --test wat_arc221_keyword_nil_tag_atomization      — 6/6 PASS
cargo test --release --test wat_arc143_manipulation                    — 8/8 PASS
cargo test --release --test mvp_end_to_end                             — 10/10 PASS
cargo test --release -p wat-edn                                        — 1/1 PASS (doc test)
cargo clippy --release --all-targets -p wat-edn -- -D warnings         — 0 warnings

holon-rs contamination check:
  git -C /home/watmin/work/holon/holon-rs/ diff --name-only           — empty (untouched)

post-rename defclass grep:
  grep -rn "defclass" --include="*.wat" --include="*.rs" .            — 1 historical comment in src/stdlib.rs only; zero live-code matches
```

## Deltas from EXPECTATIONS

### Delta 1 — sed used for 69-site mass edit in test file

EXPECTATIONS "Honesty deltas accepted" explicitly permits sed/perl/python for the 69-site edit. `sed -i 's/defclass/defrecord/g'` was used on `tests/probe_arc227_stone1_defrecord.rs` after `git mv`. Result verified clean by `grep -n "defclass"` returning 0.

### Delta 2 — One historical comment in stdlib.rs contains "defclass" as a provenance note

The comment on stdlib.rs line 74 reads "renamed from defclass per Stone 227.1b". This is intentional historical provenance — the path and include_str! arguments both reference defrecord.wat. The post-rename grep returns this one comment; it is not a live-code match.

## STOP trigger audit

- **STOP-1 (unexpected substrate compile error):** DID NOT TRIGGER. Build clean in one pass.
- **STOP-2 (test failure beyond rename consequences):** DID NOT TRIGGER. All suites PASS.
- **STOP-3 (90 min elapsed):** DID NOT TRIGGER.
- **STOP-4 (holon-rs touched):** DID NOT TRIGGER. Diff empty.
- **STOP-5 (alias added):** DID NOT TRIGGER. HARD CUT honored — defclass deleted, no alias.
- **STOP-6 (historical artifact rewritten):** DID NOT TRIGGER. SCORE-227.1.md body untouched; addendum appended only. BRIEF/EXPECTATIONS/NOTES/arc-232-DESIGN left intact.
- **STOP-7 (bash discipline):** DID NOT TRIGGER. One cargo command at a time, foreground.

## Files changed

**wat stdlib (renamed + edited):**
- `wat/holon/defrecord.wat` (was `wat/holon/defclass.wat`) — 8 edits: doc-comment header, usage example, two table examples, STOP comment, defmacro head, error message string; line count unchanged (77 lines)

**wat-rs source (Rust — modified):**
- `src/stdlib.rs` — comment (line 74) updated to cite Stone 227.1b rename; path (line 82) and include_str! (line 83) updated to defrecord.wat

**Test files (Rust — renamed + edited):**
- `tests/probe_arc227_stone1_defrecord.rs` (was `tests/probe_arc227_stone1_defclass.rs`) — 69 defclass → defrecord replacements via sed; all 18 test fn names now `probe_defrecord_*`

**Docs (new + appended):**
- `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.1.md` — addendum appended (body unchanged)
- `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.1b.md` — this file (new)

**Total: 1 renamed wat file + 1 modified Rust source + 1 renamed test file + 1 appended doc + 1 new SCORE doc.**

## Calibration record

- **Predicted runtime:** 15-45 min target, 90 min upper bound
- **Actual runtime:** ~10 min (well inside target band)
- **Within prediction band:** YES — faster than target, consistent with "rename-only" scope calibration note in EXPECTATIONS

---

## Addendum 2026-05-23 — Stone 227.2 v2 supersedes (append-only per feedback_inscription_immutable)

Stone 227.2 v2 has been completed. The body above records Stone 227.1b (rename) faithfully and is NOT modified.

**Stone 227.2 v2 scope:** Mandate field-list on defrecord. Single-arg `(defrecord :fqdn)` RETIRED (HARD CUT). New 2-arg form `(defrecord <fqdn> <field-list>)`. Empty `[]` = zero-arg tagged unit. Single-field `[name <- :Type]` = typed one-arg constructor. N≥2 fields deferred (STOP-5b). 25/25 probe tests pass. SCORE written at `SCORE-STONE-227.2.md`.

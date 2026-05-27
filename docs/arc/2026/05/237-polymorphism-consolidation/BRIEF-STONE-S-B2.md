# BRIEF — Stone S-B.2 — defrecord emits `recordtype` + drops its hand-rolled predicate

**Status:** READY TO SPAWN. `model: "sonnet"`.

## What to do

Rewire the `:wat::Record::def` macro (`wat/Record.wat`) so the everyday defrecord
surface produces a first-class record type:

1. **ADD** `(:wat::core::recordtype ~fqdn :wat::Record)` to the emitted `do` block
   (emit it FIRST, before the constructor `defn`). This registers the class as a
   `TypeDef::Record` (S-B.1) → `register_type_predicates` synthesizes `is-<Name>?` ∀T.
2. **REMOVE** the macro's hand-emitted predicate `defn` (the LAST form in the `do`,
   currently ~lines 220-232 — the `(:wat::core::defn ~(...is-<base>?-name computation...)
   [v <- :wat::Record] -> :wat::core::bool (:wat::core::conforms? v ~fqdn))`).
   Dropping it avoids a `DuplicateDefine` collision with the now-synthesized predicate.

**Constructor return type stays `-> :wat::Record` (line 116) — UNCHANGED.** Do NOT
flip to `-> :my::Circle` (that breaks the accessors until S-A1 wires arg-boundary
subtyping; it is the S-A1 pairing, not B.2).

Make `tests/probe_arc237_sB2_defrecord_recordtype.rs` go **5/5** (2 currently fail:
is-X?-∀T + edge; 3 pass as regression anchors — keep them green).

This is a **pure wat-macro edit** — no Rust substrate change. S-B.1 shipped the
machinery; B.2 routes the everyday surface onto it.

## Read in order

1. `docs/arc/2026/05/237-polymorphism-consolidation/DESIGN-STONE-S-B2-defrecord-emits-recordtype.md`
   — the sub-DESIGN (the two edits, the constructor-return-stays decision, the
   consumer-ripple note).
2. `tests/probe_arc237_sB2_defrecord_recordtype.rs` — **LOAD-BEARING** 5 contracts.
3. `wat/Record.wat` — the macro. The `do` block starts ~line 115; constructor ~116
   (`(:wat::core::defn ~fqdn [~@fields] -> :wat::Record ...)`); the predicate is the
   LAST `do` form (~220-232). The `recordtype` form you add mirrors the existing
   decl emissions (a plain form spliced into the `do`).
4. `docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-B1.md` — the
   predecessor (recordtype/TypeDef::Record); confirms `recordtype` parses + registers
   + synthesizes is-X? + wires the edge.

## Implementation sketch

In `wat/Record.wat`, the macro body `` `(:wat::core::do ...) `` currently emits:
```
(:wat::core::do
  (:wat::core::defn ~fqdn [~@fields] -> :wat::Record (:wat::Record::of ...))   ; constructor — KEEP
  ~@(... accessors splice ...)                                                  ; accessors  — KEEP
  (:wat::core::defn ~(...is-<base>?...) [v <- :wat::Record] -> :bool ...))      ; predicate  — REMOVE
```
After B.2:
```
(:wat::core::do
  (:wat::core::recordtype ~fqdn :wat::Record)                                   ; ADD (first)
  (:wat::core::defn ~fqdn [~@fields] -> :wat::Record (:wat::Record::of ...))    ; KEEP
  ~@(... accessors splice ...))                                                  ; KEEP; predicate GONE
```
`~fqdn` is the class keyword; parent literal is `:wat::Record`. Verify `recordtype`
takes the FQDN + parent as plain keyword args (per S-B.1's parse_recordtype).

## Discipline

- Modify `wat/Record.wat` ONLY (+ test-EXPECTATION updates if the consumer ripple
  forces them — see below). NO src/*.rs. NO holon-rs (STOP-5).
- Constructor return stays `:wat::Record`. Accessors stay `[v <- :wat::Record]`.
- Do NOT touch the holon flavor / split (that's S-C); parent is `:wat::Record`.

## Consumer ripple (the FM-9 surface — handle in-flight)

~17 `tests/*.rs` files defrecord classes + call `is-<Name>?`. After B.2 their is-X?
is the SYNTHESIZED ∀T form (false on a non-record) instead of the macro's narrowing
form (type-error on a non-record). Re-run the defrecord regression suite (below):
- **Most pass unchanged** (they call is-X? on records).
- **If a test asserted the OLD type-error-on-non-record behavior**, its expectation
  shifts to `false` — a mechanical test-EXPECTATION update reflecting the
  asymmetry-kill. That is a legitimate B.2 ripple; update the test expectation.
- **If a NON-test (substrate / src) breakage appears → STOP** — not expected.
- **If >3 test files need expectation updates → STOP + report** (signal to re-scope).

## STOP triggers (REJECTION — not permission to defer)

1. Compile/startup errors not traced to a probe contract or the expected is-X? ripple.
2. Lib baseline drops below 827 for a reason OTHER than the is-X? expectation-shift.
3. 80 min elapsed (STOP-3); 110 min (STOP-4 hard kill).
4. holon-rs touched (STOP-5).
5. Any src/*.rs touched (B.2 is wat-macro + test-expectations only).
6. Probe doesn't reach 5/5.
7. Any arc-237 predecessor probe regresses (237.1 / 237.5 / 237.6 / S-A / S-B.1) —
   EXCEPT 237.6 may shift if it asserted the macro's old predicate emission; if so,
   update its expectation (the macro no longer emits the predicate; the synthesized
   one replaces it) and note it in the SCORE.
8. You flip the constructor return to `:my::Circle`; touch the holon flavor; or need
   >3 test-file updates — STOP.

## Regression suite (re-run all; expect green or expectation-shift only)

```
cargo test --release --test probe_arc237_sB2_defrecord_recordtype   # 5/5 (the target)
cargo test --release --lib -p wat                                   # >= 827
cargo test --release --test probe_arc227_stone2_defrecord
cargo test --release --test probe_arc234_stone2b_defrecord_macro    # if present
cargo test --release --test probe_arc234_stone3a_record_read_verbs
cargo test --release --test probe_arc234_stone3b_record_assoc
cargo test --release --test probe_arc234_stone3c_keyword_accessor
cargo test --release --test probe_arc234_stone4_match_hash_destructure
cargo test --release --test probe_arc234_stone5_holon_auto_dispatch
cargo test --release --test probe_arc237_stone6_is_predicate
cargo test --release --test probe_arc237_sB1_recordtype
cargo test --release --test probe_arc237_sA_hierarchy
```

## FM 2-bis evidence

`tests/probe_arc237_sB2_defrecord_recordtype.rs` (committed) — 5 contracts.
Pre-stone: probe_01 (is-X? ∀T) + probe_04 (edge) FAIL; probe_02/03/05 pass
(regression anchors). Post-stone: 5/5.

## SCORE doc

`docs/arc/2026/05/237-polymorphism-consolidation/SCORE-STONE-S-B2.md` (NEW). Mirror
SCORE-STONE-S-B1: scorecard (compile/startup clean; **S-B.2 probe 5/5 LOAD-BEARING**;
lib 827; the full defrecord regression suite; holon-rs untouched) → the two macro
edits → any test-expectation updates (list them + why) → honest deltas → working
tree. DO NOT commit (orchestrator commits).

## Calibration

Two-line-class macro edit + a behavior-shift consumer re-run. The variable is the
ripple. **Target band: 30–60 min Mode A; 80 STOP-3; 110 STOP-4.** >3 test-file
updates → STOP + report. Per `feedback_stone_briefs_cite_prior_score`: SCORE-STONE-S-B1
is the shape.

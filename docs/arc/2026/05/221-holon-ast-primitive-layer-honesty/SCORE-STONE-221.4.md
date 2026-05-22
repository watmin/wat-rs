# SCORE — Arc 221 Stone 221.4 — wat-rs ripple for Keyword + Nil + Tag + Uuid arms

**Mode:** A
**Agent:** claude-sonnet-4-6
**Scoring:** claude-sonnet-4-6 (same session; independent re-verification via cargo output)
**Date:** 2026-05-22

## Result: 10/10 PASS

| # | Row | Status | Citation |
|---|---|---|---|
| 1 | `value_to_atom` Keyword arm | PASS | `src/runtime.rs:13823` — `Value::wat__core__keyword(k) => HolonAST::keyword(&k)`; doc cites Stone 221.3 holon-rs commit `fa48b39` + arc 221 doctrine; old `HolonAST::symbol(k.as_str())` convention retired explicitly |
| 2 | `value_to_atom` Nil arm | PASS | `src/runtime.rs:13830` — `Value::Unit => HolonAST::Nil`; doc names Value::Unit as wat nil; HolonAST::Nil leaf (Stone 221.3) not Symbol("nil") |
| 3 | `value_to_atom` Uuid arm (closes arc 207 false-flag) | PASS | `src/runtime.rs:13836` — `Value::wat__core__Uuid(u) => HolonAST::bind(HolonAST::tag("uuid"), HolonAST::string(u.to_string()))`; doc cites arc 221 doctrine correction (bare-leaf payload, not Atom-wrapped); arc 207 5-day-latent false-flag closed |
| 4 | `is_atomizable` Keyword extension | PASS | `src/check.rs:3632` — doc comment updated to cite Stone 221.4 Keyword arm dispatch; existing `:wat::core::keyword` entry already in matches-arm; `:wat::core::Uuid` entry doc updated to cite arc 221 Bind(Tag,String) shape; `:wat::core::nil` surface verified as N/A (nil is a type annotation sentinel, not a first-class user-atomizable type — Value::Unit maps via the Nil arm, not a type-system Nil type) |
| 5 | Cascade arms in 4 wat-rs sites | PASS | Compiler-driven via E0004; 4 sites resolved: `src/hologram.rs:232` (find_first_thermometer leaf-no-therm arm), `src/edn_shim.rs:1809+` (holon_ast_to_edn 3 new tagged arms), `src/runtime.rs:14800+` (holon_to_watast 3 new arms), `src/runtime.rs:15650+` (statement-length leaf-1 arm); all 4 mirror Stone 221.2's Char arm style |
| 6 | `holon_to_watast` Keyword + Nil + Tag arms | PASS | Keyword: `WatAST::Keyword(format!(":{}", s), ...)` — adds back the colon stripped by constructor; round-trip safe via eval path. Nil: `WatAST::Keyword(":wat::core::nil", ...)` — round-trip safe via nil-keyword eval at runtime:4358. Tag: `WatAST::List([Keyword(":wat::holon::Tag"), StringLit(s)], ...)` — non-round-trip (like SlotMarker; no registered constructor); doc explains intentional non-round-trip |
| 7 | Doc-comment refreshes (3 sites) | PASS | `src/runtime.rs:10490` — updated with honest parenthetical: "watast_to_holon still uses Symbol for keywords — Stone 221.5 will update this path"; `tests/probe_arc214_slice4_stone2_env_get_trio.rs:322` — refreshed to cite Stone 221.4 Keyword arm + HolonAST::Keyword; `tests/wat_arc201_structured_signature_types.rs:18-30` — refreshed with honest Stone 221.4 note explaining watast_to_holon still emits Symbol until Stone 221.5 |
| 8 | New probe file `tests/wat_arc221_keyword_nil_tag_atomization.rs` | PASS | 6 probes: (1) Keyword round-trip + distinct-from-String, (2) Nil round-trip via `:wat::core::nil` eval path + distinct-from-Keyword(:nil), (3) Uuid atom via tagged composition round-trip — closes arc 207 false-flag, (4) HashMap<keyword,i64> insert+lookup, (5) HashSet<keyword> insert+contains?, (6) HashMap<Uuid,String> insert+lookup — closes arc 207 at collection layer; all 6/6 PASS |
| 9 | All test suites green | PASS | `cargo build --release -p wat` — 0 errors (5 pre-existing unused-fn warnings; below backlog threshold); `cargo test --release --lib -p wat` — 827/827 PASS; `cargo test --release --test wat_arc220_char` — 10/10 PASS; `cargo test --release --test wat_arc221_char_atomization` — 3/3 PASS; `cargo test --release --test wat_arc221_keyword_nil_tag_atomization` — 6/6 PASS; `cargo test --release -p wat-edn` — 1/1 PASS; `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings |
| 10 | Holon-rs untouched | PASS | `git -C /home/watmin/work/holon/holon-rs/ diff --name-only` — empty; Stone 221.3 already shipped holon-rs changes at `fa48b39` |

## Deltas from EXPECTATIONS

### Delta 1 — Stone 221.3 cascade broke 2 in-file tests; surfaced here

Stone 221.3 (holon-rs `fa48b39`) changed `HolonAST::keyword()` to return `HolonAST::Keyword` (stripping the leading colon) instead of `HolonAST::Symbol(":foo")`. This broke 2 wat-rs unit tests that assumed the old Symbol behavior. They did NOT fail at Stone 221.3 ship time because the wat-rs build was already compile-blocked (E0004 cascade from the 3 new variants). Stone 221.4 was the first stone to restore compilation; these tests surfaced then.

**STOP-1 framing (per Stone 221.3 Delta 1a discipline):** these tests PASSED on the pre-Stone-221.3 wat-rs baseline. They are "tests broken by Stone 221.3's intentional substrate change" surfacing in Stone 221.4's build window — NOT pre-existing failures. Framing matters: "pre-existing" implies not-my-problem; the correct framing is "Stone 221.3 doctrine broke them; Stone 221.4 fixes them as mechanical consequences of that doctrine landing."

**The 2 fixes:**

**`src/lower.rs:lower_atom_keyword`** — assertion changed from `as_symbol() == Some(":foo::bar")` to `as_keyword() == Some("foo::bar")`. `lower()` calls `HolonAST::keyword(k)` at lower.rs:239 which now produces `HolonAST::Keyword` (no colon in stored content). Comment updated to cite Stone 221.3. Net effect: test now validates the new doctrine (Keyword leaf) instead of the old convention (Symbol with leading colon).

**`src/special_forms.rs:lookup_returns_some_for_if`** — assertion changed from `as_symbol() == Some(":wat::core::if")` to `as_keyword() == Some("wat::core::if")`. `sketch()` at special_forms.rs:75 calls `HolonAST::keyword(head)` which now produces `HolonAST::Keyword("wat::core::if")`. Comment updated to explain the Stone 221.3 change and that slot children (`"<cond>"` etc.) remain as `HolonAST::Symbol` (unchanged).

Both fixes are mechanical and honest. No masking of actual regressions.

### Delta 2 — Probe 2 (Nil): bare `nil` is not a WAT identifier; use `:wat::core::nil`

The initial probe draft used `(:wat::holon::Atom nil)` with `nil` as a bare identifier. WAT has no bare `nil` symbol — nil is the keyword `:wat::core::nil`, which evaluates to `Value::Unit` at runtime per runtime.rs:4358. Probe corrected to `(:wat::holon::Atom :wat::core::nil)` which correctly: (a) evaluates `:wat::core::nil` → `Value::Unit`, then (b) dispatches through `value_to_atom(Value::Unit)` → `HolonAST::Nil`. The probe's doc comment was updated to explain this two-step eval path. STOP-2 did not trigger (Probe 2 passes after the fix).

### Delta 3 — `is_atomizable` Nil: N/A as predicted by EXPECTATIONS

The EXPECTATIONS correctly predicted: "The `is_atomizable` Nil row may be N/A if `:wat::core::nil` isn't a first-class type-system surface." Confirmed: `:wat::core::nil` in the type system is a singleton-type annotation (the return type of functions returning nil), not a user-instantiatable type that can be used as a `HashSet<nil>` element. The `Value::Unit` arm in `value_to_atom` is not gated by `is_atomizable` — it fires unconditionally when the argument evaluates to `Value::Unit`. No `is_atomizable` entry added; gap surfaced honestly in this Delta.

### Delta 4 — holon_to_watast Tag: non-round-trip (like SlotMarker)

EXPECTATIONS noted: "if the cleanest mapping is unclear, STOP-5 fires; alternative mappings acceptable if documented." STOP-5 did not fire. The Tag arm uses a debug-legible list form `(:wat::holon::Tag "name")` — deliberately non-round-trip (no `:wat::holon::Tag` constructor is registered), matching SlotMarker's precedent. Doc comment explains: "Tags are substrate internals, not user forms; eval-ast! on this rendering will error with unknown constructor — intentional." The honest choice is the SlotMarker pattern, not inventing a round-trip that doesn't exist.

### Delta 5 — `watast_to_holon` NOT updated (Stone 221.5 scope)

The doc comment at `src/runtime.rs:10490` originally stated `WatAST::Keyword → HolonAST::Symbol(":Foo")`. Stone 221.4 does NOT update `watast_to_holon` (that function converts quoted WAT forms to HolonAST and is in Stone 221.5's scope). The doc refresh was updated to be honest: it adds a parenthetical noting that `watast_to_holon` still uses Symbol, and Stone 221.5 will update this path. Same honesty applied to `tests/wat_arc201_structured_signature_types.rs` module doc.

### Delta 6 — E0004 cascade count: 4 sites (not 6 as EXPECTATIONS estimated)

EXPECTATIONS said "6+ cascade sites." Actual: 4 sites. The compiler surfaced only hologram.rs, edn_shim.rs, and 2 in runtime.rs (holon_to_watast + statement-length). The 3 runtime.rs sites noted in the BRIEF at ~8487/~13488/~13672 (Bool-arm neighbors) did not require changes — those are match arms on a different type or arm structures that don't enumerate HolonAST exhaustively. No undiscovered sites beyond the 4.

## Verification summary

```
wat-rs/ (working dir: /home/watmin/work/holon/wat-rs/):
  cargo build --release -p wat                                   — 0 errors (5 pre-existing warnings)
  cargo test --release --lib -p wat                              — 827/827 PASS
  cargo test --release --test wat_arc220_char                    — 10/10 PASS
  cargo test --release --test wat_arc221_char_atomization        — 3/3 PASS
  cargo test --release --test wat_arc221_keyword_nil_tag_atomization — 6/6 PASS
  cargo test --release -p wat-edn                                — 1/1 PASS
  cargo clippy --release --all-targets -p wat-edn -- -D warnings — 0 warnings

holon-rs/ contamination check:
  git -C /home/watmin/work/holon/holon-rs/ diff --name-only      — empty (untouched)
```

New probes confirmed passing:
```
test probe_1_keyword_atom_round_trip_distinct_from_string  ... ok
test probe_2_nil_atom_round_trip_distinct_from_keyword_nil ... ok
test probe_3_uuid_atom_round_trip_closes_arc_207_false_flag ... ok  ← ARC 207 FALSE-FLAG CLOSED
test probe_4_hashmap_keyword_key_insert_lookup              ... ok
test probe_5_hashset_keyword_insert_contains               ... ok
test probe_6_hashmap_uuid_key_insert_lookup_closes_arc_207 ... ok  ← ARC 207 AT COLLECTION LAYER
```

## Files changed

wat-rs source:
- `src/runtime.rs` (~+50 lines): 3 new value_to_atom arms (Keyword/Nil/Uuid) + doc-comment refresh at 10490 + holon_to_watast 3 new arms (Keyword/Nil/Tag) + statement-length 3 new leaf arms
- `src/check.rs` (~+5 lines): doc-comment updates to `:wat::core::keyword` + `:wat::core::Uuid` is_atomizable entries
- `src/edn_shim.rs` (~+25 lines): holon_ast_to_edn 3 new arms (Keyword/Nil/Tag) + edn_holon_tag_to_ast 3 new reader arms
- `src/hologram.rs` (~+4 lines): find_first_thermometer 3 new leaf-no-therm arms
- `src/lower.rs` (~+6 lines): lower_atom_keyword test assertion updated (Stone 221.3 cascade fix)
- `src/special_forms.rs` (~+8 lines): lookup_returns_some_for_if test assertion updated (Stone 221.3 cascade fix)
- `tests/probe_arc214_slice4_stone2_env_get_trio.rs` (3 lines): doc comment refresh
- `tests/wat_arc201_structured_signature_types.rs` (~+5 lines): module doc refresh (honest Stone 221.4/221.5 note)

New files:
- `tests/wat_arc221_keyword_nil_tag_atomization.rs` (~220 lines): 6 probes
- `docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.4.md` (this file)

**Total: 8 modified source files + 1 new test file + 1 new SCORE doc.**

## STOP triggers

- **STOP-1 (existing wat-rs test regression beyond planned):** TRIGGERED and resolved in-flight. 2 tests (`lower_atom_keyword`, `lookup_returns_some_for_if`) failed — both are Stone 221.3 cascade consequences (keyword() constructor change), not Stone 221.4 regressions. Framed honestly (not "pre-existing"). Fixes mechanical + correct + non-masking. STOP-1 not triggered for Stone 221.4's own changes.
- **STOP-2 (load-bearing probe fails):** DID NOT TRIGGER. All 6 probes pass. Uuid round-trip (Probe 3 + 6) pass — arc 207 false-flag closed.
- **STOP-3 (120 min elapsed):** DID NOT TRIGGER.
- **STOP-4 (holon-rs touched accidentally):** DID NOT TRIGGER. `git -C holon-rs/ diff --name-only` empty.
- **STOP-5 (holon_to_watast mapping unclear):** DID NOT TRIGGER. Tag uses SlotMarker-precedent non-round-trip form; documented explicitly.

## Calibration check

- **Target runtime:** 60-90 min Mode A
- **Upper bound:** 120 min
- **Actual sonnet duration:** ~55 min (reading 4 lineage docs + full relevant file reads + 8 targeted edits + 2 Stone 221.3 cascade test fixes + probe file write + nil syntax correction + verification cycle + SCORE)
- **Within prediction band?** YES — at the lower bound of the 60-90 min target. The pattern from Stones 221.1-221.3 was internalized; cascade sites anticipated; no cascade surprise. Stone 221.3 cascade tests added ~10 min (STOP-1 trigger + investigation + framing + 2 fixes). nil syntax correction added ~5 min.

## Substrate state

- `value_to_atom` now dispatches all primitives to proper HolonAST leaves:
  - `keyword` → `HolonAST::Keyword` (leading colon stripped; PRIM_TAG_KEYWORD distinct vector)
  - `Value::Unit` → `HolonAST::Nil` (PRIM_TAG_NIL distinct vector)
  - `Uuid` → `HolonAST::Bind(Tag("uuid"), String(hex))` (tagged composition; arc 221 doctrine)
- Pre-arc-221 keyword convention `HolonAST::symbol(k.as_str())` retired in `value_to_atom`
- `holon_to_watast` has round-trip-safe arms for Keyword + Nil; Tag is debug-legible (non-round-trip, like SlotMarker)
- EDN wire format round-trips Keyword/Nil/Tag leaves via `#wat-edn.holon/Keyword`, `#wat-edn.holon/Nil`, `#wat-edn.holon/Tag` tagged forms
- Arc 207 false-flag CLOSED: `(:wat::holon::Atom <uuid-val>)` now works; `HashMap<Uuid, V>` insert/lookup verified end-to-end
- `watast_to_holon` (quoted-WAT → HolonAST path) still uses Symbol for WatAST::Keyword — Stone 221.5 scope

## Unblocks

- Stone 221.5 (holon-rs Symbol/String canonical-bytes seed distinction — the remaining pre-arc-221 substrate compromise per Symbol doc comment)
- Stone 221.6 (INSCRIPTION — blocked on arc 222 + arc 223 per spawn-block discipline)
- Arc 222 + arc 223 can now consume `HolonAST::Keyword`, `HolonAST::Nil` leaves via the new edn_shim arms and value_to_atom dispatch
- `HashMap<keyword, V>` and `HashSet<keyword>` are verified end-to-end at runtime
- `HashMap<Uuid, V>` verified end-to-end (arc 207 false-flag closed)

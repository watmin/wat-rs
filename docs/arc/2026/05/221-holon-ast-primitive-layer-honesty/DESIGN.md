# DESIGN — Arc 221 — HolonAST primitive-layer honesty

**Opened:** 2026-05-22
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**First holon-rs touch in ~4 weeks** (per user 2026-05-22 — substrate sat untouched while wat-rs layer matured; the arc 220 atomization investigation surfaced the next layer of substrate work).

## Triggering observation

User question 2026-05-22: *"the atom of a char is.... (:wat::holon::Atom \\N) ?.."*

The investigation surfaced **three related substrate-doctrine gaps** at the HolonAST primitive layer:

1. **Char gap (arc 220):** `:wat::core::Char` has a Value variant + Hash impl (Stone 220.2) but `value_to_atom` has NO Char arm → `(:wat::holon::Atom \\N)` would fail at runtime. The `is_atomizable` check-time predicate also doesn't include Char.
2. **Uuid false-flag (arc 207):** `:wat::core::Uuid` IS in `is_atomizable` but `value_to_atom` has NO Uuid arm — false-flag since arc 207 shipped. `(:wat::holon::Atom <some-uuid>)` would fail at runtime.
3. **Convention-based encoding inside Symbol leaf:** HolonAST collapses Keyword and Nil into `HolonAST::Symbol` via leading-colon convention; collapses Symbol and String canonical-bytes seeds via shared `PRIM_TAG_STRING`. These are substrate-level dishonesties documented in the holon_ast.rs doc-comment itself (lines 53-71).

These are convention-based encodings, not structural distinctions. The "no convention-based encoding" honesty test — the same test that rejected the `String("char:a")` prefix hack for Char during Stone 220.5 investigation — applies here at the substrate layer.

## Doctrine: untagged primitives vs tagged literals

EDN has two distinct categories of literal:

- **Untagged primitive:** `nil`, `true`/`false`, integers, floats, strings, keywords, symbols, **chars** — atomic vocabulary.
- **Tagged:** `#uuid "..."`, `#inst "..."`, `#wat.core/Some N`, `#wat.core/None nil`, etc. — `#tag value` shape.

**Untagged primitives MUST have direct HolonAST leaves** (no convention-based discriminator inside another leaf).

**Tagged literals MUST encode as `Bind(Atom(Symbol("#tag")), payload)`** per arc 216.7 encoding doctrine.

This is the doctrine that closes the open question from arc 216.7: *what shape do non-collection primitives take when there's no existing leaf?* Answer: they get their own leaf (untagged), or they get Bind composition (tagged). No convention-based prefix encoding inside an existing leaf.

## Current HolonAST primitive coverage

| EDN form | Has tag? | Current encoding | Honest encoding |
|---|---|---|---|
| `nil` | no | `Symbol("nil")` (convention-based) | **NEW: `HolonAST::Nil`** |
| `true` / `false` | no | `Bool` | unchanged |
| `42` | no | `I64` | unchanged |
| `3.14` | no | `F64` | unchanged |
| `"hello"` | no | `String` | unchanged structurally; **fix canonical-bytes seed** so `String("x")` and `Symbol("x")` have distinct vectors |
| `:foo` | no | `Symbol(":foo")` (convention-based) | **NEW: `HolonAST::Keyword(":foo")`** |
| `foo` | no | `Symbol("foo")` | unchanged structurally |
| `\a` | no | (no encoding — gap) | **NEW: `HolonAST::Char(char)`** |
| `#uuid "..."` | yes | (no value_to_atom arm — gap) | `Bind(Atom(Symbol("#uuid")), Atom(String(hex)))` |
| `#inst "..."` | yes | (no encoding) | `Bind(Atom(Symbol("#inst")), Atom(String(rfc3339)))` — arc 216 Stone 216.9 |
| `#wat.core/Some N` | yes | (no encoding) | `Bind(Atom(Symbol("#wat.core/Some")), <N atomized>)` — arc 216 Stone 216.8 |

## Phasing

Arc 221 is split into TWO phases. Phase A is the MINIMUM scope required to unblock arc 220 Slice 5 closure. Phase B is doctrine-completeness work that can ship independently (or could be split off as arc 221b at user direction).

### Phase A — Char atomization (unblocks arc 220 Slice 5)

#### Stone 221.1 — holon-rs `HolonAST::Char(char)` leaf

`holon-rs/src/kernel/holon_ast.rs`:
- Add `Char(char)` variant
- Debug arm
- PartialEq arm
- Hash arm (`discriminant` then `char as u32` to bytes)
- Canonical-bytes path: new `PRIM_TAG_CHAR: &str = "char"` constant + write UTF-8 byte payload via `write_atom_payload`
- VSA encoder seed path (mirrors `String` leaf encoding shape)
- Constructor: `pub fn char_(c: char) -> Self` (underscore to avoid keyword collision)

Tests in `holon-rs/src/kernel/holon_ast.rs::tests`:
- `char_leaf_round_trip` — construct + Hash + canonical_bytes + parse back
- `char_distinct_from_string` — `Char('a')` produces distinct vector from `String("a")` and `Symbol(\"a\")`
- `char_bmp_only` — supplementary plane (`\u{1F600}`) — defer to wat-rs side; holon-rs accepts full `char` type; BMP gate is wat-rs Stone 220.2's responsibility

#### Stone 221.2 — wat-rs `value_to_atom` Char + Uuid arms + `is_atomizable` Char

`wat-rs/src/runtime.rs::value_to_atom` (~13800):
- Char arm: `Value::wat__core__Char(c) => HolonAST::Char(c)` (uses Stone 221.1)
- Uuid arm: `Value::wat__core__Uuid(u) => Bind(Atom(Symbol("#uuid")), Atom(String(u.to_string())))` (tagged-form per doctrine; closes arc 207 false-flag)

`wat-rs/src/check.rs::is_atomizable` (~3623):
- Add `| ":wat::core::Char"` (Uuid stays where it is; now actually works at runtime)

Tests: `tests/wat_arc221_atom_primitives.rs` — 6 probes:
1. `(:wat::holon::Atom \\a)` round-trips
2. `(:wat::holon::Atom <uuid-val>)` round-trips
3. `HashMap<Char, i64>` insert + lookup
4. `HashMap<Uuid, String>` insert + lookup
5. `HashSet<Char>` insert + contains?
6. `HashSet<Uuid>` insert + contains?

Cross-verify: `cargo test --release --test wat_arc220_char` still 10/10 PASS (Stone 220.2 unchanged); `cargo test --release --lib -p wat` 827/0 PASS (no regression).

**Phase A unblocks arc 220 Slice 5.** Arc 220 INSCRIPTION can honestly state Char is fully atomizable.

### Phase B — Substrate-doctrine completeness (independent of arc 220)

These stones can ship in any order after Phase A; they don't block arc 220 closure but they DO close the deeper doctrine gaps surfaced during this investigation.

#### Stone 221.3 — holon-rs `HolonAST::Keyword` + `HolonAST::Nil` + `HolonAST::Tag` leaves

`holon-rs/src/kernel/holon_ast.rs`:
- Add `Keyword(Arc<str>)` variant — content is the keyword body WITHOUT leading colon
- Add `Nil` variant (no payload)
- Add `Tag(Arc<str>)` variant — content is the tag identifier WITHOUT leading `#`
- Per-variant Debug/PartialEq/Hash arms
- Canonical-bytes paths with distinct type tags: `PRIM_TAG_KEYWORD: &str = "keyword"`, `PRIM_TAG_NIL: &str = "nil"`, `PRIM_TAG_TAG: &str = "tag"`
- VSA encoder seed paths

Holon-rs migration ripple in same stone:
- All sites producing `Symbol(":x")` for keywords → `Keyword("x")`
- All sites producing `Symbol("nil")` for nil → `Nil`
- All sites producing `Symbol("#x")` for tag dispatch markers → `Tag("x")`
- `HolonAST::keyword()` constructor produces `Keyword` (not `Symbol(":x")`)

#### Stone 221.4 — wat-rs consumer ripple

`wat-rs` consumer sweep across `src/`:
- All `HolonAST::Symbol(":foo")` produce-sites for keywords → `HolonAST::Keyword("foo")`
- All `HolonAST::Symbol("nil")` produce-sites → `HolonAST::Nil`
- All `HolonAST::Symbol("#foo")` produce-sites for tag dispatch → `HolonAST::Tag("foo")`
- All match arms on the old conventions get migrated to the new variants
- `value_to_atom::Value::wat__core__keyword(k)` arm → `HolonAST::Keyword(k)` (strip leading colon at boundary)
- `value_to_atom::Value::Unit` arm → `HolonAST::Nil`
- `value_to_atom::Value::wat__core__Uuid(u)` arm — UPDATE per doctrine correction: `Bind(Tag("uuid"), String(u.to_string()))` (was tentatively planned as `Bind(Atom(Symbol("#uuid")), Atom(String(...)))` — the Atom wraps were fabricated identity layers; corrected to bare leaves)

Per `feedback_substrate_as_teacher`: the compiler is the brief. Expected cascade is substrate-wide; iterate until `cargo test --release --lib -p wat` is green again. Test count may grow from the new variant clarity.

#### Stone 221.5 — Symbol/String canonical-bytes seed distinction

`holon-rs/src/kernel/holon_ast.rs`:
- Mint distinct `PRIM_TAG_SYMBOL: &str = "symbol"` constant
- Symbol leaf's canonical-bytes write uses `PRIM_TAG_SYMBOL` (not `PRIM_TAG_STRING`)
- VSA vector for `Symbol("x")` and `String("x")` becomes distinct

Test in `holon-rs/src/kernel/holon_ast.rs::tests`:
- `symbol_string_canonical_bytes_distinct` — `Symbol("x").canonical_bytes() != String("x").canonical_bytes()`
- `symbol_string_vectors_distinct` — vector identities differ for matched content

This is the cleanest fix to the existing collision documented at lines 67-71 of holon_ast.rs.

#### Stone 221.6 — INSCRIPTION + cross-references

- `INSCRIPTION.md` — arc 221 closure narrative
- CLIFFNOTES Currently update — articulates the untagged-primitive vs tagged-literal doctrine refinement
- Cross-references to arc 207 (forward-correction of value_to_atom gap) + arc 216.7 (encoding doctrine) + arc 220 Stone 220.2 (Char minting; this arc completes the atomization)
- 058 changelog row

## What this arc does NOT do

- Touch Bundle / Bind / Permute / Thermometer / Blend / Atom / SlotMarker (composite variants unchanged)
- Add HolonAST leaves for Uuid or Inst or Duration (these are TAGGED in EDN spec; Bind composition is honest per arc 216.7)
- **Add HolonAST leaves for List / Vector / Set / Map / Tuple** (these are COLLECTIONS; per arc 216.7 doctrine they encode via Bundle + Bind composition — Bundle for set-shape, Bundle+positional-Bind for indexed sequences, Bundle of Bind pairs for maps. No new HolonAST variants needed; the existing composite primitives (Bundle + Bind + Permute) handle all multi-element shapes via composition. The wat-runtime layer keeps native Rust containers (Vec, HashSet, HashMap, LinkedList, Tuple) for performance + ergonomics; the HolonAST encoding layer uses Bundle composition; the EDN wire layer uses spec syntax `(...) [...] #{...} {...}`. Three layers, each appropriate for its purpose.)
- Touch wat-edn (wire formats already handle EDN literals; only HolonAST encoding side needs lifting)
- Modify the `HolonAST::symbol(":foo")` constructor (kept for `from_parts_unchecked` callers + raw-symbol use cases; just migrates the canonical produce-sites for keyword/nil away from it)
- Edit arc 207's INSCRIPTION (per `feedback_inscription_immutable` — historical record stays; arc 221 closes forward)

## EDN-spec coverage after arc 221 Phase B ships

| EDN form | HolonAST encoding | Coverage |
|---|---|---|
| `nil` | `Nil` (new in 221.3) | leaf |
| `true` / `false` | `Bool` | leaf |
| integers | `I64` | leaf |
| floats | `F64` | leaf |
| strings | `String` | leaf |
| keywords | `Keyword("foo")` (new in 221.3; no leading colon in payload) | leaf |
| symbols | `Symbol` | leaf |
| chars | `Char` (new in 221.1) | leaf |
| `(1 2 3)` list | `Bundle([1, 2, 3])` composition | composite |
| `[1 2 3]` vector | `Bundle([Bind(I64(0), 1), Bind(I64(1), 2), ...])` positional composition | composite |
| `#{a b c}` set | `Bundle([a, b, c])` set-shape composition | composite |
| `{a 1 b 2}` map | `Bundle([Bind(a, 1), Bind(b, 2)])` map-shape composition | composite |
| `#tag value` tagged | `Bind(Tag("tag"), <value-leaf>)` composition (Tag new in 221.3; no leading `#` in payload) | composite |
| `#uuid "..."` | `Bind(Tag("uuid"), String(hex))`; Stone 221.2 value_to_atom Uuid arm | composite |
| `#inst "..."` | `Bind(Tag("inst"), String(rfc3339))`; arc 216 Stone 216.9 | composite |
| `#wat.core/Some N` | `Bind(Tag("wat.core/Some"), <N-leaf>)`; arc 216 Stone 216.8 | composite |
| `#wat.core/None nil` | `Bind(Tag("wat.core/None"), Nil)`; arc 216 Stone 216.8 | composite |

**Arc 221 Phase B closes EDN-syntax coverage on HolonAST.** Every EDN literal type maps cleanly to either a leaf variant (untagged primitives) or a composition of existing composite primitives (Bundle + Bind + Permute for collections + tagged). No further HolonAST variants required.

**16 HolonAST variants total after arc 221 ships:**

```
LEAVES (untagged primitive literals — 9):
  Nil, Bool, I64, F64, String, Symbol, Keyword, Char, Tag

COMPOSITES (3):
  Bundle  — collections (lists, vectors, sets, maps via composition)
  Bind    — key-value / positional / tag-payload
  Permute — positional shift

SPECIAL (kept for VSA semantics, not EDN-spec — 4):
  Atom         — opaque-identity wrap (use INTENTIONALLY, not ceremonially)
  Thermometer  — gradient encoding
  Blend        — weighted sum
  SlotMarker   — substrate-internal
```

**Deliberately out-of-spec:** BigInt + BigDecimal. Wat numeric tower is i64 + f64 only per CLIFFNOTES + `crates/wat-edn/src/edn.rs` ("EDN BigInt / BigDecimal — wat numeric tower is i64 + f64 only"). Arc 221 does not change this scope.

## Forward-correction 2026-05-22 (notation refinement)

The original DESIGN draft used `Bind(Atom(Symbol("#tag")), Atom(<payload>))` as the tagged-literal encoding shape. The user surfaced (during the doctrine dialogue captured in `INTERSTITIAL-REALIZATIONS.md` 2026-05-22 entry) that this notation conflates two distinct things:

- **Verb `:wat::holon::Atom`** — wat-level dispatcher that takes a Value and produces the appropriate HolonAST node (leaf for primitives, opaque-identity wrap for HolonAST inputs, structural lowering for WatAST). For a String value, the verb produces `HolonAST::String`, NOT `HolonAST::Atom(HolonAST::String(...))`.
- **Variant `HolonAST::Atom(Arc<HolonAST>)`** — opaque-identity wrap. Per the doc at `holon_ast.rs:88-96`: *"`Atom(Atom(x))` differs from `Atom(x)` differs from `x` — quote-wrapping is repeatable and meaningful."* The wrap MEANS something at the VSA vector level.

**Wrapping a leaf in `HolonAST::Atom(...)` adds an opaque-identity dimension that wasn't in the EDN source form.** The bare-leaf form is honest. The Atom-wrap is reserved for explicit `(:wat::holon::Atom <holon-ast-value>)` dispatch (when the user EXPLICITLY wants opaque identity).

**Corrected tagged-literal encoding examples:**

```
\a                   → Char('a')                                  ; leaf
:foo                 → Keyword("foo")                             ; leaf (no colon in payload)
nil                  → Nil                                        ; leaf
#whatever :foobar    → Bind(Tag("whatever"), Keyword("foobar"))   ; tag + keyword-leaf
#uuid "..."          → Bind(Tag("uuid"), String(hex))             ; tag + string-leaf
#inst "..."          → Bind(Tag("inst"), String(rfc3339))         ; same shape
#wat.core/Some 42    → Bind(Tag("wat.core/Some"), I64(42))        ; tag + i64-leaf
#wat.core/None nil   → Bind(Tag("wat.core/None"), Nil)            ; tag + nil-leaf
#wat.core/Ok :win    → Bind(Tag("wat.core/Ok"), Keyword("win"))   ; tag + keyword-leaf
```

This forward-corrects arc 216 Stones 216.8 + 216.9 (pending) — they should ship the cleaner `Bind(Tag, payload)` shape rather than the inscribed `Bind(Atom("#tag"), payload)` doctrine notation (which was always ambiguous about which "Atom" was meant).

Per `feedback_inscription_immutable`: arc 216 Stone 216.7 INSCRIPTION stays unchanged as historical record. This DESIGN forward-corrects for Stones 216.8 + 216.9 going forward. Arc 221's INSCRIPTION cites the lineage.

## Calibration

| Stone | Phase | Predicted | Notes |
|---|---|---|---|
| 221.1 | A | 30-60 min | holon-rs single-variant addition; new PRIM_TAG_CHAR; constructor + 3 tests |
| 221.2 | A | 20-30 min | wat-rs value_to_atom 2 arms + is_atomizable 1 line + 6 probes |
| 221.3 | B | 90-150 min | holon-rs 3 variants (Keyword + Nil + Tag) + migration ripple within holon-rs |
| 221.4 | B | 60-90 min | wat-rs consumer ripple (substrate-as-teacher cascade per arc 213's pattern); includes value_to_atom Uuid → Bind(Tag, String) per 2026-05-22 doctrine correction |
| 221.5 | B | 30-45 min | seed type-tag distinction + vector-identity probes |
| 221.6 | B | 30 min | paperwork |

**Phase A total:** ~50-90 min (Char atomization complete; arc 220 unblocked).
**Phase B total:** ~3-5 hours (substrate-doctrine completeness; can ship over multiple sessions).

## Unblocks

After Phase A:
- arc 220 Slice 5 (INSCRIPTION + USER-GUIDE) — can honestly state Char + Uuid are fully atomizable

After Phase B:
- Arc 216 Stone 216.8 + 216.9 (tagged sum-type encoding + #inst verify + #uuid verify + #wat.time/Duration mint) — gets cleaner Keyword/Nil discriminators
- Any future "primitive-as-atom" arcs — predicate honesty becomes the floor
- Cross-language interop (wat<>clj per arc 217's vision) — substrate matches EDN spec precisely

## Cross-references

- `holon-rs/src/kernel/holon_ast.rs:51-132` — the substrate enum that needs extension; doc-comment at lines 53-71 explicitly admits the Symbol/Keyword/Nil collapse + Symbol/String seed collapse
- `holon-rs/src/kernel/holon_ast.rs:208-244` — Hash impl (discriminant + per-variant)
- `holon-rs/src/kernel/holon_ast.rs:486-505` — canonical-bytes type tags (PRIM_TAG_STRING / I64 / F64 / BOOL; missing CHAR + KEYWORD + NIL + SYMBOL)
- arc 207 (`docs/arc/2026/05/207-uuid-typed-primitive/INSCRIPTION.md`) — minted `:wat::core::Uuid` Value but did not extend value_to_atom; closed forward by Stone 221.2 (per `feedback_inscription_immutable`)
- arc 216 Stone 216.7 — encoding doctrine inscription; arc 221 articulates the doctrine's untagged-primitive axis
- arc 220 Stone 220.2 (`docs/arc/2026/05/220-wat-core-edn-primitive-completeness/SCORE-STONE-220.2.md`) — minted `:wat::core::Char` Value + Hash; this arc closes the value_to_atom + is_atomizable gap
- CLIFFNOTES § Encoding doctrine — refines to articulate untagged-primitive vs tagged-literal distinction post arc 221

## Open questions for the DESIGN review

1. **Phase A only, or A+B in one arc?** Phase A unblocks arc 220 Slice 5 directly. Phase B is doctrine-completeness that can be its own arc (221b) if scope discipline prefers tighter arcs.
2. **Keyword + Tag storage form: with or without leading sigil?** RESOLVED 2026-05-22: store WITHOUT leading colon (Keyword) and WITHOUT leading `#` (Tag) — keeps the type tag at the canonical-bytes level distinct + matches the EDN-spec parser productions (the leading sigil is the parser dispatch, not part of the identifier).
3. **Migrate `HolonAST::symbol(":foo")` callers?** Stone 221.3 + 221.4 sweep produce-sites; keep the keyword-via-symbol constructor available for `from_parts_unchecked` / lower-level callers but migrate the canonical produce-sites away from it. Same for tag-via-symbol.
4. **Vector-identity regression risk in Stone 221.5:** any test fixture that assumes `Symbol("x")` and `String("x")` produce equal vectors will break. Audit scope: grep for `String(":` and `Symbol(":` co-occurrence in vector-identity tests; flag as Stone 221.5 pre-flight item.

These have been resolved as of 2026-05-22 dialogue. Stone 221.1 BRIEF can proceed.

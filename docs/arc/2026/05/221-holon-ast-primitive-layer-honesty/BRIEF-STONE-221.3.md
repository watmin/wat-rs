# BRIEF — Arc 221 Stone 221.3 — `HolonAST::Keyword` + `Nil` + `Tag` leaves in holon-rs

**Stone scope (sonnet portion):** mint three new HolonAST variants in `holon-rs/src/kernel/holon_ast.rs` — `Keyword(Arc<str>)`, `Nil`, `Tag(Arc<str>)`. Update `keyword()` constructor to produce `HolonAST::Keyword` instead of leading-colon-Symbol. Add `nil()` + `tag()` constructors. Add `as_keyword()` + `as_tag()` accessors. One consumer ripple in `holon-rs/src/memory/reckoner.rs:1103` (one `matches!` arm flip). 11 tests minimum (3 per variant + 2 distinct-from-convention). **Holon-rs ONLY this stone — wat-rs untouched.**
**Type:** Sonnet Mode A.
**Time budget:** 60-90 min target; 120 min STOP.
**Depends on:** Stone 221.1 (shipped at holon-rs `243eded`) — establishes the additive-leaf pattern + cascade-arm sweep.
**Calibration:** Per `feedback_stone_briefs_cite_prior_score`, read **Stone 221.1's SCORE** at `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.1.md` (8/8 PASS, ~25 min, under 30-60 band). This stone is 3× the variants + 1 consumer flip; band ~2.5× = 60-90.
**Unblocks:** Stone 221.4 (wat-rs ripple uses `Bind(Tag("uuid"), String(hex))` shape per arc 221 doctrine correction); enables arc 223 + arc 222 to consume these leaves.

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/holon-rs/`** (NOT wat-rs!)
- Branch: holon-rs is on `main`; commit there (orchestrator commits — sonnet does NOT commit).
- Linux only; no `--no-verify`.
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch wat-rs files this stone.
- DO NOT touch Stone 221.5's scope (Symbol/String canonical-bytes seed collision — that's a separate substrate-doctrine fix).

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Current HolonAST enum after Stone 221.1

`holon-rs/src/kernel/holon_ast.rs:51-141`:

**13 variants today:** Symbol, String, I64, F64, Bool, Char, Atom, Bind, Bundle, Permute, Thermometer, Blend, SlotMarker.

Doc comment at line 47: *"The universal AST. Thirteen variants. Closed under itself."* → updates to **"Sixteen variants"** after this stone.

The `Symbol` doc comment at lines 53-71 currently documents the Symbol/Keyword/Nil convention-based collapse. After this stone, Symbol's doc updates to describe ONLY the bare-identifier case (Keyword + Nil become their own variants).

### Lines to touch

| Section | Line | Change |
|---|---|---|
| Enum doc count | 47 | "Thirteen" → "Sixteen" |
| Symbol doc | 53-71 | Rewrite — Symbol is now ONLY bare-identifier (drop keyword/nil convention text) |
| Char (existing — model) | 87-94 | This is your pattern; mirror for new variants |
| Enum body | after line 94 (after Char) | Add `Keyword(Arc<str>)`, `Nil`, `Tag(Arc<str>)` |
| Debug impl | 143-178 | Add 3 arms |
| PartialEq impl | 180-215 | Add 3 arms |
| Hash impl | 219-256 | Add 3 arms |
| Constructor `keyword()` | 292-305 | REWRITE — produce `HolonAST::Keyword(name.into())` (strip leading `:` if present) |
| Constructors `nil()` + `tag()` | after line 290 (after `char_`) | Add 2 new constructors |
| Accessors `as_keyword()` + `as_tag()` | after line 386 (after `atom_inner`) | Add 2 new accessors (mirror `as_symbol` shape) |
| `template()` cascade | ~399 | Add 3 leaf-passthrough arms |
| `collect_slots()` cascade | ~449 | Add 3 no-op leaf arms |
| `collect_ranges()` cascade | ~477 | Add 3 no-op leaf arms |
| `PRIM_TAG_*` constants | 522-526 | Add 3 constants (snake-case strings) |
| `canonical_edn_holon()` | 542-554 | Add 3 arms |
| `encode()` cascade | 623-646 | Add 3 leaf_seed arms |
| Tests block | starts at 716 | Add ≥11 new tests |

(Cascade arms — template / collect_slots / collect_ranges / encode — were Stone 221.1's Delta 1; the Rust exhaustive-match compiler catches all of them. They are part of the scope for this stone.)

### Consumer ripple (holon-rs ONLY)

`holon-rs/src/memory/reckoner.rs:1103`:

```rust
assert!(matches!(r.label_ast(labels[0]), Some(HolonAST::Symbol(_))));
```

After this stone, `label_ast` returns `Some(HolonAST::Keyword(_))` because the test creates labels via `HolonAST::keyword("Win")` which now produces `HolonAST::Keyword("Win".into())`. Flip the matches arm:

```rust
assert!(matches!(r.label_ast(labels[0]), Some(HolonAST::Keyword(_))));
```

Other `HolonAST::keyword()` call sites in `reckoner.rs` (lines 887, 900, 926, 1073, 1093-95, 1112) are constructor-only and need no edit — they continue to work, just producing the new variant.

## Your scope (sonnet)

### 1. Add 3 variants to the enum

Place after `Char` (line 94):

```rust
/// Keyword leaf — Clojure-EDN keyword form `:foo`, `:wat.core/Some`.
/// Stored content is the keyword name MINUS the leading colon
/// (`Keyword("foo")` represents `:foo`).
///
/// Distinct from `Symbol("foo")` and `String("foo")` at the type level
/// AND canonical-bytes level — `PRIM_TAG_KEYWORD` seeds a distinct
/// vector identity.
///
/// Replaces the pre-arc-221 leading-colon-in-Symbol convention:
/// `Symbol(":foo")` is no longer the keyword encoding; `Keyword("foo")` is.
Keyword(Arc<str>),

/// Nil leaf — the EDN `nil` literal. Distinct from any other primitive
/// at the type level AND canonical-bytes level via `PRIM_TAG_NIL`.
///
/// Replaces the pre-arc-221 `Symbol("nil")` convention: the nil literal
/// is now its own variant; `Symbol("nil")` would be the bare identifier
/// "nil" (different semantic).
Nil,

/// Tag leaf — EDN tagged-literal dispatch marker, e.g. `#uuid` parses
/// to `Tag("uuid")`. Composes with payload via `Bind(Tag(t), payload)`
/// per arc 221 doctrine — bare-leaf payload, no Atom wrapping.
///
/// Stored content is the tag name MINUS the leading `#`
/// (`Tag("uuid")` represents `#uuid`).
///
/// Distinct from `Symbol("uuid")` and `Symbol("#uuid")` byte-for-byte
/// via `PRIM_TAG_TAG`.
Tag(Arc<str>),
```

### 2. Update the enum-level doc count

Line 47: `"Thirteen variants"` → `"Sixteen variants"`.

### 3. Rewrite Symbol's doc comment (lines 53-71)

Symbol now means ONLY bare identifier. The keyword/nil convention text moves OUT:

```rust
/// Bare-identifier leaf — e.g. `Symbol("foo")` represents the
/// identifier `foo` (resolves to its binding at evaluation time:
/// function name, binding name, argument name).
///
/// Distinct from `Keyword("foo")` (represents `:foo`) and `String("foo")`
/// (string literal `"foo"`) at the type level. Currently shares the
/// `PRIM_TAG_STRING` canonical-bytes seed with `String` — this is a
/// pre-arc-216 accepted-collision documented at the PRIM_TAG block;
/// Stone 221.5 resolves it. The collision is acceptable today because
/// type-level distinctness suffices for the variant matching needs.
Symbol(Arc<str>),
```

### 4. Add Debug arms (after line 151, the Char arm)

```rust
HolonAST::Keyword(s) => f.debug_tuple("Keyword").field(&&**s).finish(),
HolonAST::Nil => f.debug_tuple("Nil").finish(),
HolonAST::Tag(s) => f.debug_tuple("Tag").field(&&**s).finish(),
```

### 5. Add PartialEq arms (after line 188, the Char arm)

```rust
(HolonAST::Keyword(a), HolonAST::Keyword(b)) => a == b,
(HolonAST::Nil, HolonAST::Nil) => true,
(HolonAST::Tag(a), HolonAST::Tag(b)) => a == b,
```

### 6. Add Hash arms (after line 228, the Char arm)

```rust
HolonAST::Keyword(s) => s.hash(state),
HolonAST::Nil => { /* discriminant alone suffices */ }
HolonAST::Tag(s) => s.hash(state),
```

(The outer `std::mem::discriminant(self).hash(state)` at line 221 already fires; the Nil arm needs no payload contribution because the discriminant uniquely identifies it.)

### 7. Rewrite the `keyword()` constructor (lines 292-305)

```rust
/// Construct a `Keyword` leaf with the leading colon stripped.
/// `HolonAST::keyword("foo")` and `HolonAST::keyword(":foo")` produce
/// the same `Keyword("foo")` variant.
///
/// Per arc 221: keyword is its own variant (replaces the pre-arc-221
/// convention of encoding `:foo` as `Symbol(":foo")`).
pub fn keyword(name: &str) -> Self {
    let stored: Arc<str> = if let Some(stripped) = name.strip_prefix(':') {
        Arc::from(stripped)
    } else {
        Arc::from(name)
    };
    HolonAST::Keyword(stored)
}
```

### 8. Add `nil()` + `tag()` constructors (after `char_` at line 290)

```rust
/// Construct a `Nil` leaf — the EDN nil literal.
pub fn nil() -> Self {
    HolonAST::Nil
}

/// Construct a `Tag` leaf with the leading `#` stripped.
/// `HolonAST::tag("uuid")` and `HolonAST::tag("#uuid")` produce the
/// same `Tag("uuid")` variant. Compose with payload via
/// `HolonAST::bind(HolonAST::tag("uuid"), HolonAST::string(hex))` per
/// arc 221 doctrine.
pub fn tag(name: &str) -> Self {
    let stored: Arc<str> = if let Some(stripped) = name.strip_prefix('#') {
        Arc::from(stripped)
    } else {
        Arc::from(name)
    };
    HolonAST::Tag(stored)
}
```

### 9. Add `as_keyword()` + `as_tag()` accessors (after `atom_inner` at line 386)

```rust
/// If this is a `Keyword` leaf, return the content (without leading colon).
pub fn as_keyword(&self) -> Option<&str> {
    match self {
        HolonAST::Keyword(s) => Some(s.as_ref()),
        _ => None,
    }
}

/// If this is a `Tag` leaf, return the content (without leading `#`).
pub fn as_tag(&self) -> Option<&str> {
    match self {
        HolonAST::Tag(s) => Some(s.as_ref()),
        _ => None,
    }
}
```

### 10. Add cascade arms

**`template()` (around line 399)** — 3 leaf passthrough arms (mirror Char's arm):

```rust
HolonAST::Keyword(_) | HolonAST::Nil | HolonAST::Tag(_) => self.clone(),
```

(Or 3 separate arms if cleaner; pattern matches Char's existing arm — leaves return self.)

**`collect_slots()` (around line 449)** — 3 no-op arms:

```rust
HolonAST::Keyword(_) | HolonAST::Nil | HolonAST::Tag(_) => {}
```

**`collect_ranges()` (around line 477)** — 3 no-op arms (same shape).

**`encode()` (around line 623)** — 3 leaf_seed arms:

```rust
HolonAST::Keyword(s) => {
    let seed = leaf_seed(PRIM_TAG_KEYWORD, s.as_bytes(), vm.global_seed());
    // ... rest of leaf_seed pattern from Char (line 646)
}
HolonAST::Nil => {
    let seed = leaf_seed(PRIM_TAG_NIL, &[], vm.global_seed());
    // ... rest
}
HolonAST::Tag(s) => {
    let seed = leaf_seed(PRIM_TAG_TAG, s.as_bytes(), vm.global_seed());
    // ... rest
}
```

(Read lines 623-646 to see the exact `leaf_seed` → Vector pattern; mirror it.)

### 11. Add PRIM_TAG constants (after line 526)

```rust
const PRIM_TAG_KEYWORD: &str = "keyword";
const PRIM_TAG_NIL: &str = "nil";
const PRIM_TAG_TAG: &str = "tag";
```

### 12. Add canonical_edn_holon arms (after line 554)

```rust
HolonAST::Keyword(s) => write_atom_payload(&mut out, PRIM_TAG_KEYWORD, s.as_bytes()),
HolonAST::Nil => write_atom_payload(&mut out, PRIM_TAG_NIL, &[]),
HolonAST::Tag(s) => write_atom_payload(&mut out, PRIM_TAG_TAG, s.as_bytes()),
```

### 13. Fix consumer ripple in `reckoner.rs:1103`

```rust
// BEFORE:
assert!(matches!(r.label_ast(labels[0]), Some(HolonAST::Symbol(_))));
// AFTER:
assert!(matches!(r.label_ast(labels[0]), Some(HolonAST::Keyword(_))));
```

### 14. Tests (≥11 new)

Add to the existing `#[cfg(test)] mod tests` block (starts at line 716; mirror existing test shape including the Char tests added in Stone 221.1 around line 1159):

**Round-trip tests (3):**

```rust
#[test]
fn keyword_leaf_round_trip() {
    let h = HolonAST::keyword("foo");
    assert_eq!(h, HolonAST::Keyword(Arc::from("foo")));
    // Leading colon stripped
    assert_eq!(HolonAST::keyword(":foo"), HolonAST::keyword("foo"));
    // Hash determinism
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    h.hash(&mut h1);
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    HolonAST::keyword("foo").hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn nil_leaf_round_trip() {
    let h = HolonAST::nil();
    assert_eq!(h, HolonAST::Nil);
    // Hash determinism
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    h.hash(&mut h1);
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    HolonAST::nil().hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn tag_leaf_round_trip() {
    let h = HolonAST::tag("uuid");
    assert_eq!(h, HolonAST::Tag(Arc::from("uuid")));
    // Leading # stripped
    assert_eq!(HolonAST::tag("#uuid"), HolonAST::tag("uuid"));
}
```

**Distinct-from-Symbol tests (3) — load-bearing for substrate doctrine:**

```rust
#[test]
fn keyword_distinct_from_symbol() {
    let kw_bytes = canonical_edn_holon(&HolonAST::keyword("foo"));
    let sym_bytes = canonical_edn_holon(&HolonAST::symbol("foo"));
    assert_ne!(kw_bytes, sym_bytes,
        "Keyword(\"foo\") and Symbol(\"foo\") MUST differ in canonical bytes");
}

#[test]
fn nil_distinct_from_symbol_nil() {
    // The pre-arc-221 convention encoded nil as Symbol("nil").
    // Now Nil MUST differ from Symbol("nil") byte-for-byte.
    let nil_bytes = canonical_edn_holon(&HolonAST::nil());
    let sym_nil_bytes = canonical_edn_holon(&HolonAST::symbol("nil"));
    assert_ne!(nil_bytes, sym_nil_bytes,
        "Nil and Symbol(\"nil\") MUST differ in canonical bytes");
}

#[test]
fn tag_distinct_from_symbol() {
    let tag_bytes = canonical_edn_holon(&HolonAST::tag("uuid"));
    let sym_bytes = canonical_edn_holon(&HolonAST::symbol("uuid"));
    let sym_hashed_bytes = canonical_edn_holon(&HolonAST::symbol("#uuid"));
    assert_ne!(tag_bytes, sym_bytes,
        "Tag(\"uuid\") and Symbol(\"uuid\") MUST differ in canonical bytes");
    assert_ne!(tag_bytes, sym_hashed_bytes,
        "Tag(\"uuid\") and Symbol(\"#uuid\") MUST differ in canonical bytes");
}
```

**Cross-variant distinctness (3) — guards against PRIM_TAG collisions among new leaves:**

```rust
#[test]
fn keyword_distinct_from_nil() {
    let kw_bytes = canonical_edn_holon(&HolonAST::keyword("nil"));
    let nil_bytes = canonical_edn_holon(&HolonAST::nil());
    assert_ne!(kw_bytes, nil_bytes,
        "Keyword(\"nil\") and Nil MUST differ in canonical bytes");
}

#[test]
fn tag_distinct_from_keyword() {
    let tag_bytes = canonical_edn_holon(&HolonAST::tag("foo"));
    let kw_bytes = canonical_edn_holon(&HolonAST::keyword("foo"));
    assert_ne!(tag_bytes, kw_bytes,
        "Tag(\"foo\") and Keyword(\"foo\") MUST differ in canonical bytes");
}

#[test]
fn nil_distinct_from_bool() {
    let nil_bytes = canonical_edn_holon(&HolonAST::nil());
    let true_bytes = canonical_edn_holon(&HolonAST::bool_(true));
    let false_bytes = canonical_edn_holon(&HolonAST::bool_(false));
    assert_ne!(nil_bytes, true_bytes);
    assert_ne!(nil_bytes, false_bytes);
}
```

**Accessor tests (2):**

```rust
#[test]
fn as_keyword_returns_content_without_colon() {
    assert_eq!(HolonAST::keyword("foo").as_keyword(), Some("foo"));
    assert_eq!(HolonAST::keyword(":foo").as_keyword(), Some("foo"));
    assert_eq!(HolonAST::symbol("foo").as_keyword(), None);
    assert_eq!(HolonAST::nil().as_keyword(), None);
}

#[test]
fn as_tag_returns_content_without_hash() {
    assert_eq!(HolonAST::tag("uuid").as_tag(), Some("uuid"));
    assert_eq!(HolonAST::tag("#uuid").as_tag(), Some("uuid"));
    assert_eq!(HolonAST::symbol("uuid").as_tag(), None);
}
```

(11 tests minimum. Sonnet may add more if a useful edge case surfaces.)

### Verification (must run before SCORE)

From `/home/watmin/work/holon/holon-rs/`:

```
cargo build --release
cargo test --release
cargo clippy --release -- -D warnings
```

All three must be clean. Pre-existing warning baseline (if any) — surface count; don't gate this stone on pre-existing warnings.

**Wat-rs build NOT required this stone.** No wat-rs changes happen here.

**Write `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.3.md`** mirroring SCORE-STONE-221.1 shape (~10 rows per EXPECTATIONS scorecard).

## STOP triggers

- **STOP-1 (existing holon-rs test regression unrelated to reckoner.rs:1103):** if `cargo test --release` fail count goes UP beyond the single planned `reckoner.rs:1103` flip → diagnostic + report. Anything else suggests an undiscovered consumer assumed `keyword()` returned `Symbol`.
- **STOP-2 (canonical_bytes distinct-from tests fail):** if ANY of the 6 distinct-from-* tests FAIL, the PRIM_TAG constants aren't differentiating — diagnostic + report.
- **STOP-3 (120 min elapsed):** wall-clock STOP.
- **STOP-4 (wat-rs touched accidentally):** if `git -C /home/watmin/work/holon/wat-rs/ diff --name-only` shows any non-paperwork changes from this stone, STOP and report.
- **STOP-5 (more than ONE matches! arm flip in reckoner.rs):** if you find additional `HolonAST::Symbol(_)` patterns in `reckoner.rs` that need to become `HolonAST::Keyword(_)`, STOP and report — pre-flight grep found only line 1103.

## Out-of-scope

- wat-rs changes (Stone 221.4 — separate stone; will use new variants via `Bind(Tag("uuid"), String(hex))` shape per arc 221 doctrine)
- Symbol/String canonical-bytes seed distinction (Stone 221.5 — explicit substrate-doctrine fix)
- Any other `HolonAST::keyword()` call site outside `reckoner.rs:1103` (none expected per pre-flight grep)
- INSCRIPTION (Stone 221.6)
- Migration of any pre-existing `Symbol(":foo")` patterns elsewhere in holon-rs to `Keyword("foo")` (out of scope; surfaces in consumer arcs if needed)
- Updates to BOOK or USER-GUIDE (Stone 221.6 INSCRIPTION)

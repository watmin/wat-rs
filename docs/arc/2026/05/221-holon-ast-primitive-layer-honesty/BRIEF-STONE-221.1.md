# BRIEF — Arc 221 Stone 221.1 — `HolonAST::Char` leaf in holon-rs

**Stone scope (sonnet portion):** mint `HolonAST::Char(char)` variant in `holon-rs/src/kernel/holon_ast.rs`. Single-variant addition + 5 arms (Debug / PartialEq / Hash / canonical_edn_holon / constructor) + `PRIM_TAG_CHAR` constant + 3 tests. **Holon-rs ONLY this stone — wat-rs untouched.**
**Type:** Sonnet Mode A.
**Time budget:** 30-60 min target; 90 min STOP.
**Depends on:** DESIGN-221 (commit `d317c02`) + the 2026-05-22 INTERSTITIAL entry.
**Calibration:** 14 stones at-or-below band across recent arcs; first holon-rs touch in ~4 weeks. Band 30-60 reflects single-variant addition + reading the existing arms to mirror their shape.
**Unblocks:** Stone 221.2 (wat-rs value_to_atom Char + Uuid arms + is_atomizable Char) which unblocks arc 220 Slice 5 closure.

## Working dir + constraints

- **Working dir for this stone: `/home/watmin/work/holon/holon-rs/`** (NOT wat-rs!)
- This is a SEPARATE workspace from wat-rs. cd carefully; use `git -C` from wat-rs OR `cd /home/watmin/work/holon/holon-rs/` first.
- Branch: same as wat-rs's current — but check `git -C /home/watmin/work/holon/holon-rs/ branch` to confirm the active branch. If you find yourself on a different branch in holon-rs, surface that as a question before committing anything.
- Linux only; Zero Mutex (holon-rs doesn't typically use Mutex; the discipline still applies); no `--no-verify`
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch wat-rs files this stone.

## Pre-flight verified (orchestrator-grep'd 2026-05-22)

### Current HolonAST enum

`holon-rs/src/kernel/holon_ast.rs:51-132`:

12 variants: Symbol, String, I64, F64, Bool (leaves) + Atom, Bind, Bundle, Permute, Thermometer, Blend, SlotMarker.

Doc comment at lines 47-49: *"The universal AST. Twelve variants. Closed under itself."* — needs update to "Thirteen" after this stone (sonnet may update the comment in same stone, or leave for Stone 221.6 INSCRIPTION; recommendation: update inline since it's a 1-character change).

### Hash impl

`holon-rs/src/kernel/holon_ast.rs:208-244`:

```rust
impl Hash for HolonAST {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);  // discriminant tag first
        match self {
            HolonAST::Symbol(s) => s.hash(state),
            HolonAST::String(s) => s.hash(state),
            HolonAST::I64(n) => n.hash(state),
            HolonAST::F64(x) => x.to_bits().hash(state),  // NaN-safe
            HolonAST::Bool(b) => b.hash(state),
            // ... other variants
        }
    }
}
```

Pattern: discriminant + per-variant payload hash.

### PartialEq impl

`holon-rs/src/kernel/holon_ast.rs:170-205`: matches `(HolonAST::Variant(a), HolonAST::Variant(b)) => a == b` per pair. Self-only match (no cross-type Eq).

### Debug impl

`holon-rs/src/kernel/holon_ast.rs:134-168`: `f.debug_tuple("Variant").field(&payload).finish()` per arm.

### canonical_edn_holon (VSA seed bytes)

`holon-rs/src/kernel/holon_ast.rs:520-585` (full function shown earlier; you have the source pattern):

```rust
HolonAST::Symbol(s) => write_atom_payload(&mut out, PRIM_TAG_STRING, s.as_bytes()),
HolonAST::String(s) => write_atom_payload(&mut out, PRIM_TAG_STRING, s.as_bytes()),
HolonAST::I64(n) => write_atom_payload(&mut out, PRIM_TAG_I64, &n.to_le_bytes()),
HolonAST::F64(x) => write_atom_payload(&mut out, PRIM_TAG_F64, &x.to_le_bytes()),
HolonAST::Bool(b) => write_atom_payload(&mut out, PRIM_TAG_BOOL, &[*b as u8]),
```

Pattern: `write_atom_payload(&mut out, PRIM_TAG_<TYPE>, &<payload-bytes>)`.

### PRIM_TAG constants

`holon-rs/src/kernel/holon_ast.rs:494-505`:

```rust
const PRIM_TAG_STRING: &str = "String";
const PRIM_TAG_I64: &str = "i64";
const PRIM_TAG_F64: &str = "f64";
const PRIM_TAG_BOOL: &str = "bool";
const ATOM_INNER_TAG: &str = "wat/algebra/Holon";
```

Pattern: short snake-case string for primitive type tag.

### Existing constructors

`holon-rs/src/kernel/holon_ast.rs:246+` (`impl HolonAST` block):

```rust
pub fn symbol(content: impl Into<Arc<str>>) -> Self { HolonAST::Symbol(content.into()) }
pub fn string(content: impl Into<Arc<str>>) -> Self { HolonAST::String(content.into()) }
// ... etc
```

Pattern: lowercase name matching variant (rust keyword collision is the problem for Char/Bool — use trailing underscore like `bool_` if a `bool` constructor exists).

## Your scope (sonnet)

Execute the following edits in `holon-rs/src/kernel/holon_ast.rs`:

### 1. Add `Char(char)` variant to the enum

Place alongside the other primitive leaves (Symbol/String/I64/F64/Bool — pick the alphabetical or thematic slot you prefer, recommendation: right after `Bool` to keep all leaves grouped). Include a doc comment:

```rust
/// Char leaf — single Unicode scalar value. EDN-literal form `\a`,
/// `\newline`, `\u{NNNN}` per Clojure-EDN spec. BMP-only is a wat-rs
/// surface concern (arc 220 Stone 220.2); holon-rs accepts full `char`.
///
/// Distinct from `String(":outcome")` and `Symbol(":outcome")` at both
/// the type level AND the canonical-bytes level — `PRIM_TAG_CHAR` is
/// a distinct seed; the vector identity differs from String/Symbol.
Char(char),
```

Update the enum-level doc comment count: *"Twelve variants. Closed under itself."* → *"Thirteen variants. Closed under itself."* (this stone's count; subsequent stones in arc 221 bump further).

### 2. Debug arm

```rust
HolonAST::Char(c) => f.debug_tuple("Char").field(c).finish(),
```

### 3. PartialEq arm

```rust
(HolonAST::Char(a), HolonAST::Char(b)) => a == b,
```

### 4. Hash arm

```rust
HolonAST::Char(c) => (*c as u32).hash(state),
```

(The outer `std::mem::discriminant(self).hash(state)` already runs at the top of the match — this arm just adds the payload-hash. `char as u32` gives a deterministic u32 representation.)

### 5. canonical_edn_holon arm

```rust
HolonAST::Char(c) => write_atom_payload(&mut out, PRIM_TAG_CHAR, &(*c as u32).to_le_bytes()),
```

(4-byte LE u32 payload. Fixed-width + deterministic. Distinct from `String("a")` byte-for-byte because PRIM_TAG_CHAR differs from PRIM_TAG_STRING.)

### 6. PRIM_TAG_CHAR constant

Add alongside the others at `holon-rs/src/kernel/holon_ast.rs:494-505`:

```rust
const PRIM_TAG_CHAR: &str = "char";
```

### 7. Constructor

Add to `impl HolonAST` block (`holon-rs/src/kernel/holon_ast.rs:246+`):

```rust
/// Construct a `Char` leaf from a Rust `char`. The substrate accepts
/// full Unicode; BMP-only enforcement is a wat-rs surface concern.
pub fn char_(c: char) -> Self {
    HolonAST::Char(c)
}
```

(Trailing underscore because `char` is a Rust keyword.)

### 8. Tests

In the existing `#[cfg(test)] mod tests` block at the bottom of `holon-rs/src/kernel/holon_ast.rs` (find via grep `mod tests` if not obvious; mirror existing test shape):

```rust
#[test]
fn char_leaf_round_trip() {
    let h = HolonAST::char_('a');
    assert_eq!(h, HolonAST::Char('a'));
    assert_ne!(h, HolonAST::char_('b'));
    // Hash determinism: same char → same hash
    let mut h1 = std::collections::hash_map::DefaultHasher::new();
    h.hash(&mut h1);
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    HolonAST::char_('a').hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[test]
fn char_distinct_from_string() {
    // Char('a') and String("a") MUST produce distinct canonical bytes
    // (and therefore distinct VSA vectors). PRIM_TAG_CHAR ≠ PRIM_TAG_STRING.
    let char_bytes = canonical_edn_holon(&HolonAST::char_('a'));
    let str_bytes = canonical_edn_holon(&HolonAST::string("a"));
    assert_ne!(char_bytes, str_bytes,
        "Char('a') and String(\"a\") MUST differ in canonical bytes");
}

#[test]
fn char_distinct_from_symbol() {
    // Char('a') and Symbol("a") MUST produce distinct canonical bytes.
    let char_bytes = canonical_edn_holon(&HolonAST::char_('a'));
    let sym_bytes = canonical_edn_holon(&HolonAST::symbol("a"));
    assert_ne!(char_bytes, sym_bytes,
        "Char('a') and Symbol(\"a\") MUST differ in canonical bytes");
}
```

(Import `std::hash::{Hash, Hasher}` if not already imported in the test module.)

### Verification (must run before SCORE)

From `/home/watmin/work/holon/holon-rs/`:

1. `cargo build --release` — workspace clean (0 warnings; if pre-existing warnings exist, surface count but don't gate)
2. `cargo test --release` — all tests PASS including new 3
3. `cargo clippy --release -- -D warnings` — 0 warnings on the canonical clippy gate (if pre-existing warnings in holon-rs exist, surface count but don't gate this stone)

**Wat-rs build NOT required this stone.** No wat-rs changes happen here.

**Write `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.1.md`** mirroring SCORE-STONE-220.4 shape (smaller — 8 rows per EXPECTATIONS scorecard).

## STOP triggers

- **STOP-1 (existing holon-rs test regression):** if `cargo test --release` fail count goes UP from baseline → diagnostic + report. Adding a variant should be additive; anything that breaks suggests a downstream consumer was assuming the enum was closed at 12.
- **STOP-2 (canonical_bytes test confirms identity collapse):** if `char_distinct_from_string` or `char_distinct_from_symbol` FAILS, the PRIM_TAG_CHAR constant isn't being used or write_atom_payload isn't differentiating — diagnostic + report.
- **STOP-3 (90 min elapsed):** wall-clock STOP.
- **STOP-4 (wat-rs touched accidentally):** if `git -C /home/watmin/work/holon/wat-rs/ diff` shows any changes from this stone, STOP and report — wat-rs is OUT OF SCOPE this stone.
- **EXTRA — interop handshakes NOT required this stone** — wat-edn surface untouched; holon-rs-only change.

## Out-of-scope

- wat-rs changes (Stone 221.2 — separate stone)
- Other HolonAST variants (Stone 221.3 — Keyword + Nil + Tag together)
- Symbol/String canonical-bytes seed distinction (Stone 221.5)
- Migration ripple in holon-rs consumers (Stone 221.3 sweep)
- INSCRIPTION (Stone 221.6)
- Documentation beyond test comments + SCORE

# BRIEF — Arc 221 Stone 221.5 — Symbol/String canonical-bytes seed distinction in holon-rs

**Stone scope (sonnet portion):** Mint `PRIM_TAG_SYMBOL = "symbol"` constant in `holon-rs/src/kernel/holon_ast.rs`; flip Symbol's arm in `canonical_edn_holon()` + `encode()` from `PRIM_TAG_STRING` to `PRIM_TAG_SYMBOL`. Updates the Symbol doc comment (lines 67-71) to retire the "Stone 221.5 resolves it" deferral. 2 new distinctness tests. **Holon-rs ONLY — wat-rs untouched.** Last substrate stone in arc 221's Phase B; arc 221's INSCRIPTION (Stone 221.6) blocked on arc 222 + 223 per spawn-block.
**Type:** Sonnet Mode A.
**Time budget:** 30-45 min target; 60 min STOP.
**Depends on:** Stone 221.3 (holon-rs `fa48b39`) + Stone 221.4b (wat-rs `9450bd3`) — Symbol arm location already established in canonical_edn_holon + encode.
**Calibration:** Per `feedback_stone_briefs_cite_prior_score`, read **SCORE-STONE-221.1.md** (the closest precedent — holon-rs single-variant addition, ~25 min under band). Stone 221.5 is SIMPLER (no variant addition, just a constant rename in 2 sites + 2 tests + 1 doc refresh).

## Working dir + constraints

- **Working dir: `/home/watmin/work/holon/holon-rs/`** (NOT wat-rs!)
- Branch: holon-rs `main` (already current; same as Stones 221.1 + 221.3)
- Linux only; no `--no-verify`.
- DO NOT commit. Orchestrator commits after independent scoring.
- DO NOT touch wat-rs files this stone.

## Pre-flight verified (orchestrator-grep'd 2026-05-22 very-late)

### Current state of Symbol's canonical-bytes seed

**`holon-rs/src/kernel/holon_ast.rs` — `canonical_edn_holon()` Symbol arm (line ~549):**
```rust
HolonAST::Symbol(s) => write_atom_payload(&mut out, PRIM_TAG_STRING, s.as_bytes()),
HolonAST::String(s) => write_atom_payload(&mut out, PRIM_TAG_STRING, s.as_bytes()),
```
Both share `PRIM_TAG_STRING`. After this stone:
```rust
HolonAST::Symbol(s) => write_atom_payload(&mut out, PRIM_TAG_SYMBOL, s.as_bytes()),
HolonAST::String(s) => write_atom_payload(&mut out, PRIM_TAG_STRING, s.as_bytes()),
```

**`encode()` Symbol arm (line ~626 area):**
```rust
HolonAST::Symbol(s) => {
    let seed = leaf_seed(PRIM_TAG_STRING, s.as_bytes(), vm.global_seed());
    // ... rest
}
HolonAST::String(s) => {
    let seed = leaf_seed(PRIM_TAG_STRING, s.as_bytes(), vm.global_seed());
    // ... rest
}
```
Same flip — Symbol → `PRIM_TAG_SYMBOL`.

### PRIM_TAG block (line ~522 area)

```rust
const PRIM_TAG_STRING: &str = "String";
const PRIM_TAG_I64: &str = "i64";
const PRIM_TAG_F64: &str = "f64";
const PRIM_TAG_BOOL: &str = "bool";
const PRIM_TAG_CHAR: &str = "char";
const PRIM_TAG_KEYWORD: &str = "keyword";
const PRIM_TAG_NIL: &str = "nil";
const PRIM_TAG_TAG: &str = "tag";
const ATOM_INNER_TAG: &str = "wat/algebra/Holon";
```

Add `PRIM_TAG_SYMBOL: &str = "symbol"` — snake-case mirrors existing leaf tags.

### Symbol doc comment (lines 53-71)

Post-Stone-221.3 it currently says:
> *"Currently shares the `PRIM_TAG_STRING` canonical-bytes seed with `String` — this is a pre-arc-216 accepted-collision documented at the PRIM_TAG block; Stone 221.5 resolves it..."*

After this stone: rewrite to reflect resolution — drop the "Stone 221.5 resolves it" deferral; state plainly that Symbol and String have distinct PRIM_TAG seeds (`"symbol"` vs `"String"`); distinct at type level AND canonical-bytes level.

### Regression risk audit (DESIGN-221 open question #4)

Orchestrator pre-flight grep: ZERO active assertions in holon-rs source/tests claim `Symbol("x") == String("x")` at canonical-bytes or vector level. The only match for "Symbol.*String" or vice versa is a HISTORICAL COMMENT in `keyword_distinct_from_symbol_at_type_level` (Stone 221.3 rewrite) explaining what was changed. **Low risk of cascade — but verify after edit.**

## Your scope (sonnet)

### 1. Add PRIM_TAG_SYMBOL constant

In the PRIM_TAG block (around line 522), add:
```rust
const PRIM_TAG_SYMBOL: &str = "symbol";
```

### 2. Flip Symbol arm in canonical_edn_holon (line ~549)

```rust
HolonAST::Symbol(s) => write_atom_payload(&mut out, PRIM_TAG_SYMBOL, s.as_bytes()),
```

(String arm unchanged.)

### 3. Flip Symbol arm in encode (line ~626 area)

```rust
HolonAST::Symbol(s) => {
    let seed = leaf_seed(PRIM_TAG_SYMBOL, s.as_bytes(), vm.global_seed());
    // ... rest of leaf_seed → Vector pattern unchanged
}
```

### 4. Rewrite Symbol doc comment (lines 53-71)

Replace the post-Stone-221.3 text that says "Stone 221.5 resolves it" with affirmative resolution:
- Symbol is bare-identifier-only (resolved by Stone 221.3)
- Distinct from String at type level (always was)
- **NEW:** distinct from String at canonical-bytes level via `PRIM_TAG_SYMBOL` (resolved by THIS stone)
- The pre-arc-216 collision is closed; arc 221 doctrine complete

### 5. Add 2 new tests

In the existing `#[cfg(test)] mod tests` block:

```rust
#[test]
fn symbol_string_canonical_bytes_distinct() {
    // Stone 221.5: PRIM_TAG_SYMBOL distinct from PRIM_TAG_STRING.
    // Symbol("x") and String("x") MUST produce distinct canonical bytes.
    let sym_bytes = canonical_edn_holon(&HolonAST::symbol("x"));
    let str_bytes = canonical_edn_holon(&HolonAST::string("x"));
    assert_ne!(sym_bytes, str_bytes,
        "Symbol(\"x\") and String(\"x\") MUST differ in canonical bytes (Stone 221.5)");
}

#[test]
fn symbol_string_vectors_distinct() {
    // Stone 221.5: Symbol and String produce distinct VSA vectors at matched content.
    let (vm, se) = fresh_env();
    let v_sym = encode(&HolonAST::symbol("x"), &vm, &se);
    let v_str = encode(&HolonAST::string("x"), &vm, &se);
    assert_ne!(v_sym, v_str,
        "Symbol(\"x\") and String(\"x\") MUST produce distinct vectors (Stone 221.5)");
}
```

(If `fresh_env()` is the established test helper — confirm from existing tests like `keyword_vs_string_distinct_by_content` at line ~840 area. Use same helper.)

### 6. Verification (must run before SCORE)

From `/home/watmin/work/holon/holon-rs/`:

```
cargo build --release
cargo test --release
cargo clippy --release -- -D warnings
```

All clean. Pre-existing baseline: 287/287 PASS (Stone 221.3 post-flight). Expect 289/289 post-Stone-221.5 (+2 new tests).

**Wat-rs build NOT required this stone** — no wat-rs changes.

**Write `wat-rs/docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.5.md`** mirroring SCORE-STONE-221.1.md shape (~7 rows per scorecard).

## STOP triggers

- **STOP-1 (existing holon-rs test regression):** if `cargo test --release` fail count goes UP from baseline 287 (besides the +2 new), STOP + diagnostic + report. The change is mechanical (constant rename in 2 sites); regression suggests an undiscovered consumer assumed Symbol/String produced equal vectors. Per Stone 221.3 Delta 1a discipline: tests broken BY this stone are NOT pre-existing — frame honestly.
- **STOP-2 (distinctness tests fail):** if `symbol_string_canonical_bytes_distinct` or `symbol_string_vectors_distinct` FAILS, the PRIM_TAG_SYMBOL constant isn't being used in the arm — diagnostic + report.
- **STOP-3 (60 min elapsed):** wall-clock STOP.
- **STOP-4 (wat-rs touched accidentally):** STOP and report.

## Out-of-scope

- wat-rs changes (no Symbol/String collision concern there — wat's runtime layer uses native Rust containers)
- Stone 221.6 INSCRIPTION (blocked on arc 222 + 223 per spawn-block)
- Arc 222 + arc 223 work
- New HolonAST variants (settled at 16 per arc 221 doctrine)
- Holon-rs migration of any code that produced same-content Symbol+String (if any pre-existing — likely none per pre-flight)
- BOOK / USER-GUIDE updates (Stone 221.6)

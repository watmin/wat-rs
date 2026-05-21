# BRIEF — Arc 218 Stone 218.4 — UUID strictness + USER-GUIDE doc fixes

**Stone scope (sonnet portion):** four items — two substrate (UUID strictness symmetric across EDN + JSON paths) + two doc fixes (USER-GUIDE map separator claim + parse_wire/parse_wire_owned documentation).
**Type:** Sonnet Mode A.
**Time budget:** 20-40 min target; 55 min STOP.
**Depends on:** Stone 218.3 (`aff23db` — contract precision shipped 11/11).
**Calibration:** 218.1 ~20 (band 25-45); 218.2 ~15 (band 30-50); 218.3 ~25 (band 40-65). All below lower bound. Pattern locked: substrate-pre-grep + mechanical + locked-decisions = fast.
**Unblocks:** Stone 218.5 (closure paperwork — INSCRIPTION + re-cast vigilia).

## Pre-flight verified (orchestrator-grep'd 2026-05-21, post-218.3)

### Item A — `is_canonical_uuid` upper-hex acceptance

`crates/wat-edn/src/parser.rs:455-471` — currently:
```rust
fn is_canonical_uuid(s: &str) -> bool {
    if s.len() != 36 { return false; }
    let bytes = s.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        let expect_dash = matches!(i, 8 | 13 | 18 | 23);
        let is_dash = b == b'-';
        if expect_dash != is_dash { return false; }
        if !is_dash && !b.is_ascii_hexdigit() { return false; }  // ← accepts upper + lower
    }
    true
}
```

The docstring (parser.rs:450-454) claims:
> "The canonical form is 8-4-4-4-12 lowercase hexadecimal characters separated by hyphens."

But `b.is_ascii_hexdigit()` accepts both upper AND lower hex. Doc lies; substrate is lenient. Fix: tighten to lowercase-only.

### Item B — `decode_uuid` (JSON bridge) skips canonical check

`crates/wat-edn/src/json.rs:390-396` — currently:
```rust
fn decode_uuid(v: &JV) -> JsonResult<OwnedValue> {
    let s = v.as_str().ok_or_else(|| JsonError::InvalidUuid(v.to_string()))?;
    let u = uuid::Uuid::parse_str(s).map_err(|e| JsonError::InvalidUuid(format!("{}: {}", s, e)))?;
    Ok(Value::Uuid(u))
}
```

`uuid::Uuid::parse_str` is lenient (accepts simple-form, URN-form, braced-form, upper-hex, etc.). The EDN path at `parser.rs:297` calls `is_canonical_uuid` BEFORE `parse_str`; the JSON path skips that check entirely. Asymmetric strictness.

### Item C — USER-GUIDE map separator claim wrong

**Writer truth:** `crates/wat-edn/src/writer.rs:333-345` `write_map` emits **space-only** between entries (`out.push(' ')` at line 338). The byte-level output is `{:a 1 :b 2}`, NOT `{:a 1, :b 2}`. Confirmed by writer test at `writer.rs:399`: `assert_eq!(write(&m), "{:a 1 :b 2}");`.

**USER-GUIDE lies:**
- `crates/wat-edn/docs/USER-GUIDE.md:231-232` — "Maps emit `key value` pairs separated by `, ` (commas are whitespace per spec; the comma is purely visual)."
- `crates/wat-edn/docs/USER-GUIDE.md:294` — `assert_eq!(write(&order), r#"#myapp/Order {:id 1, :name "Alice"}"#);` — this assertion would FAIL if it were a real test.

### Item D — USER-GUIDE missing parse_wire/parse_wire_owned docs

**Reality:** `crates/wat-edn/src/lib.rs:144` defines `pub fn parse_wire(input: &str) -> Result<Value<'_>>`; `:150` defines `pub fn parse_wire_owned(input: &str) -> Result<OwnedValue>`. Real public functions.

**USER-GUIDE:** No mention. The current parse documentation around lines 145-161 covers `parse`, `parse_owned`, `parse_all`, `Parser::new`, `Parser::new_wire`, `parse_top`, `parse_all` — but the free-function wire-mode parsers (`parse_wire`, `parse_wire_owned`) are absent. Downstream consumers reading USER-GUIDE wouldn't know these exist.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

### Part A — `is_canonical_uuid` lowercase enforcement

1. **At `crates/wat-edn/src/parser.rs:466`**, replace:
   ```rust
   if !is_dash && !b.is_ascii_hexdigit() {
   ```
   with a check that accepts only lowercase hex:
   ```rust
   let is_lowercase_hex = b.is_ascii_digit() || (b'a'..=b'f').contains(&b);
   if !is_dash && !is_lowercase_hex {
   ```
   (Or sonnet's cleaner spelling — semantically: digits OR lowercase a-f.)

### Part B — `decode_uuid` (JSON bridge) canonical-strict

2. **At `crates/wat-edn/src/json.rs:390-396`**, add canonical check before `parse_str`:
   ```rust
   fn decode_uuid(v: &JV) -> JsonResult<OwnedValue> {
       let s = v.as_str().ok_or_else(|| JsonError::InvalidUuid(v.to_string()))?;
       if !is_canonical_uuid(s) {
           return Err(JsonError::InvalidUuid(format!("{}: not canonical 8-4-4-4-12 lowercase form", s)));
       }
       let u = uuid::Uuid::parse_str(s).map_err(|e| JsonError::InvalidUuid(format!("{}: {}", s, e)))?;
       Ok(Value::Uuid(u))
   }
   ```
   
   **`is_canonical_uuid` exposure:** currently `fn` (private) at `parser.rs:455`. To share with json.rs, sonnet picks the cleanest option:
   - (A) Make it `pub(crate)` in parser.rs and `use crate::parser::is_canonical_uuid` in json.rs
   - (B) Move it to `vocab.rs` (the shared vocabulary module) and `pub(crate)` there
   
   Lean toward (A) — minimal disturbance; one-word visibility change. (B) is cleaner long-term but invites larger rename surface than this stone needs. Sonnet picks; documents.

### Part C — USER-GUIDE map separator claim fix

3. **At `crates/wat-edn/docs/USER-GUIDE.md:231-232`**, change:
   ```
   Maps emit `key value` pairs separated by `, ` (commas are
   whitespace per spec; the comma is purely visual). Tagged values
   ```
   to:
   ```
   Maps emit `key value` pairs separated by a single space. EDN treats
   commas as whitespace, so a reader will accept `{:a 1, :b 2}` and
   `{:a 1 :b 2}` identically — the writer chooses the compact form.
   Tagged values
   ```
   (Or sonnet's wording — preserve the "commas are whitespace" teaching context but fix the writer-output claim.)

4. **At `crates/wat-edn/docs/USER-GUIDE.md:294`**, fix the assertion example:
   ```
   assert_eq!(write(&order), r#"#myapp/Order {:id 1, :name "Alice"}"#);
   ```
   to:
   ```
   assert_eq!(write(&order), r#"#myapp/Order {:id 1 :name "Alice"}"#);
   ```
   (Remove the comma; match what `write_map` actually emits per `writer.rs:338`.)

5. **Sweep for any other map-format claims in USER-GUIDE.md** that mention comma-separators in writer output. The vigilia note cited lines 233 + 294 specifically; there may be sibling claims. One grep:
   ```
   grep -n ", :" crates/wat-edn/docs/USER-GUIDE.md
   ```
   For each match: is it a CLAIM about writer output (fix) or an EXAMPLE of valid INPUT (keep — readers accept comma-separated)?

### Part D — USER-GUIDE add parse_wire / parse_wire_owned documentation

6. **At `crates/wat-edn/docs/USER-GUIDE.md` around line 145-161** (the parse section), add documentation for `parse_wire` + `parse_wire_owned`. They're free-function wire-mode parsers — equivalent to `Parser::new_wire(input).parse_top()` and `Parser::new_wire(input).parse_top().map(|v| v.into_owned())` respectively.

   Suggested addition (sonnet picks the cleanest placement + wording):
   ```rust
   use wat_edn::{parse, parse_owned, parse_all, parse_wire, parse_wire_owned, Parser};

   // Source-mode (default) parsers: comma is whitespace per EDN spec.
   let v: Value<'_> = parse(input)?;
   let vs: Vec<Value<'_>> = parse_all("1 2 3")?;

   // Wire-mode parsers: applies the `,` → `_` swap inside parametric type
   // arglists (`:Foo<A,B>` survives the wire form). Used when the input
   // came from wat-rs's wire-encoding layer.
   let v: Value<'_> = parse_wire(wire_input)?;
   let v_owned: OwnedValue = parse_wire_owned(wire_input)?;
   ```
   
   Cross-reference wire-mode behavior to the existing wire-encoding section if one exists.

### Part E — Probes

7. **Add probes for the strictness fixes.** Two probes (one each for EDN + JSON paths):

   - **Probe 1: `is_canonical_uuid_rejects_uppercase`** — preferred placement `crates/wat-edn/tests/spec_strict.rs` (sits next to `accepts_canonical_uuid` at line 222; same family). Asserts that `is_canonical_uuid("550E8400-E29B-41D4-A716-446655440000")` returns false (post-fix). Since `is_canonical_uuid` is `fn` (or `pub(crate)`), the test needs access — adding the test alongside in `spec_strict.rs` works if the function is `pub(crate)` and the test file uses `use wat_edn::parser::is_canonical_uuid;` (or wherever it lives). If exposure is tighter than `pub(crate)`, sonnet probes the EDN parse path instead: assert that `parse("#uuid \"550E8400-E29B-41D4-A716-446655440000\"")` returns `Err` (the parser rejects pre-canonical-strict). Either form is acceptable; sonnet picks based on visibility.
   
   - **Probe 2: `decode_uuid_rejects_uppercase_via_json_bridge`** — `crates/wat-edn/src/json.rs` internal `#[cfg(test)]` tests (sibling of the parse_map_key probe from 218.3). Asserts that `json_to_edn` on a sentinel with `#uuid` tag containing uppercase hex returns `Err(JsonError::InvalidUuid(...))`. Either uses `decode_uuid` directly or constructs a JSON value that routes through it.

### Part F — Verification

8. **Run the wat-edn test suite — verify zero regressions + probes pass:**
   ```
   cargo build --release -p wat-edn
   cargo test --release -p wat-edn
   cargo clippy --release -p wat-edn -- -D warnings
   ```
   Baseline (post-218.3): 337/337 PASS. Expected after this stone: **339/339 PASS** (+2 new probes; additive). If any existing test regresses, surface — strictness changes shouldn't break existing tests (existing UUID test fixtures use lowercase canonical form).

### Part G — SCORE

9. **SCORE doc** at `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.4.md` — scorecard matching EXPECTATIONS row count; deltas; verification summary; elapsed time. Calibration shape per `SCORE-STONE-218.3.md`.

## NOT your scope

- Stone 218.5 public-API runes + INSCRIPTION + re-cast vigilia — closure paperwork
- Adding new variants beyond what UUID strictness requires (e.g., no new `JsonError` variants needed; reuse `InvalidUuid`)
- Touching tagged-literal naming — arc 216.8/.9 territory; out of arc 218 entirely
- DESIGN.md / INTERSTITIAL amendments — orchestrator-direct
- Performance work — surfaced items only

## STOP triggers

- **STOP-1: existing UUID test regresses on strictness** — if a real test was depending on upper-hex acceptance, surface (would be a Surface design question)
- **STOP-2: `is_canonical_uuid` visibility change ripples beyond parser.rs + json.rs** — surface; might need vocab.rs move
- **STOP-3: USER-GUIDE has more comma-separator claims than vigilia cited** — surface count; sweep all or pause
- **STOP-4: wat-edn test regresses (beyond the expected +2 probes)** — surface
- **STOP-5: clippy new warnings** — surface
- **STOP-6: 55 min elapsed**

## Verification (one per line)

```
cargo build --release -p wat-edn
cargo test --release -p wat-edn
cargo clippy --release -p wat-edn -- -D warnings
```

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas (`is_canonical_uuid` visibility choice; any extra USER-GUIDE comma claims found beyond vigilia's two; probe placement decisions), verification summary, elapsed time. Cite the new test count (337 → 339 expected) as additive.

Don't commit. Orchestrator commits after review.

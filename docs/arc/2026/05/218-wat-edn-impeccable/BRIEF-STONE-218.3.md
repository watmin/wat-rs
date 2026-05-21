# BRIEF — Arc 218 Stone 218.3 — Contract precision

**Stone scope (sonnet portion):** six contract-precision items from struere (5) + temperare (1). Each independent; bundles cleanly. Foundation-clean substrate post-218.1 + 218.2; this stone tightens the contract surface.
**Type:** Sonnet Mode A.
**Time budget:** 40-65 min target; 90 min STOP.
**Depends on:** Stone 218.2 (`525b8a3` — naming sweep; vocab.rs renamed), Stone 218.1 (`57af300` — write_keyword_body extracted).
**Calibration:** 218.1 predicted 25-45 actual ~20; 218.2 predicted 30-50 actual ~15. Pattern: substrate-pre-grep + bundled mechanical edits ship fast. This stone has slightly more design surface (pretty-print map decision; closer-token diagnostic shape) so prediction band widens.
**Unblocks:** Stones 218.4 / 218.5.

## Pre-flight verified (orchestrator-grep'd 2026-05-21, post-218.2)

- **`crates/wat-edn/src/writer.rs:106-125`** — `write_pretty_indented` Map arm. Asymmetry confirmed: `if i > 0 { push_indent(...) }` at line 113 skips first key; closing `}` at line 123 fused to value (no newline+indent before close). Vector arm at lines 66-104 IS symmetric (open + `\n`, indent before every item, `\n` + indent before close). The asymmetry is real.
- **`crates/wat-edn/src/json.rs:162-163`** — `to_json_string` with `.expect("serde_json::to_string on Value")`. **HONEST DELTA from vigilia:** vigilia's L2 #12 cited `writer.rs:162-170 (json wrapper)` — the function is actually in `json.rs`, not `writer.rs`. Sonnet notes this delta. Fix target same; file path corrected.
- **`crates/wat-edn/src/json.rs:279-294`** — `parse_map_key`. Confirmed silent-fallback at line 289: `if let Ok(v) = crate::parse(k) { return Ok(v.into_owned()) }` — when EDN parse FAILS, falls through to `Ok(Value::String(k.to_string().into()))`. Silent swallow.
- **`crates/wat-edn/src/parser.rs:158-160`** — closer-token diagnostic arm:
  ```rust
  Token::RParen | Token::RBracket | Token::RBrace | Token::Eof => {
      Err(Error::at(pos, ErrorKind::UnexpectedEof))
  }
  ```
  All four reported as `UnexpectedEof`. Vigilia cited line 158; actual is line 159 (off-by-one; honest delta).
- **`crates/wat-edn/src/lexer.rs:213`** — `let mut out = String::with_capacity(self.input.len() - body_start);` confirmed. `self.input.len()` is whole-input length; `self.pos - body_start` would be the already-consumed body length (tighter upper bound).
- **`crates/wat-edn/src/parser.rs:382-391`** — identifier suffix scan. Confirmed:
  ```rust
  if let Some(idx) = body.find('/') {     // ← scan 1
      let ns = &body[..idx];
      let name = &body[idx + 1..];
      if ns.is_empty() { ... }
      if name.is_empty() { ... }
      if name.contains('/') { ... }       // ← scan 2 (same data, second pass)
      ...
  }
  ```
  Two linear scans; fold via `splitn(3, '/')` collapses to one.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

### Part A — Pretty-print map symmetry (struere L2 #11)

1. **Make map pretty-print symmetric with vector at `writer.rs:106-125`.** Match the vector pattern at lines 66-104:
   ```rust
   Value::Map(entries) => {
       if entries.is_empty() {
           out.push_str("{}");
       } else {
           out.push('{');
           out.push('\n');
           let inner = level + 1;
           for (i, (k, val)) in entries.iter().enumerate() {
               push_indent(out, inner);
               write_pretty_indented(k, out, inner);
               out.push(' ');
               write_pretty_indented(val, out, inner);
               if i + 1 < entries.len() {
                   out.push('\n');
               }
           }
           out.push('\n');
           push_indent(out, level);
           out.push('}');
       }
   }
   ```
   (Or sonnet's cleaner spelling that produces equivalent symmetric output.)

2. **Update test expectations** for any pretty-print map snapshots in `crates/wat-edn/tests/pretty.rs` + `comprehensive.rs` + any others that snapshot map pretty-print output. The format change is intentional — tests that asserted the OLD asymmetric format now assert the NEW symmetric format. Each test update must preserve the test's INTENT (what it was checking); only the expected-output string changes.

**Four-questions on the design choice (orchestrator-locked, do not re-litigate):**
- Symmetrize: Obvious YES (matches vector pattern; single mental model), Simple YES, Honest YES (consistent with rest of pretty-print), Good UX YES (no exception to remember)
- Document asymmetry: Obvious NO (readers see vector vs map and ask why), Honest marginal, Good UX NO (two models)
- **Symmetrize wins YES×4.** Ship the symmetric form.

### Part B — to_json_string .expect() rune annotation (struere L2 #12)

3. **At `crates/wat-edn/src/json.rs:162-163`** (NOT writer.rs as vigilia cited; honest delta), add a rune annotation justifying the `.expect()`. The `.expect()` panic is structurally unreachable because `edn_to_json` (called immediately before serde_json) only emits well-formed `serde_json::Value` graphs — primitive types + Maps + Arrays, no NaN-in-Number paths. The invariant is COUPLED to edn_to_json's closed construction.

   Suggested annotation:
   ```rust
   pub fn to_json_string(v: &Value<'_>) -> String {
       // rune:struere(invariant-coupling) — serde_json::to_string cannot
       // fail here because edn_to_json's closed construction emits only
       // well-formed serde_json::Value graphs (no NaN-in-Number). The
       // .expect() panic is structurally unreachable; the coupling is
       // the invariant.
       serde_json::to_string(&edn_to_json(v)).expect("serde_json::to_string on Value")
   }
   ```
   
4. **Same rune on `to_json_string_pretty` at `json.rs:167-170`** — same invariant; matching annotation. (Two call sites; both get runes per `feedback_verbose_is_honest` precedent from 218.1.)

### Part C — parse_map_key strict mode (struere L2 #13)

5. **At `crates/wat-edn/src/json.rs:279-294`**, fix the silent-fallback. Currently when a key LOOKS like EDN (starts with `:`/`[`/`{`/`(`/`#`/`"`) but FAILS to parse, the code silently falls through to treating it as a plain String. This loses information.

   **Four-questions on strict vs documented-silent (orchestrator-locked):**
   - Strict (return Err on EDN-looking parse failure): Obvious YES (diagnostic surfaces); Honest YES (no silent swallow); Good UX YES (user sees real error not mystery key-as-string)
   - Documented-silent: Honest NO (silent fallback IS the issue); Good UX NO (mystery keys)
   - **Strict wins.**

   Implementation: at `json.rs:289`:
   ```rust
   if looks_like_edn {
       match crate::parse(k) {
           Ok(v) => return Ok(v.into_owned()),
           Err(e) => return Err(JsonError::InvalidMapKey {
               key: k.to_string(),
               source: e.to_string(),
           }),
       }
   }
   Ok(Value::String(k.to_string().into()))
   ```
   
   If `JsonError::InvalidMapKey` doesn't exist as a variant, add it to `JsonError` enum. Look at sibling variants in `json.rs` (e.g., `InvalidBigInt`) for the variant shape pattern.

6. **Add a test in `crates/wat-edn/tests/`** (json-side, file of your choice — `comprehensive.rs` or `spec_conformance.rs` likely host JSON tests) that asserts: an EDN-looking key that fails to parse returns `JsonError::InvalidMapKey`, not a `Value::String`. One probe.

### Part D — Closer-token diagnostic split (struere L2 #14)

7. **At `crates/wat-edn/src/parser.rs:158-160`** (vigilia cited 158; actual is 159), split the arm:
   ```rust
   Token::Eof => Err(Error::at(pos, ErrorKind::UnexpectedEof)),
   Token::RParen | Token::RBracket | Token::RBrace => {
       Err(Error::at(pos, ErrorKind::UnexpectedToken(token_name)))
   }
   ```
   Where `token_name` describes which closer was seen (`")"`, `"]"`, `"}"`). If `ErrorKind::UnexpectedToken` doesn't exist, either:
   - (a) Add a new variant `UnexpectedToken(&'static str)` or `UnexpectedToken(String)` — sonnet picks based on existing enum patterns
   - (b) Reuse `ErrorKind::UnexpectedByte(u8)` with the closer's byte — `b')'`, `b']'`, `b'}'`

   Inspect the `ErrorKind` enum in `crates/wat-edn/src/error.rs` first; pick the cleaner option that fits the existing pattern.

### Part E — lexer.rs:213 allocation cap (struere L2 #15)

8. **At `crates/wat-edn/src/lexer.rs:213`**, change:
   ```rust
   let mut out = String::with_capacity(self.input.len() - body_start);
   ```
   to:
   ```rust
   let mut out = String::with_capacity(self.pos - body_start);
   ```
   The already-consumed body length is the tighter upper bound; `self.input.len()` over-allocates when the string is followed by lots of input.

### Part F — Identifier suffix scan fold (temperare L2 #23)

9. **At `crates/wat-edn/src/parser.rs:382-391`**, fold the two-scan into one via `splitn(3, '/')`:
   ```rust
   let mut parts = body.splitn(3, '/');
   let first = parts.next().unwrap();  // body always has at least one char
   match (parts.next(), parts.next()) {
       (None, _) => {
           // No '/': bare name
           validate_first_char(first).map_err(|m| wrap(format!("{}: {}", body, m)))?;
           Ok((None, first))
       }
       (Some(name), None) => {
           // Exactly one '/': namespaced
           let ns = first;
           if ns.is_empty() {
               return Err(wrap(format!("empty prefix in {}", body)));
           }
           if name.is_empty() {
               return Err(wrap(format!("empty name in {}", body)));
           }
           validate_first_char(ns).map_err(|m| wrap(format!("prefix in {}: {}", body, m)))?;
           validate_first_char(name).map_err(|m| wrap(format!("name in {}: {}", body, m)))?;
           Ok((Some(ns), name))
       }
       (Some(_), Some(_)) => {
           // Two or more '/': illegal
           Err(wrap(format!("more than one / in {}", body)))
       }
   }
   ```
   (Or sonnet's cleaner spelling. Preserve all four error messages exactly: "empty prefix in", "empty name in", "more than one / in", and per-side validate_first_char error wrapping. The shape matters; the error text matters.)

### Part G — Verification

10. **Run the wat-edn test suite — must hold at 336/336 PASS** (with any pretty-print expectations updated to the new symmetric format counted as updates-not-regressions):
    ```
    cargo build --release -p wat-edn
    cargo test --release -p wat-edn
    cargo clippy --release -p wat-edn -- -D warnings
    ```

### Part H — SCORE

11. **SCORE doc** at `docs/arc/2026/05/218-wat-edn-impeccable/SCORE-STONE-218.3.md` — scorecard matching EXPECTATIONS row count; deltas (especially the json.rs vs writer.rs file delta + the parser.rs:158 vs :159 off-by-one + the parse_map_key strict-mode + pretty-print test updates count); verification summary; elapsed time. Calibration shape per `SCORE-STONE-218.2.md`.

## NOT your scope

- Stone 218.4 UUID strictness + USER-GUIDE map format claim + parse_wire docs — separate stone
- Stone 218.5 public-API runes + INSCRIPTION + re-cast vigilia — closure paperwork
- Stones 216.8 / 216.9 tagged-literal changes — they're far-future and use the FQDN doctrine per 2026-05-21b forward-correction
- DESIGN.md / INTERSTITIAL amendments — orchestrator-direct
- New file creation beyond the SCORE doc — no new modules / no new test files unless Part C requires one
- Performance work beyond the allocation cap (Item 8) and scan fold (Item 9) — these surfaced; nothing else

## STOP triggers

- **STOP-1: pretty-print map symmetrization breaks tests beyond simple snapshot updates** — if a test asserts BEHAVIOR (not just format) that breaks, surface before changing
- **STOP-2: `JsonError::InvalidMapKey` requires touching many call sites** — if adding the variant ripples beyond `json.rs`, surface; might need a smaller scope
- **STOP-3: `ErrorKind::UnexpectedToken` requires touching many error display sites** — if the addition ripples, surface; might use existing `UnexpectedByte` instead
- **STOP-4: identifier suffix fold breaks ANY parser test** — the error messages must match exactly; STOP if a test fails with a different message
- **STOP-5: wat-edn test regresses (beyond pretty-print snapshot updates)** — surface
- **STOP-6: clippy new warnings** — surface
- **STOP-7: 90 min elapsed**

## Verification (one per line)

```
cargo build --release -p wat-edn
cargo test --release -p wat-edn
cargo clippy --release -p wat-edn -- -D warnings
```

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas (vigilia citation corrections; pretty-print test update count; JsonError variant decision; UnexpectedToken vs UnexpectedByte decision), verification summary, elapsed time. Cite the pretty-print snapshot updates as a separate count from the test regression count (they're updates, not regressions).

Don't commit. Orchestrator commits after review.

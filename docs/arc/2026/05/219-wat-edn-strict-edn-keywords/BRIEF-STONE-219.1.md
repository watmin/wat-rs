# BRIEF — Arc 219 Stone 219.1 — Substrate strict-EDN + constructor translation + test sweep

**Stone scope (sonnet portion):** the full strict-EDN tightening — vocab.rs drops `:` + `#` from symbol-continue chars; constructors translate `::` → `.` on input; wat-edn-internal test fixtures sweep from `::` to `.` form. Bundled because the surfaces are coordinated; splitting would leave the tree red between stones.
**Type:** Sonnet Mode A.
**Time budget:** 45-75 min target; 95 min STOP.
**Depends on:** Stone 218.4 (`7870483` — UUID strictness shipped; wat-edn baseline 339/339).
**Calibration:** four stones all at-or-below lower band (218.1 ~20, 218.2 ~15, 218.3 ~25, 218.4 ~20). This stone is larger (dual surface: substrate + test sweep) so the band widens.
**Unblocks:** Stones 219.2-219.4; arc 218 Stone 218.5 closure pending arc 219 completion.

## Doctrine (LOCKED in DESIGN-219; do not re-litigate)

- **Option β** — substrate strict; constructor translation; wat-rs internal storage unchanged
- **Decision:** wat-edn becomes strict-EDN on input AND output. Wat-rs callers can keep passing `::`-form strings to constructors; the constructor auto-translates to `.` form for storage. The boundary is the constructor.

## Pre-flight verified (orchestrator-grep'd 2026-05-21, post-218.4)

### Substrate sites

- **`crates/wat-edn/src/vocab.rs:101-122`** `is_symbol_continue` currently accepts `b':'` and `b'#'` (wat-extension chars). Strict EDN does not allow these in symbol bodies.
- **`crates/wat-edn/src/value.rs:233`** `Symbol::ns(namespace, name)` — translation insertion point #1
- **`crates/wat-edn/src/value.rs:257`** `Symbol::try_ns(namespace, name)` — translation insertion point #2
- **`crates/wat-edn/src/value.rs:306`** `Keyword::ns(namespace, name)` — translation insertion point #3
- **`crates/wat-edn/src/value.rs:328`** `Keyword::try_ns(namespace, name)` — translation insertion point #4
- **`crates/wat-edn/src/value.rs:368`** `Tag::ns(namespace, name)` — translation insertion point #5
- **`crates/wat-edn/src/value.rs:385`** `Tag::try_ns(namespace, name)` — translation insertion point #6
- **`from_parts_unchecked`** variants — **do NOT translate**; unchecked caller responsibility

### Test fixture sites (wat-edn-internal)

orchestrator found 5 sites in `crates/wat-edn/tests/wire_encoding.rs` using `::` form:
- `wire_encoding.rs:166` `kw("rust::crossbeam_channel::Sender")`
- `wire_encoding.rs:174` `kw("rust::sync::Mutex<i64>")`
- `wire_encoding.rs:203` `assert_eq!(k.name(), "rust::crossbeam_channel::Sender")`
- `wire_encoding.rs:246` `kw("wat::core::HashMap<wat::core::String,wat::core::i64>")`
- `wire_encoding.rs:337` `"wat::core::HashMap<wat::core::String,wat::core::i64>"`

Sonnet does a broader grep for the full count — `grep -rn '"[a-z_-]*::[a-z]' crates/wat-edn/` covers the wat-edn-internal `::` test literals.

### Wat-rs-internal `::` references

- `src/edn_shim.rs:1045/1049/1518/1522` use `Tag::ns("wat-edn.result", "ok")` — these already use `.` form (`wat-edn.result`). Out of scope for translation.
- `src/edn_shim.rs:1591/1605` use `Tag::ns("wat-edn.holon", "Vector")` + `Tag::ns("wat-edn.opaque", "HandlePool")` — also `.` form.
- Other wat-rs sites: many `:wat::core::Foo` literal references in `.wat` source files + Rust source — UNCHANGED per Option β; wat-rs's internal `::` storage stays.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/`
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`

## Your scope (sonnet)

### Part A — vocab.rs: drop `:` and `#` from is_symbol_continue

1. **At `crates/wat-edn/src/vocab.rs:101-122`** in `is_symbol_continue`, remove `b':'` and `b'#'` from the `matches!` pattern. Final char set for strict EDN symbol bodies:
   ```rust
   b.is_ascii_alphanumeric() || matches!(b, b'.' | b'*' | b'+' | b'!' | b'-' | b'_' | b'?' | b'$' | b'%' | b'&' | b'=' | b'<' | b'>' | b'/')
   ```
   Keep `b'/'` — it's the namespace boundary (parser-level enforces single-`/` per 218.3).
   
   **Note on `b'<' | b'>'`:** these are wat-edn parametric-type-arg chars (`Vec<i64>`). They ARE in strict EDN spec for symbol bodies. Keep.

### Part B — Translation helper

2. **Add a translation helper** in `crates/wat-edn/src/value.rs` (private to the module):
   ```rust
   /// Translate wat-rs `::` namespace separators to strict-EDN `.` form.
   /// One-pass; idempotent (`.` input passes through unchanged).
   fn translate_wat_to_strict(ns: &str) -> String {
       ns.replace("::", ".")
   }
   ```
   (Or sonnet's cleaner spelling. `String::replace` is fine; the translation is small + bounded.)

### Part C — Constructor translation

3. **`Symbol::ns`** at `value.rs:233` — call `translate_wat_to_strict` on `namespace` before further use:
   ```rust
   pub fn ns(namespace: impl AsRef<str>, name: impl AsRef<str>) -> Self {
       let ns_translated = translate_wat_to_strict(namespace.as_ref());
       // ... rest of construction uses ns_translated
   }
   ```
   
4. **`Symbol::try_ns`** at `value.rs:257` — same translation; runs BEFORE `validate_first_char` (so validation runs against the strict form).

5. **`Keyword::ns`** at `value.rs:306` — same pattern as Symbol::ns.

6. **`Keyword::try_ns`** at `value.rs:328` — same pattern as Symbol::try_ns.

7. **`Tag::ns`** at `value.rs:368` — same pattern.

8. **`Tag::try_ns`** at `value.rs:385` — same pattern.

   **STOP if `from_parts_unchecked` exists for these types** — those stay UNCHANGED (unchecked path; caller is responsible for the form). Look for `from_parts_unchecked` in the impl blocks; verify NO translation added there.

### Part D — Test fixture sweep

9. **Sweep wat-edn-internal test fixtures.** First grep:
   ```
   grep -rn '"[a-zA-Z_-]*::[a-zA-Z]' crates/wat-edn/tests/
   ```
   For each match: is it a wat-edn test fixture (string literal passed to a wat-edn constructor)? → flip `::` to `.`. Display assertions checking the resulting strings → also flip expected output.
   
   Known sites (orchestrator found):
   - `wire_encoding.rs:166` `kw("rust::crossbeam_channel::Sender")` → `kw("rust.crossbeam_channel.Sender")`
   - `wire_encoding.rs:174` `kw("rust::sync::Mutex<i64>")` → `kw("rust.sync.Mutex<i64>")`
   - `wire_encoding.rs:203` assertion expecting `"rust::crossbeam_channel::Sender"` → `"rust.crossbeam_channel.Sender"`
   - `wire_encoding.rs:246` + `:337` `"wat::core::HashMap<wat::core::String,wat::core::i64>"` → `"wat.core.HashMap<wat.core.String,wat.core.i64>"`
   
   The wire-mode `,`→`_` swap inside `<...>` is ORTHOGONAL; that behavior continues to work after the `::`→`.` translation (the swap is about commas in arglists, not the namespace separator).
   
   Sonnet's grep may surface more wat-edn-internal sites; flip them all. NON-wat-edn-internal sites (e.g., wat-rs `src/` literals) stay as-is — those are wat-rs's internal `::` usage that the constructor translates.

10. **Verify the constructor translation makes the fixtures still PASS.** A test that calls `kw("rust::crossbeam_channel::Sender")` with the OLD wat-extension expectation that `k.name() == "rust::crossbeam_channel::Sender"` now FAILS after translation. Either:
    - (A) Update the test's expected value to `"rust.crossbeam_channel.Sender"` (translation happened internally; the test reflects post-translation truth)
    - (B) Drop the test if it was specifically testing the `::` extension (unlikely; most are using `::` incidentally)
    
    Sonnet picks per test; documents.

### Part E — Probes

11. **Add 3 new probes** in `crates/wat-edn/tests/spec_strict.rs` (sibling of UUID strictness probes from 218.4):

    - **Probe 1: `is_symbol_continue_rejects_colon`** — unit test of `vocab::is_symbol_continue(b':')` returning false (and `is_symbol_continue(b'#')` returning false). Asserts the lexer-level strictness.
    
    - **Probe 2: `parser_rejects_double_colon_in_keyword`** — `parse(":wat::core::HashMap")` returns `Err` (lexer fails to consume `::` because `:` no longer in continue chars).
    
    - **Probe 3: `keyword_ns_translates_wat_to_strict`** — `Keyword::ns("wat::core", "HashMap")` constructs a Keyword whose `.namespace()` returns `Some("wat.core")` (constructor translation visible).

### Part F — Verification

12. **Run the wat-edn test suite — verify zero regressions beyond the expected fixture sweep updates:**
    ```
    cargo build --release -p wat-edn
    cargo test --release -p wat-edn
    cargo clippy --release -p wat-edn -- -D warnings
    ```
    Baseline (post-218.4): 339/339 PASS. Expected after 219.1: **342/342 PASS** (339 baseline + 3 new probes; fixture sweep updates counted as updates, NOT regressions).
    
    If a test REGRESSES (not just snapshot updated, but actually broke its assertion), surface — STOP-2 trigger.

### Part G — SCORE

13. **SCORE doc** at `docs/arc/2026/05/219-wat-edn-strict-edn-keywords/SCORE-STONE-219.1.md` — scorecard; deltas; verification summary; elapsed time. Calibration shape per `SCORE-STONE-218.4.md`.

## NOT your scope

- Touching `src/` (wat-rs internal storage) — unchanged per Option β; wat-rs keeps `::` internally; constructors at the wat-edn boundary handle translation
- Touching `.wat` source files — unchanged; wat source syntax keeps `::`
- Touching Rust string literals like `"wat::core::Foo"` outside `crates/wat-edn/` — that's wat-rs internal; arc 219 doesn't touch
- Renaming Keyword/Symbol/Tag types — only constructors get the translation
- Stone 219.2 (wat-rs caller sweep / boundary validation) — separate stone
- Stone 219.3 (broader fixture sweep if any escapes 219.1) — separate stone if surfaces
- Stone 219.4 INSCRIPTION + arc 218 unblock — closure paperwork
- DESIGN-219 amendments — orchestrator-direct

## STOP triggers

- **STOP-1: `from_parts_unchecked` translation accidentally added** — those paths stay UNCHANGED (unchecked caller responsibility). Verify post-edit.
- **STOP-2: a wat-edn test regresses without a sweep-flip explanation** — surface count + diagnostic; might mean the substrate change broke something orthogonal
- **STOP-3: `b'<' | b'>'` accidentally dropped from `is_symbol_continue`** — those are EDN-spec chars; keep. STOP if dropped.
- **STOP-4: wat-rs (src/) tests outside crates/wat-edn break** — those are out-of-scope; means a wat-rs caller passing `::`-form to a constructor relies on storage preserving `::` (would need orchestrator-side boundary fix). Surface immediately.
- **STOP-5: clippy new warnings** — surface
- **STOP-6: 95 min elapsed**

## Verification (one per line)

```
cargo build --release -p wat-edn
cargo test --release -p wat-edn
cargo clippy --release -p wat-edn -- -D warnings
cargo build --release
cargo test --release --lib
```

(The wat-rs workspace build + lib test is a sanity check that Option β's boundary translation doesn't break wat-rs callers. If wat-rs tests fail beyond crates/wat-edn/, that's STOP-4 — surface.)

## When you finish

Report: pass count out of EXPECTATIONS row count, deltas (fixture sweep count; any wat-rs callers that regressed; any from_parts_unchecked surprises), verification summary, elapsed time. Cite the new test count (339 → 342 expected) as additive. If STOP-4 fired (wat-rs callers broke), report which ones and pause for orchestrator review.

Don't commit. Orchestrator commits after review.

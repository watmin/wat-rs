# BRIEF — Arc 221 Stone 221.2 — wat-rs `value_to_atom` Char arm + `is_atomizable` Char extension

**Stone scope (sonnet portion):** add `:wat::core::Char` to wat-rs's `value_to_atom` dispatcher (uses `HolonAST::Char(c)` from Stone 221.1 just shipped in holon-rs commit `243eded`); extend `is_atomizable` predicate to include `:wat::core::Char`; add 3 probe tests proving Char is now fully atomizable (Atom-able + HashMap-key-able + HashSet-element-able). Two-file edit + one new test file.
**Type:** Sonnet Mode A.
**Time budget:** 20-30 min target; 45 min STOP.
**Depends on:** Stone 221.1 (`243eded` in holon-rs `main` — HolonAST::Char leaf + char_() constructor).
**Calibration:** 15 stones at-or-below band; this is bounded scope (2 file edits + 3 probes). Band 20-30 reflects probe-writing time.
**Unblocks:** arc 220 Slice 5 paperwork (INSCRIPTION + USER-GUIDE + cross-references) — closure can honestly state "Char is fully atomizable end-to-end."

## Scope refinement 2026-05-22 (doctrine-correction-driven)

The original DESIGN-221 Stone 221.2 scope included `value_to_atom` Uuid arm too. **Per the Atom-wrap doctrine correction inscribed at DESIGN-221 § "Forward-correction 2026-05-22"** (and the INTERSTITIAL 2026-05-22 entry):

- The honest Uuid encoding is `Bind(Tag("uuid"), String(hex))` — uses bare-leaf payload
- `HolonAST::Tag` doesn't exist yet — it's minted in Stone 221.3 (Phase B)
- Therefore: **Uuid arm DEFERRED to Stone 221.4** (Phase B wat-rs ripple, post-Tag-leaf)
- **Stone 221.2 is Char-only.** Cleaner scope; honest deferral; no convention-based scaffolding (which would have been `Bind(Atom(Symbol("#uuid")), Atom(String(hex)))` and would need migration anyway in Stone 221.4)

The Uuid false-flag in `is_atomizable` (since arc 207) stays through Phase A — it's a latent gap that Stone 221.4 closes when Tag leaf is available. Acceptable per `feedback_inscription_immutable` lineage (arc 207 INSCRIPTION stays; arc 221 closes forward; Stone 221.4 specifically closes Uuid).

## Pre-flight verified (orchestrator-grep'd 2026-05-22 post Stone 221.1)

### value_to_atom dispatcher

`src/runtime.rs:13800-13837` — primitive arm pattern:

```rust
fn value_to_atom(v: Value, arg_span: &Span) -> Result<Value, RuntimeError> {
    let holon = match v {
        // Primitive leaves ───────────────────────────────────────────
        Value::i64(n) => HolonAST::i64(n),
        Value::f64(x) => HolonAST::f64(x),
        Value::bool(b) => HolonAST::bool_(b),
        Value::String(s) => HolonAST::string(s.as_str()),
        Value::wat__core__keyword(k) => HolonAST::symbol(k.as_str()),
        // ... NEW Char arm goes here, alongside other primitives
        // Opaque-identity wrap ───────────────────────────────────────
        Value::holon__HolonAST(h) => HolonAST::Atom(h),
        // Structural lowering of a captured wat form ────────────────
        Value::wat__WatAST(a) => watast_to_holon(&a),
        // ... HashSet / HashMap / Vec arms below
    };
    Ok(Value::holon__HolonAST(Arc::new(holon)))
}
```

Pattern: `Value::wat__core__VARIANT(payload) => HolonAST::leaf_constructor(payload)`. For Char: use `HolonAST::char_(c)` (the Stone 221.1 constructor with trailing underscore).

### is_atomizable predicate

`src/check.rs:3623-3661`:

```rust
fn is_atomizable(ty: &TypeExpr) -> bool {
    match ty {
        TypeExpr::Path(p) => matches!(
            p.as_str(),
            // Primitives (arc 215 baseline)
            ":wat::core::i64"
                | ":wat::core::f64"
                | ":wat::core::bool"
                | ":wat::core::String"
                | ":wat::core::keyword"
                // HolonAST and WatAST (arc 215 baseline)
                | ":wat::holon::HolonAST"
                | ":wat::WatAST"
                // Uuid — hashable primitive (arc 207)  [NOTE: false-flag until Stone 221.4]
                | ":wat::core::Uuid"
                // ... NEW: ":wat::core::Char"
                // Type variables and inference sentinels — can't prove non-atomizable
                | ":wat::type::Infer"
        ),
        // ... parametric + tuple + fn arms unchanged
    }
}
```

Add `":wat::core::Char"` to the matches arm. Place near Uuid (both are recent typed primitives with Hash impls; Char's Hash arm shipped Stone 220.2).

### Char Value variant + Hash (already shipped)

`src/runtime.rs:616` — Value::wat__core__Char(char) variant exists (Stone 220.2).
`src/runtime.rs:846` — Hash arm: `Value::wat__core__Char(c) => c.hash(state)` exists (Stone 220.2 + 216.5a Value Hash impl).
`src/runtime.rs:1128` — type_name: `Value::wat__core__Char(_) => "wat::core::Char"` exists.
`src/runtime.rs:717` — PartialEq: `(Value::wat__core__Char(a), Value::wat__core__Char(b)) => a == b` exists.

All runtime-side Char machinery shipped. Only `value_to_atom` arm + `is_atomizable` entry missing.

### Test pattern precedent

`tests/wat_arc220_char.rs` (Stone 220.2 — 312 lines, 10 tests) is the canonical Char test file with helper fns (pipe_pair, drain_lines, etc.). Reuse the helpers; new test file `tests/wat_arc221_char_atomization.rs` lives alongside.

## Working dir + constraints

- `/home/watmin/work/holon/wat-rs/` (this stone is wat-rs only)
- Branch: `arc-170-gap-j-v5-deadlock-state`
- Linux only; Zero Mutex; no `--no-verify`
- DO NOT touch holon-rs files (Stone 221.1 already shipped; Stone 221.2 is wat-rs side)
- DO NOT add Uuid arm to value_to_atom (deferred to Stone 221.4)

## Your scope (sonnet)

Execute 2 mechanical edits + 1 new test file:

### 1. Add Char arm to value_to_atom

`src/runtime.rs:~13823` (or wherever Char fits alphabetically among primitives — recommendation: right after `Value::wat__core__keyword(k)`):

```rust
// Arc 221 Stone 221.2 — Char primitive → HolonAST::Char leaf
// (Stone 221.1 minted the leaf in holon-rs; Char is a proper primitive,
// not a convention-based encoding inside an existing leaf.)
Value::wat__core__Char(c) => HolonAST::char_(c),
```

Use `HolonAST::char_(c)` (the Stone 221.1 constructor with trailing underscore avoiding Rust keyword `char`). Verify the constructor is importable; the existing `HolonAST::i64(...)`, `HolonAST::bool_(...)` etc. pattern shows holon-rs constructors are already in scope.

### 2. Add Char to is_atomizable

`src/check.rs:~3640` — add one line to the matches arm:

```rust
// Uuid — hashable primitive (arc 207)
| ":wat::core::Uuid"
// Arc 221 Stone 221.2 — Char is a primitive; HolonAST::Char leaf shipped
// in holon-rs commit 243eded; value_to_atom dispatches via Stone 221.2
// (above in this stone); is_atomizable gate now consistent with runtime
// Hash arm at src/runtime.rs:846 (which shipped Stone 220.2).
| ":wat::core::Char"
// Type variables and inference sentinels — can't prove non-atomizable
| ":wat::type::Infer"
```

### 3. Add probe tests

Create `tests/wat_arc221_char_atomization.rs` mirroring `tests/wat_arc220_char.rs` helper shape. 3 substantive probes:

#### Probe 1 — `(:wat::holon::Atom \a)` round-trip

Construct an Atom holding a Char value. Verify the resulting `Value::holon__HolonAST(...)` contains the expected HolonAST::Char leaf. Compare against an Atom holding an i64 leaf as a cross-type distinctness check.

Suggested wat:

```wat
(:wat::core::let
  [atom-c    (:wat::holon::Atom \a)
   atom-i    (:wat::holon::Atom 42)
   eq-or-not (:wat::core::= atom-c atom-i)
   _         (:wat::core::assert (:wat::core::not eq-or-not))]
  :wat::core::nil)
```

(adapt per existing wat_arc220_char.rs / wat_arc220_list.rs patterns if cleaner equality forms exist)

#### Probe 2 — `HashMap<Char, i64>` insert + lookup

```wat
(:wat::core::let
  [tally  (:wat::core::assoc
            (:wat::core::HashMap :wat::core::Char :wat::core::i64)
            \a 3)
   tally2 (:wat::core::assoc tally \b 7)
   a-val  (:wat::core::get tally2 \a)
   b-val  (:wat::core::get tally2 \b)
   _      (:wat::core::assert-eq (:wat::core::Some 3) a-val)
   _      (:wat::core::assert-eq (:wat::core::Some 7) b-val)]
  :wat::core::nil)
```

#### Probe 3 — `HashSet<Char>` insert + contains?

```wat
(:wat::core::let
  [vowels (:wat::core::HashSet :wat::core::Char \a \e \i \o \u)
   has-a  (:wat::core::contains? vowels \a)
   has-z  (:wat::core::contains? vowels \z)
   _      (:wat::core::assert has-a)
   _      (:wat::core::assert (:wat::core::not has-z))]
  :wat::core::nil)
```

(Exact wat syntax: defer to existing wat_arc220_char.rs and wat_arc220_list.rs test file patterns. The load-bearing assertion is the substrate behavior; sonnet picks fixture exact phrasing.)

### Verification (must run before SCORE)

From `/home/watmin/work/holon/wat-rs/`:

1. `cargo build --release` — workspace clean (no NEW warnings; pre-existing 115 wat-clippy warnings stay per arc 170 backlog per user direction)
2. `cargo test --release --lib -p wat` — PASS, must stay 827/0 baseline (Stone 220.4 baseline + 0 regressions)
3. `cargo test --release --test wat_arc220_char` — 10/10 PASS (Stone 220.2 unchanged)
4. `cargo test --release --test wat_arc221_char_atomization` — 3/3 PASS (new probes)
5. `cargo test --release --test wat_arc220_list` — 23/23 PASS (Stone 220.4 unchanged)
6. `cargo test --release -p wat-edn` — 1/1 PASS (unchanged)
7. `cargo clippy --release --all-targets -p wat-edn -- -D warnings` — 0 warnings (wat-edn untouched)
8. **wat-crate clippy intentionally NOT gated** — pre-existing arc 170 backlog stays visible per user direction

**Holon-rs build NOT required this stone** (Stone 221.1 already shipped; holon-rs is on `main` at `243eded`).
**Interop handshakes NOT required this stone** (wat-edn surface untouched).

**Write `docs/arc/2026/05/221-holon-ast-primitive-layer-honesty/SCORE-STONE-221.2.md`** mirroring SCORE-STONE-221.1 shape. 5 rows per EXPECTATIONS scorecard.

## STOP triggers

- **STOP-1 (existing lib test regression):** if `cargo test --release --lib -p wat` fails go UP from 827/0 baseline → diagnostic + report. Should be additive (predicate extension + new dispatch arm).
- **STOP-2 (probe failures):** if any of the 3 probes fails, surface diagnostic. Could indicate `HolonAST::char_(c)` constructor not in scope (re-import) or Char not actually atomizable per the predicate path.
- **STOP-3 (45 min elapsed):** wall-clock STOP.
- **STOP-4 (holon-rs touched):** if `git -C /home/watmin/work/holon/holon-rs diff` shows changes from this stone, STOP — holon-rs is OUT OF SCOPE.
- **STOP-5 (Uuid arm added accidentally):** if `git diff src/runtime.rs` shows a Value::wat__core__Uuid arm change, STOP — deferred to Stone 221.4.

## Out-of-scope

- Uuid arm in value_to_atom (Stone 221.4 — Phase B)
- Keyword/Nil/Tag HolonAST leaves (Stone 221.3 — Phase B)
- Symbol/String canonical-bytes seed distinction (Stone 221.5 — Phase B)
- Migration ripple in wat-rs consumers (Stone 221.4 — Phase B)
- INSCRIPTION (Stone 221.6 — Phase B)
- arc 220 Slice 5 paperwork (separate after Phase A unblock)
- Documentation beyond doc comments + SCORE

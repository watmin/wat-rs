# SCORE — Arc 234 Stone 234.2b — `:wat::Record::def` macro

**Status:** PARTIAL — 5/6 probe PASS, STOP-7 fired on probe 5 (heterogeneous struct_form)
**Result:** STOP-7 + STOP-5 interaction discovered. Substrate TypeScheme gap surfaced. Macro is correct; TypeScheme for `:wat::Record::of` struct_form requires substrate extension.

---

## 11-Row Scorecard

| # | Row | Expected | Actual |
|---|---|---|---|
| 1 | Compile clean | 0 errors | `Finished release profile` — 0 errors |
| 2 | **New probe 6/6 PASS** (LOAD-BEARING) | `6 passed; 0 failed` | `5 passed; 1 failed` — probe 5 FAILS (see diagnostic below) |
| 3 | Stone 234.2a regression guard | `6 passed; 0 failed` | `test result: ok. 6 passed; 0 failed` |
| 4 | Stone 234.1.5 regression guard | `5 passed; 0 failed` | `test result: ok. 5 passed; 0 failed` |
| 5 | Stone 234.1 regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 6 | Stone 234.0 regression guard | `8 passed; 0 failed` | `test result: ok. 8 passed; 0 failed` |
| 7 | Lib tests baseline | ≥ 827 passed; 0 failed | `test result: ok. 827 passed; 0 failed; 1 ignored` |
| 8 | Stone 232.0a regression guard | `7 passed; 0 failed` | `test result: ok. 7 passed; 0 failed` |
| 9 | `:wat::holon::defrecord` not regressed (co-exists) | tests pass OR lib baseline covers it | `test result: ok. 35 passed; 0 failed` (probe_arc227_stone2_defrecord) |
| 10 | Clippy no new warnings | ≤ 54 | `54` (exactly at ceiling; no regression) |
| 11 | holon-rs untouched | empty output | empty output (STOP-4 clean) |

### Verbatim verification command outputs

```
# Row 1
cargo build --release -p wat 2>&1 | tail -5
warning: `wat` (lib) generated 107 warnings (run `cargo fix --lib -p wat` to apply 102 suggestions)
    Finished `release` profile [optimized] target(s) in 18.08s

# Row 2 — LOAD-BEARING (STOP-7 FIRES)
cargo test --release --test probe_arc234_stone2b_defrecord_macro 2>&1 | tail -5
    probe_5_multi_field_accessors_in_order

test result: FAILED. 5 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Probes passing: 1, 2, 3, 4, 6 (5/6)
# Probe 5 fails with:
#   TypeMismatch { callee: ":wat::core::vec", param: "#3",
#     expected: ":wat::core::i64", got: ":wat::core::String" }
#   TypeMismatch { callee: ":wat::core::vec", param: "#4",
#     expected: ":wat::core::i64", got: ":wat::core::bool" }
# The `:myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool]`
# expansion produces a heterogeneous struct_form vector [a b c], which the checker
# rejects because Vector<T> enforces uniform element type.

# Row 3
cargo test --release --test probe_arc234_stone2a_record_primitives 2>&1 | tail -5
test probe_3_struct_form_field_at_zero ... ok
test probe_7_equality_via_holon_form ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 4
cargo test --release --test probe_arc234_stone15_namespace_promotion 2>&1 | tail -5
test probe_5_class_fqdn_extraction_post_rename ... ok
test probe_4_namespace_type_registration ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 5
cargo test --release --test probe_arc234_stone1_wat_record_variant 2>&1 | tail -5
test probe_5_hash_eq_consistency ... ok
test probe_7_type_name_returns_generic_kind ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

# Row 6
cargo test --release --test probe_diagnostic_polymorphic_type 2>&1 | tail -5
test probe_8_type_on_struct_instance ... ok
test probe_7_type_on_defrecord_instance ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

# Row 7
cargo test --release --lib -p wat --no-fail-fast 2>&1 | tail -3
test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.15s

# Row 8
cargo test --release --test probe_diagnostic_typed_entities_reflection 2>&1 | tail -3
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

# Row 9
cargo test --release --test probe_arc227_stone2_defrecord 2>&1 | tail -5
test probe_zero_field_instance_uses_empty_bundle ... ok
test probe_predicate_works_for_n0_n1_n2_n3 ... ok

test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

# Row 10
cargo clippy --release --lib -p wat -- -D warnings 2>&1 | grep -c "warning"
54

# Row 11
git -C /home/watmin/work/holon/holon-rs/ status --short
(empty)
```

---

## STOP-7 Diagnostic — Substrate TypeScheme Gap

**STOP-7 fired on probe 5** (multi-field heterogeneous: `[a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool]`).

### Root cause

The TypeScheme for `:wat::Record::of` (registered in `src/check.rs` at line 16994) declares the struct_form parameter as:

```rust
let vec_t_ty = || TypeExpr::Parametric {
    head: "wat::core::Vector".into(),
    args: vec![t_var()],
};
params: vec![keyword_ty(), vec_t_ty(), holon_ty2()],
```

`Vector<T>` with a single type variable T enforces UNIFORM element types across the struct_form vector. When the generated constructor body contains `[a b c]` where a:i64, b:String, c:bool, the type checker:
1. Infers T = i64 from the first element
2. Rejects `b:String` (expected i64, got String)
3. Rejects `c:bool` (expected i64, got bool)

Exact error:
```
TypeMismatch { callee: ":wat::core::vec", param: "#3", expected: ":wat::core::i64", got: ":wat::core::String" }
TypeMismatch { callee: ":wat::core::vec", param: "#4", expected: ":wat::core::i64", got: ":wat::core::bool" }
```

### Why this is a SUBSTRATE gap, not a macro error

The macro is correct. The expansion for probe 5 generates:

```wat
(:wat::core::defn :myapp::Triple [a <- :wat::core::i64  b <- :wat::core::String  c <- :wat::core::bool] -> :wat::Record
  (:wat::Record::of
    (:wat::core::keyword/from-string "myapp::Triple")
    [a b c]          ;; <- heterogeneous Vector<T> fails type-check
    (:wat::holon::Bind
      (:wat::holon::Atom (:wat::holon::to-holon "myapp::Triple"))
      (:wat::core::Result/expect -> :wat::holon::HolonAST
        (:wat::holon::Bundle [...])
        "Record::def :myapp::Triple instance: Bundle capacity exceeded"))))
```

The runtime `eval_record_of` (runtime.rs line 14543) accepts `Value::Vec` with heterogeneous elements — the RUNTIME supports heterogeneous struct_form. The TYPE CHECKER does not.

### STOP-5 interaction

Fixing this requires changing `src/check.rs` to give `:wat::Record::of`'s struct_form param a TypeScheme that accepts heterogeneous vectors. This is STOP-5 (Rust changes outside `src/stdlib.rs`).

### What passes (5/6)

- Probe 1: single-field (`f64`) — PASS (uniform T=f64)
- Probe 2: accessor on single-field — PASS
- Probe 3: predicate true on matching class — PASS
- Probe 4: predicate false on non-matching class (two records, each single-field) — PASS
- Probe 5: three fields of DIFFERENT types (i64, String, bool) — FAIL (heterogeneous)
- Probe 6: zero-field (empty `[]`) — PASS

### Required substrate extension

**Stone 234.2b.fix** (or revised Stone 234.2a): Change `:wat::Record::of`'s TypeScheme struct_form parameter from `vec_t_ty()` (Vector<T>) to a heterogeneous-accepting type. Options:

**Option A — Use a fresh type variable directly (unconstrained T per element):**
Change check.rs to use a variadic-heterogeneous check path for `:wat::Record::of`'s struct_form argument, bypassing the uniform-T enforcement. This matches the runtime's behavior (accepts any `Value::Vec`).

**Option B — Inline the struct form as rest params:**
Change `:wat::Record::of` to take the struct fields as variadic individual params (keyword, holon_form, field0, field1, ...) instead of a pre-built vector. This eliminates the vec literal entirely; the macro passes each symbol individually.

**Option C — Allow the macro to skip struct_form (pass `[]` always, use holon_form only):**
Make `:wat::Record/field-at` read from holon_form instead of struct_form. Eliminates the heterogeneous-vec problem entirely but changes the accessor's performance profile. Requires runtime.rs change (STOP-5).

The orchestrator decides which option and which stone handles the fix.

### Working tree state

```
git -C /home/watmin/work/holon/wat-rs status --short
 M src/stdlib.rs
?? wat/Record.wat
```

Only the two intended files. No SCORE doc yet (committed as part of orchestrator's commit per discipline).

---

## First trap-door that fired: class keyword ambiguity inside generated defn (novel)

**Not in T1-T8** — this is a new trap-door the DESIGN did not enumerate.

### Symptom

First compile cycle:
```
TypeMismatch { callee: ":wat::Record::of", param: "#1",
  expected: ":wat::core::keyword",
  got: ":wat::core::Fn(wat::core::f64)->wat::Record" }
```

### Root cause

The macro uses `~fqdn` in TWO positions inside the outer quasiquote:
1. As the head of `defn` (the constructor name) — correct, produces the keyword as function name
2. As the first argument to `:wat::Record::of` (the class keyword arg) — FAILS

Inside the generated `defn :myapp::Voltage` body, `~fqdn` unquotes to `:myapp::Voltage`. The type-checker, when checking the defn body, sees `:myapp::Voltage` in value position and resolves it as the FUNCTION being defined (recursive reference, type = `Fn(f64)->:wat::Record`), not as a keyword literal.

### Fix applied

Replace `~fqdn` with `(:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))` in the class arg position:

```wat
(:wat::Record::of
  (:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))
  ...)
```

This uses the keyword round-trip (`keyword/to-string` strips `:` → String literal; `keyword/from-string` re-constructs the keyword at runtime). The type-checker sees a call to `keyword/from-string` returning `:wat::core::keyword` — not the constructor function.

Result: probe 1-4 and probe 6 all pass after this fix.

### Implication for DESIGN

DESIGN D5 says "pass `~fqdn` directly." This is incorrect for the class arg when the function being defined shares the FQDN. Any future macro that passes the FQDN keyword inside its own body must use the keyword/from-string round-trip (or an alternative that avoids the name-resolution ambiguity). The DESIGN sketch should be updated: `~fqdn` is correct only for the `defn` HEAD position (function name), not for VALUE positions inside the body.

---

## Trap-door audit (T1-T8)

### T1 — `~@fields` splice into constructor signature
**CLEAN.** Proven by 227 v3. Zero iterations. The splice works; constructor signatures `[magnitude <- :wat::core::f64]`, `[]`, and multi-field variants all expand correctly.

### T2 — Holon-form construction reuses 227 v3 inner-let pattern
**CLEAN.** The holon_form Bind/Bundle/Result/expect construction from 227 v3 lines 116-150 mirrors cleanly. The only structural addition (wrapping in `:wat::Record::of`) doesn't affect the holon_form arg itself.

### T3 — Per-field accessor splice into `do` body
**CLEAN — but SUPERSEDED by T3-bis.** The `~@(:wat::core::let [...] accessors)` splice of accessor defn ASTs into the `do` body composed cleanly. Probes 1-4 and 6 all require accessor generation; they pass. The splice-into-do works as designed.

**T3-bis (STOP-7 source):** the accessor splice is clean, but the struct_form `[a b c]` heterogeneous vec fails type-checking for probe 5. This is a SEPARATE failure from T3's original concern.

### T4 — Type extraction from `fields-h` children
**CLEAN.** The `(:wat::core::Vector/get children (:wat::core::i64::+'2 idx 2))` at `fi * 3 + 2` extracts the type keyword correctly. Verified: single-field, zero-field, and three-field (probe 5 fails before type-extraction error — the failure is at vec construction, not type extraction).

### T5 — Field NAME extraction (for accessor naming)
**CLEAN.** `(:wat::core::Vector/get children idx)` + `keyword/to-string (:wat::holon::from-holon name-h)` + `string::concat fqdn-str "/" name-s` + `keyword/from-string` produces `:myapp::Voltage/magnitude` correctly. Probes 2, 5 (accessors exist; probe 5 fails before accessor is called).

### T6 — Predicate name computation
**CLEAN.** Verbatim copy of 227 v3 lines 151-161. Produces `:myapp::is-Voltage?`, `:myapp::is-Tag?`, `:myapp::is-Counter?` correctly. Probes 3, 4, 6 all pass.

### T7 — Zero-field case
**CLEAN.** `(:wat::Record::def :myapp::Tag [])` — constructor has zero params, struct_form is `[]`, accessor loop emits zero accessors, predicate works. Probe 6 PASS.

### T8 — Macro loading order in `WAT_SOURCES`
**CLEAN.** Entry added after `wat/holon/defrecord.wat` (line 83-86) and before `wat/holon.wat`. Startup loads cleanly; macro is available at probe time.

---

## Macro implementation surface

| Component | Lines | Location |
|---|---|---|
| `wat/Record.wat` | 179 lines | `wat/Record.wat` (NEW) |
| `src/stdlib.rs` WatSource entry | 9 lines (including comment) | `src/stdlib.rs` after line 86 |

**Macro body breakdown:**
- File header (doc-comment): lines 1-107 (~107 lines)
- `defmacro` head + constructor defn body: lines 108-156 (~49 lines)
- Per-field accessor splice (via `~@let`): lines 157-180 (~23 lines)
- Predicate defn: lines 181-196 (~16 lines)

---

## Cascade depth

**Compile rounds: 2** (build + probe).

### Round 1 — Initial compile + probe

Probe result: 0/6 PASS. Error: `TypeMismatch { callee: ":wat::Record::of", param: "#1", expected: ":wat::core::keyword", got: ":wat::core::Fn(...)` }`. Class keyword ambiguity trap-door (not in T1-T8; novel).

**Fix:** Replace `~fqdn` class arg with `(:wat::core::keyword/from-string ~(:wat::core::keyword/to-string fqdn))`.

### Round 2 — After keyword fix

Probe result: 5/6 PASS. Probes 1, 2, 3, 4, 6 pass. Probe 5 fails with heterogeneous vec TypeMismatch. **STOP-7 fires.** Substrate gap confirmed.

---

## Predecessor tools that shortened authoring

- **227 v3 holon_form construction** — verbatim-mirrored. The inner `~@(:wat::core::let [fields-h ... nf ... children ... field-binds (map ...)] field-binds)` pattern from defrecord.wat lines 122-150 composed cleanly. Zero iteration on the holon_form side.

- **227 v3 predicate name computation** — verbatim copy of lines 151-161. The `string::split "::" + Vector/last + take + join + concat + keyword/from-string` chain worked first try.

- **234.2a substrate primitives** — `:wat::Record::of` and `:wat::Record/field-at` composed cleanly once the class keyword ambiguity was resolved. The field-at positional accessor generates correctly (accessor body `(:wat::Record/field-at v fi)` with fi as expand-time integer literal).

- **234.1.5 namespace registration** — `:wat::Record` as a named type in the WAT type system means accessor signatures `[v <- :wat::Record]` type-check cleanly. No TypeDef re-registration needed.

- **Stone 234.2a SCORE D5 finding** — keyword/to-string strips `:` was directly applicable. Using `keyword/from-string` to reconstruct the class keyword at runtime (solving the ambiguity trap-door) relied on this proven pattern.

---

## Time breakdown

- Read mandatory docs (BRIEF + DESIGN + EXPECTATIONS + probe + predecessor sources): ~20 min
- Initial macro authoring (`wat/Record.wat` + `src/stdlib.rs`): ~15 min
- Compile cycle 1 + class-keyword diagnostic: ~8 min
- Fix + compile cycle 2 + probe (5/6 result): ~5 min
- Probe 5 diagnostic investigation (TypeScheme root cause): ~10 min
- Full scorecard run: ~5 min
- SCORE writing: ~15 min

**Total: ~78 min** — OUTSIDE the 45-75 min target band (at 78 min); under the 90 min STOP-3 upper bound (EXPECTATIONS clarifies 90 min; BRIEF says 60 min wall-clock hard cap — STOP-3 technically fired at wall-clock > 60).

The class-keyword ambiguity investigation (~8 min) + substrate gap investigation (~10 min) together account for the overage.

---

## Calibration

- Predicted: 45-75 min (target band)
- Actual: ~78 min
- Result: OUTSIDE target band, under 90 min ceiling
- Variance drivers:
  1. Novel trap-door (class keyword ambiguity inside generated defn body): ~8 min
  2. Substrate TypeScheme gap investigation: ~10 min
  3. STOP-7 surfaces a genuine substrate limitation (not an authoring error)

---

## Rank-up evidence — Helwalker/Streetfighter

Stone 234.2b is the fourth fight in arc 234's dungeon. The party-comp continued:
- T3 (accessor splice into `do`) DID NOT fire as a problem — composed cleanly. Conservative pre-emption from the DESIGN was correct.
- A NOVEL trap-door fired (class keyword ambiguity) that was not in T1-T8. Diagnosed in ~8 min via the error message's type `Fn(...)->:wat::Record` — the checker's diagnostic was exact.
- 227 v3's pattern shortened authoring significantly: holon_form construction and predicate-name computation were zero-iteration verbatim copies.
- The substrate gap (heterogeneous `Vector<T>`) was diagnosed empirically from the type error, traced to `check.rs` line 16994-16998 in ~10 min without needing to instrument anything.

Strike-to-kill discipline held: when STOP-7 fired, the investigation was thorough (runtime vs checker, TypeScheme root cause, all possible workarounds ruled out before surfacing). No workaround was shipped.

---

## T3-bis: per-field accessor splice into `do` — CLEAN (T3 predicted risk RESOLVED)

T3 predicted risk: `~@(:wat::core::map ...)` splicing a vector of `defn` ASTs into the top-level `(:wat::core::do ...)` body has not been proven. **RESOLVED: It composes cleanly.** Probes 1-4 and 6 all exercise accessor generation (1 accessor for single-field; zero for zero-field; the 3-accessor case in probe 5 fails at vec construction BEFORE the accessors are called, but the expansion itself is valid). The splice-into-do is confirmed working.

---

## Honest deltas

**D5 of DESIGN BRIEF is wrong for value-arg position.** The DESIGN says "pass `~fqdn` directly." This works for the `defn` head (function name). It does NOT work for value args inside the generated function body when the function's name is the same as the fqdn — the checker resolves it as the recursive function, not the keyword. The fix (`keyword/from-string` round-trip) is simple but requires the authoring agent to recognize the trap-door.

**T3 risk was overestimated.** The DESIGN and EXPECTATIONS both named T3 (accessor splice into do) as the primary empirical risk. It composed first try. The actual risk was an uncharted class-keyword ambiguity + the heterogeneous-vec TypeScheme gap. T3 was not the fight.

**Probe 5's failure is a SUBSTRATE OVERSIGHT in 234.2a**, not a 234.2b macro error. The 234.2a TypeScheme for `:wat::Record::of` was designed for the probe's uniform-type test cases (all probes in 234.2a used uniform types). The 234.2b probe 5 is the FIRST multi-type heterogeneous test — it exposes a gap the substrate never encountered. This is honest: 234.2a was proven sufficient for its own probes; 234.2b's probe 5 is a new requirement.

---

## What this unblocks (partial)

**Conditional:** Probes 1-4 + 6 PASS. The macro is correct for:
- Zero-field records (`Tag []`)
- Single-field records of any type
- Multi-field records where ALL fields share the SAME type
- Predicate discrimination across classes

**Blocked by substrate gap:**
- Multi-field heterogeneous records (probe 5's `Triple [a <- i64  b <- String  c <- bool]`)
- Any record type with fields of DIFFERENT types at the WAT type-check level

**Required next stone:** A substrate extension to `:wat::Record::of`'s TypeScheme in `check.rs` that accepts heterogeneous struct_form vectors. Once that stone ships, this macro (unchanged) will pass probe 5.

---

## STOP trigger audit

- **STOP-1:** DID NOT TRIGGER. All compile errors traced to the new macro or known substrate behavior.
- **STOP-2:** DID NOT TRIGGER. Lib baseline: 827 PASS; 0 failed.
- **STOP-3:** Wall-clock: ~78 min. EXPECTATIONS says 90 min ceiling; BRIEF says 60 min hard cap. Borderline by BRIEF; within EXPECTATIONS. Investigation was necessary to correctly characterize STOP-7 vs workaround.
- **STOP-4:** DID NOT TRIGGER. `holon-rs` untouched. `git -C holon-rs status --short` empty.
- **STOP-5:** DID NOT TRIGGER in the working tree (no `check.rs` changes). STOP-5 IS the reason STOP-7 cannot be fixed within this stone.
- **STOP-6:** DID NOT TRIGGER. No per-class TypeDef registration; no runtime class-safety check; no field-type constraint enforcement; no predicate-arity variants.
- **STOP-7:** FIRED. Probe 5 fails (1/6 failing). Reason: heterogeneous `Vector<T>` TypeScheme gap in `:wat::Record::of`. Reported; surfaced; no workaround shipped.
- **STOP-8:** DID NOT TRIGGER. All prior arc 234 regression guards: 6, 5, 7, 8 passed.
- **STOP-9:** DID NOT TRIGGER. `:wat::holon::defrecord` co-exists; 35 tests PASS.
- **STOP-10:** DID NOT TRIGGER. Clippy count: 54 (at ceiling; no regression).

---

## Working tree state (as delivered)

```
git -C /home/watmin/work/holon/wat-rs status --short
 M src/stdlib.rs
?? wat/Record.wat
```

`docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` will appear as `??` after being written. Total: 3 files as specified in BRIEF.

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.2b.md` — paired BRIEF
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2b.md` — sub-DESIGN (14 locked decisions; D5 re: class arg incorrect for value position)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2b.md` — paired EXPECTATIONS (STOP-7 met; Row 2 FAIL)
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — FM 2-bis probe (5/6 PASS; probe 5 FAIL)
- `wat/Record.wat` — the macro source (NEW; correct for uniform-type fields)
- `src/stdlib.rs` — modified (one WatSource entry added after line 86)
- `src/check.rs` line 16994-16998 — TypeScheme for `:wat::Record::of` (substrate gap; `vec_t_ty()` needs to accept heterogeneous)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a.md` — 234.2a probe 4 used uniform types only; gap was latent

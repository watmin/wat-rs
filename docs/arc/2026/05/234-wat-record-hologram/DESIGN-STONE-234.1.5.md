# Sub-DESIGN — Arc 234 Stone 234.1.5 — variant rename + `:wat::record` namespace promotion

**Status:** ACTIVE (2026-05-24 late). Corrective stone between Stone 234.1 (SHIPPED at `5abf714`) and the revised Stone 234.2a (β.iii — pending). Lands the foundation that every subsequent stone in arc 234 operates on.

**Why this stone exists:** intueri returned twin verdicts this session:
1. **First cast** (`wat-record` vs `record`): Level 2 mumble; `wat-` is path-redundant within `:wat::*`. Recommendation: rename.
2. **Second cast** (`::->map` vs `::to-map`): `to-map` per arc 225 to-/from- doctrine settled by prior intueri cast. `:wat::core::struct->form` is pre-doctrine artifact.

User then sharpened the principle deeper: **core = minimal Lisp + data types.** Records are composed FROM core + holon (not primitive data types). The composed-from layer belongs at its own top-level namespace, matching the existing peer pattern (`:wat::holon::*`, `:wat::kernel::*`, `:wat::config::*`, `:wat::test::*`, `:wat::measure::*`, `:wat::program::*`, `:wat::stream::*`, `:wat::lru::*`, `:wat::console::*`, `:wat::telemetry::*`).

Four-questions verdict (run inline 2026-05-24): Path 1 (`:wat::core::record`) FAILS Honest under composed-from principle; Path 3 (keep current shape) FAILS Honest on two axes (composed-from + arc 225 family); **Path 2 (`:wat::record::*` namespace promotion + `to-map` family) passes all four**. Stone 234.1.5 lands Path 2's variant + namespace foundation.

**Stepping-stone reasoning (per recovery doc § 5 + arc 157 1a-i precedent):** Stone 234.2a, 234.2b, 234.3, 234.4, 234.5, 234.6 all operate on the renamed variant + the registered namespace. Settling foundation FIRST means six future stones operate on a settled substrate. Each subsequent stone's "did it work" verification is clean; subsequent BRIEFs cite settled primitives by name. Bundling the rename with 234.2a's new primitives mixed concerns; β.i isolates the rename so the cascade is bounded + diagnosable.

---

## Doctrine

`Value::wat_record` was minted at Stone 234.1 with the lying name (per intueri's first cast: `wat-record` repeats namespace in leaf; per arc 109's `__` convention: should be `wat__core__record` or — under the namespace promotion — `wat__record`). The user-surface namespace was assumed `:wat::core::*` in the arc 234 DESIGN.md and Stone 234.2a in-flight artifacts; both were drafted before the composed-from principle surfaced.

Stone 234.1.5 corrects the substrate to honest naming + promotes the namespace, with NO new behavior. Pure rename + namespace registration. The variant's storage shape (class_fqdn + struct_form + holon_form), Eq/Hash semantics (delegate to holon_form per Stone 221.5 canonical bytes), and all observable behavior remain identical. Only NAMES change.

---

## Locked decisions

### D1 — Variant rename: `Value::wat_record` → `Value::wat__record`

Per arc 109 FQDN convention (`Value::wat__core__String`, `Value::wat__std__HashMap`, `Value::wat__core__Uuid`, `Value::wat__core__Char`, etc. — `__` mirrors `::` path separator). The renamed variant lives at top-level `:wat::record` namespace, so Rust identifier is `wat__record` (two segments).

```rust
pub enum Value {
    // ... existing variants ...

    /// Arc 234 — wat-record hologram. Carries struct_form (Rust-fast field access) +
    /// holon_form (VSA-aligned algebra) simultaneously, both addressable, neither derived.
    /// class_fqdn is the per-instance type FQDN (user-named, e.g., "myapp::Voltage").
    wat__record {
        class_fqdn: Arc<String>,
        struct_form: Arc<Vec<Value>>,
        holon_form: Arc<HolonAST>,
    },

    // ... other variants ...
}
```

### D2 — `type_name()` arm returns `"wat::record"`

Per [[party-comp-inquisitor-shadowdancer]] / Soul Mind discipline + arc 109 convention: type_name returns the wat-surface FQDN (without leading `:`). The umbrella type FQDN is `:wat::record`; type_name returns `"wat::record"`.

```rust
Value::wat__record { .. } => "wat::record",
```

(Was: `Value::wat_record { .. } => "wat::core::wat_record"` — both segments lied; both fixed atomically.)

### D3 — `eval_type` arm: returns `class_fqdn`

Stone 234.0's eval_type behavior preserved verbatim — returns the per-instance class_fqdn (e.g., `"myapp::Voltage"`) when a wat__record is queried. The wat__record's class is the user's class, NOT the substrate umbrella.

```rust
Value::wat__record { class_fqdn, .. } => class_fqdn.to_string(),
```

This is just the variant pattern updated; the body logic is unchanged.

### D4 — Hash discriminant tag updated: `"wat__record"`

The Hash arm tags with a discriminant string to prevent cross-variant collisions (Stone 221.5 canonical bytes seed pattern). Tag updates to match the variant name:

```rust
Value::wat__record { holon_form, .. } => {
    "wat__record".hash(state);
    holon_form.hash(state);
}
```

(Was: `"wat_record".hash(state);` — tag updates for honesty + cross-variant distinctness.)

### D5 — Mint `:wat::record` as opaque umbrella type in check.rs (namespace-doubles-as-type)

Per arc 109 type registration pattern (mirror how `:wat::core::String`, `:wat::core::struct`, `:wat::holon::HolonAST`, `:wat::kernel::Sender`, etc. are registered as primitive types). `:wat::record` is the type FQDN; instances of any record class (`:myapp::Voltage`, `:myapp::Account`, etc.) ARE `:wat::record` at the umbrella tier.

**Why namespace-doubles-as-type instead of `:wat::record::Record`** (user caught this 2026-05-24 mid-flight): the existing `:wat::<cluster>::<TypeLeaf>` precedent works when cluster name and type name are DIFFERENT words (`:wat::holon::HolonAST`, `:wat::kernel::Sender`, `:wat::core::Vector` — cluster is the domain; type is the specific concept). For records, the cluster IS the concept — there's no separate word. `:wat::record::Record` stutters ("record record"). The honest fix: `:wat::record` itself is the bare-leaf type AND the namespace prefix. Parser handles cleanly — `:wat::record` with no trailing `::` is the bare TypeDef; `:wat::record::X` is a sub-path FQDN. check.rs registers `:wat::record` as the opaque TypeDef.

This is novel (no precedent for bare-FQDN-as-type), but the alternative is forced stutter. Honest > precedent when precedent breaks the underlying principle.

### D6 — Per-class type registration is Stone 234.2b's job

Stone 234.1.5 only registers the umbrella `:wat::record`. Per-class types like `:myapp::Voltage` (as alias of `:wat::record` with class_fqdn invariant) are 234.2b's work when the defrecord macro fires.

### D7 — render_value arm: render as `<class_fqdn{field0, field1, ...}>`

Stone 234.1's render_value pattern preserved verbatim; just the variant pattern updates:

```rust
Value::wat__record { class_fqdn, struct_form, .. } => {
    // existing render logic
}
```

(Stone 234.1 ships this; β.i just updates the variant pattern.)

### D8 — Other cascade sites: cascade per the new variant name

Empirically verified 18 total sites referencing the old variant name:
- `src/runtime.rs`: 14 sites (variant definition, Eq arms (2), Hash arm comments + arm, type_name arm, eval_type doc + arm, render_value comment + arm)
- `src/edn_shim.rs`: 2 sites (render comment + arm)
- `src/closure_extract.rs`: 2 sites (comment + arm)

All update mechanically: `wat_record` → `wat__record`. Three doc-comment "Stone 234.1" references update to "Stone 234.1.5 (rename)" or stay (historical context — keep "Arc 234 Stone 234.1" comments naming the original introduction; that history is honest).

### D9 — Stone 234.1's probe regression guard updates

`tests/probe_arc234_stone1_wat_record_variant.rs` constructs `Value::wat_record { ... }` via the `make_record` helper. β.i updates the helper to construct `Value::wat__record { ... }`. The 7 contracts remain semantically identical (only the variant name changes); probe stays GREEN as the regression guard for the renamed variant.

NOTE: the probe filename stays `probe_arc234_stone1_wat_record_variant.rs` (historical lineage — Stone 234.1 minted the variant; β.i renames it). Renaming the file would erase the lineage. The TEST NAMES inside (probe_1_variant_compiles, etc.) reference `wat_record` in comments — those update to `wat__record` per honesty.

### D10 — HARD CUT — no aliases

`Value::wat_record` name disappears entirely. No back-compat alias, no Deref shim, nothing. Per HARD CUT precedent (arc 227 Stone 227.1b defclass→defrecord, arc 109 surface retirements). The variant existed for exactly two commits (`5abf714` → β.i); no consumers exist at the wat surface.

---

## Implementation surface

**`src/runtime.rs` (14 sites):**

1. Line 651: `wat_record { ... }` → `wat__record { ... }` (variant definition)
2. Line 813: comment `wat_record: identity` → `wat__record: identity`
3. Lines 816-817: Eq arm pattern `Value::wat_record { ... }` → `Value::wat__record { ... }` (both halves of the pair)
4. Lines 1026-1027: Hash arm comments
5. Line 1029: Hash arm pattern `Value::wat_record { ... }` → `Value::wat__record { ... }`
6. Line 1030: discriminant tag `"wat_record"` → `"wat__record"`
7. Line 1177: type_name arm `Value::wat_record { .. } => "wat::core::wat_record"` → `Value::wat__record { .. } => "wat::record"`
8. Line 14460: eval_type doc-comment
9. Line 14485: eval_type arm pattern
10. Line 18216: render_value comment
11. Line 18218: render_value arm pattern

**`src/edn_shim.rs` (2 sites):**

12. Line 1694: render comment
13. Line 1697: render arm pattern

**`src/closure_extract.rs` (2 sites):**

14. Line 1716: comment
15. Lines 1718-1719: comment + arm pattern

**`src/check.rs` (1 site — NEW):**

16. New TypeDef registration for `:wat::record` as opaque primitive type. Mirror existing primitive type registration pattern (e.g., how `:wat::core::String` or `:wat::holon::HolonAST` are registered).

**`tests/probe_arc234_stone1_wat_record_variant.rs` (~7 sites — helper + 7 test fns):**

17. Helper `make_record` constructs `Value::wat__record { ... }` (variant pattern updated)
18. Test fn bodies: destructure patterns `Value::wat__record { ... }` (variant pattern updated; semantics unchanged)
19. Doc-comment headers: `wat_record variant` → `wat__record variant` (honesty in test docs)

---

## FM 2-bis probe plan

The probe at `tests/probe_arc234_stone15_namespace_promotion.rs` verifies BOTH the rename + the namespace registration. Initial state: FAIL (compile-error on `Value::wat__record` — variant doesn't exist yet; type-check error on `[v <- :wat::record]` — type not registered yet). Post-stone: PASS.

5 probe contracts:

1. **Variant compile-pass** — `Value::wat__record { ... }` constructible via Rust helper (verifies the renamed variant exists)
2. **type_name() returns `"wat::record"`** — verifies D2 + D5 in lockstep (the type FQDN the variant returns matches the umbrella registered in check.rs)
3. **Eq + Hash consistency under rename** — two same-args wat__records compare equal AND hash-equal (regression guard against Stone 221.5 invariants surviving the rename)
4. **Type registration accepts `[v <- :wat::record]`** — wat source declaring this type annotation parses + type-checks cleanly (verifies the check.rs TypeDef registration)
5. **`(:wat::core::type <wat__record-instance>)` returns class_fqdn** — Stone 234.0 + 234.1 integration end-to-end preserved through rename (regression guard combining Stone 234.0's polymorphic type primitive + Stone 234.1.5's renamed variant)

Initial FAIL signal: 5/5 with compile-error or UnknownType.
Post-stone PASS: 5/5.

---

## Substrate-as-teacher cascade

Predicted shallow (~18 sites — 16 mechanical pattern updates + 1 type registration + 1 probe-helper update). The cascade is "find every `Value::wat_record` and replace with `Value::wat__record`" — mechanical sed/replace plus the 1 string literal at line 1030 (`"wat_record"` discriminant) and 1 string literal at line 1177 (`"wat::core::wat_record"` type_name) and 1 string in eval_type doc + 1 in render_value comment.

The grep `grep -nE "Value::wat_record|\bwat_record\b" src/ tests/` is the canonical site enumerator. After the rename, the same grep should return zero hits except in historical doc-comments referencing "Stone 234.1" which can stay (history is honest).

---

## Trap-door audit (per FM 2-bis BRIEF discipline)

1. **Probe regression guard** — Stone 234.1's probe at `tests/probe_arc234_stone1_wat_record_variant.rs` is a regression guard that MUST stay GREEN through the rename. β.i sonnet updates the helper + 7 test bodies; if any contract fails post-rename, that's a behavior change masquerading as a rename, STOP.

2. **String literal updates carry semantic weight** — line 1030 (`"wat_record"` Hash discriminant) is NOT cosmetic; it prevents cross-variant collisions. If updated to `"wat__record"` consistently, no collision risk. If left as `"wat_record"`, the variant + tag-string disagree which is dishonest but functionally OK (no collision unless some OTHER variant also tags `"wat_record"` which is unlikely). Sonnet updates the tag for honesty.

3. **type_name() string update is observable** — line 1177's return value is what arc 234 Stone 234.0's `eval_type` fallback uses + what `type_name()` consumers observe. Updating from `"wat::core::wat_record"` to `"wat::record"` is the load-bearing string change. Any wat-level test asserting on `"wat::core::wat_record"` would break — empirically check.

4. **check.rs TypeDef registration mechanism** — investigate the existing pattern (e.g., how `:wat::holon::HolonAST` is registered as an opaque primitive type). Likely a simple `register_type_alias` or similar; sonnet investigates + mirrors before authoring.

5. **`#[wat_value]` seal verification** — Stone 234.1's variant passed the seal (container with three Arc'd fields). The rename doesn't change the shape; seal stays passive.

6. **Cascade scope creep** — sonnet should ONLY update the 18 known sites + register the umbrella type + update the Stone 234.1 probe. If cargo surfaces a site needing OTHER changes (e.g., a wat-level test fixture asserting on `"wat::core::wat_record"`), surface as honest delta. Do NOT silently expand scope.

7. **Stone 234.2a's in-flight artifacts at `db39ebd` + `7113c51`** — these reference `wat-record` and `:wat::core::wat-record/of`. β.i does NOT touch them; they're β.ii orchestrator paperwork to revise after β.i ships. If sonnet sees them and is confused, IGNORE — they're working artifacts under revision, not active substrate.

---

## Risks

- **Cascade depth larger than 18** — unlikely given grep exhaustively enumerated, but possible if any embedded wat string in src/check.rs or test fixtures references the old type name. Mitigation: substrate-as-teacher cascade absorbs.
- **check.rs TypeDef registration mechanism differs from primitive types** — Stone 234.2a's BRIEF had this same trap-door audit item; needs empirical investigation. Mitigation: sonnet reads existing pattern before authoring.
- **Hash discriminant rename breaks an existing test that's hashing the discriminant string somehow** — extremely unlikely (discriminant tagging is internal to Hash impl), but if so the test would surface immediately. Mitigation: full lib test re-run is row 4 of EXPECTATIONS.

---

## Out-of-scope (explicit)

- New substrate primitives (`:wat::record::of`, `:wat::record::field-at`, `:wat::record::def`, `:wat::record::is?`, `:wat::record::to-map`) — Stone 234.2a (β.iii)
- defrecord macro — Stone 234.2b
- Per-class type registration (`:myapp::Voltage` as alias of `:wat::record`) — Stone 234.2b
- Polymorphic verb extensions — Stone 234.3
- `:wat::holon::to-holon` wat__record arm extension — later stone (probably 234.3 or earlier)
- Stone 234.2a in-flight artifacts revision (`db39ebd` + `7113c51`) — β.ii orchestrator paperwork after β.i ships
- holon-rs — STOP-4
- Parallel API / aliases — HARD CUT per D10

---

## Calibration prediction

**Target band:** 15–30 min Mode A
**Upper bound (STOP-3):** 45 min
**Confidence:** very high — mechanical rename + small type registration + probe update; 18 sites enumerated empirically.

**Rationale:**
- 16 pattern updates in src/ (each ~1 line) = mechanical
- 1 type registration in check.rs (~5-10 lines) = small
- 1 probe helper + test body updates (~10 line changes) = mechanical
- Compile + iterate: ~1-2 rounds (variant rename is a single substrate-as-teacher cascade; cargo names every leftover site)
- SCORE: ~5 min

Stone 234.0 was ~38 min (single new primitive + dispatch + TypeScheme). Stone 234.1.5 is SIMPLER (no new behavior; pure rename + 1 type registration); below band's lower edge is plausible.

---

## STOP triggers (REJECTION criteria)

- **STOP-1:** unexpected compile errors NOT tracing to the variant rename + cascade addressing + type registration
- **STOP-2:** baseline lib tests regress below 827
- **STOP-3:** 45 min elapsed (apply partial-state-grading per [[partial-state-grading]])
- **STOP-4:** holon-rs touched
- **STOP-5:** clippy warnings above 54
- **STOP-6:** scope creep — new primitives, defrecord macro, per-class types, polymorphic verbs, anything beyond the rename + namespace registration
- **STOP-7:** FM 2-bis probe doesn't flip 0/5 → 5/5
- **STOP-8:** Stone 234.1's probe regression guard (probe_arc234_stone1_wat_record_variant.rs) regresses post-rename
- **STOP-9:** Stone 234.0 polymorphic type probe regresses (probe_diagnostic_polymorphic_type)
- **STOP-10:** any arc 233 regression guard regresses
- **STOP-11:** Stone 232.0a typed-entities reflection probe regresses

---

## What this unblocks

- **β.ii** (orchestrator paperwork) — revise the now-superseded Stone 234.2a artifacts (`db39ebd` + `7113c51`) to use `:wat::record::*` shape
- **β.iii** (revised Stone 234.2a) — mint `:wat::record::of` + `:wat::record::field-at` + their TypeSchemes on the settled foundation
- **Stone 234.2b** — defrecord macro consumes settled `:wat::record::*` primitives
- **Arc 234.x cascade** — all subsequent stones operate on settled foundation

---

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN.md` — arc 234 umbrella (β.i lands with a pivot section update)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.1.md` — Stone 234.1's SCORE (variant minted; β.i renames + promotes)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.0.md` — Stone 234.0's SCORE (polymorphic type primitive; β.i preserves)
- `tests/probe_arc234_stone1_wat_record_variant.rs` — Stone 234.1's regression guard (β.i updates variant pattern)
- `tests/probe_diagnostic_polymorphic_type.rs` — Stone 234.0's regression guard (β.i preserves)
- `docs/arc/2026/05/225-atomize-materialize-rename/` — arc 225 bridge-naming family precedent (informs `to-map` decision for β.iii)
- `feedback_partial_state_grading.md` — discipline if STOP-3 fires
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs + scores; sonnet writes substrate
- `feedback_no_broken_commits.md` — green tree on disk discipline (β.i ships green; β.iii unblocks)

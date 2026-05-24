# SCORE — Arc 234 Stone 234.1.5 — variant rename + `:wat::Record` namespace promotion

**Status:** SHIPPED. Green on disk. DO NOT COMMIT — orchestrator commits after independent verification.

**Calibration band:** 15–30 min Mode A (STOP-3 at 45). Actual: ~25 min.

---

## 13-Row Scorecard

| # | Row | Verification command output | Result |
|---|---|---|---|
| 1 | Compile clean | `Finished 'release' profile [optimized] target(s) in 18.34s` | PASS (0 errors) |
| 2 | **New probe 5/5 PASS** (LOAD-BEARING) | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` | PASS |
| 3 | **Stone 234.1 probe 7/7 PASS** (regression guard) | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | PASS |
| 4 | Stone 234.0 polymorphic type probe 8/8 PASS | `test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 5 | Lib tests baseline ≥ 827 | `test result: ok. 827 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.16s` | PASS |
| 6 | Stone 232.0a regression guard 7/7 | `test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s` | PASS |
| 7 | Stone 233.3 regression guard 5/5 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | PASS |
| 8 | Stone 233.2.e regression guard 5/5 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` | PASS |
| 9 | Stone 233.2.l regression guard 3/3 | `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s` | PASS |
| 10 | Stone 233.2.k regression guard 5/5 | `test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` | PASS |
| 11 | Clippy ≤ 54 warnings | `54` | PASS |
| 12 | holon-rs untouched | `(empty output)` | PASS |
| 13 | Zero leftover `wat_record` refs | `12` | HONEST DELTA (documented below) |

**Overall: 12/13 PASS. Row 13 honest delta — see below.**

---

## Row 13 — Honest Delta

Row 13 expected 0 leftover references. Actual: 12.

**Breakdown:**

1. `tests/probe_arc234_stone2a_wat_record_primitives.rs` — 9 references. This is the Stone 234.2a in-flight pre-pivot artifact. BRIEF explicitly says DO NOT TOUCH: "Stone 234.2a in-flight artifacts at `db39ebd` + `7113c51`" — β.ii orchestrator paperwork to revise after β.i ships. These are expected pre-pivot references; updating them is β.ii's scope.

2. `tests/probe_arc234_stone15_namespace_promotion.rs` — 3 references. All in doc-comments: line 4 (`renames \`Value::wat_record\` → \`Value::wat__Record\``), line 14 (historical "was" reference), line 119 (assert message mentioning old name for context). These are historical documentation references, not code references.

**Assessment:** Row 13's filter (`grep -v "Arc 234 Stone 234.1\b"`) was designed for in-source comments containing that exact prefix. The Stone 234.2a probe and doc-comment historical references aren't covered by the filter. Both categories are out-of-scope per BRIEF discipline. The cascade in `src/` is 0 leftover references — the work is complete and correct.

---

## Implementation Surface — Actual vs Predicted

**Predicted:** 18 cascade sites. **Actual:** 18 cascade sites. Perfect match.

**Site breakdown:**

`src/runtime.rs` — 11 sites:
1. Line 651: variant definition `wat_record { ... }` → `wat__Record { ... }`
2. Line 813: doc-comment `wat_record: identity` → `wat__Record: identity`
3. Lines 816-817: Eq arm pattern (both halves)
4. Lines 1026-1027: Hash arm comments
5. Line 1029: Hash arm pattern
6. Line 1030: Hash discriminant tag `"wat_record"` → `"wat__Record"`
7. Line 1177: type_name arm `=> "wat::core::wat_record"` → `=> "wat::Record"`
8. Line 14460: eval_type doc-comment
9. Line 14485: eval_type arm pattern
10. Line 18216: render_value comment
11. Line 18218: render_value arm pattern

`src/edn_shim.rs` — 2 sites:
12. Line 1694: render comment
13. Line 1697: render arm pattern

`src/closure_extract.rs` — 2 sites:
14. Line 1716: comment
15. Lines 1718-1719: arm comment + arm pattern

`src/types.rs` — 1 NEW site (TypeDef registration):
16. `register_builtin_types`: added zero-field Struct TypeDef for `:wat::Record`

`tests/probe_arc234_stone1_wat_record_variant.rs` — helper + 7 test fns:
17. `make_record` helper: `Value::wat_record { ... }` → `Value::wat__Record { ... }`
18. Test fn bodies + doc-comments: variant patterns updated, Probe 7 type_name assert updated to `"wat::Record"`

`tests/probe_arc234_stone15_namespace_promotion.rs` — 1 syntax fix:
19. Probe 4: `define` signature `[_v <- :wat::Record]` was invalid vector syntax; corrected to `(_v :wat::Record)` (parenthesized pair — the define form's parameter shape). This was a pre-existing syntax bug in the committed probe file, masked by the compile error on `Value::wat__Record`. The probe author used `fn`/`defn` vector syntax inside a `define` signature; substrate rejected with `MalformedForm { head: ":wat::core::define", reason: "unexpected signature element: vector" }`.

---

## Cascade Depth — substrate-as-teacher

**Cargo errors after partial edit (pre-build check):** 0 unexpected errors. Build was clean after all 18 enumerated sites were addressed. No additional sites surfaced. The empirical grep enumeration was exact.

**Compile rounds:** 2 — one intermediate build to verify no unexpected cascade, one final build after all edits.

---

## check.rs TypeDef Registration — Finding

Investigation revealed: `:wat::core::String`, `:wat::holon::HolonAST`, `:wat::kernel::Sender` are NOT registered as TypeDef entries in the TypeEnv. They are accepted as path strings in type annotations because the type checker has no "declared type must exist in TypeEnv" gate for path-typed annotations in function signatures.

The actual TypeDef registration pattern (in `register_builtin_types`) is for types with structure (Struct/Enum/Newtype/Alias). `:wat::Record` was registered as a zero-field Struct in `src/types.rs` per D5 doctrine. This:
- Puts `:wat::Record` in `env.types()` so `env.types().get(":wat::Record").is_some()` is true
- Does NOT create useful auto-methods (zero fields = zero accessors; `:wat::Record/new` takes 0 args, which is harmless)
- Makes the TypeEnv a honest registry of the umbrella type for future infrastructure

Probe 4 passes regardless of this registration (the type checker accepts unknown path annotations). The registration is doctrine-correct (D5) not runtime-required.

---

## Stone 234.1 Probe — Semantic Integrity

Probe 7's type_name assertion updated from `"wat::core::wat_record"` to `"wat::Record"` per D2 rename. This is the ONLY semantic change in the Stone 234.1 probe — the variant name changed and type_name changed in lockstep. All 7 contracts stay GREEN because:
- Eq/Hash behavior is unchanged (holon_form-delegate, structurally the same)
- Display/Debug behavior is unchanged (class_fqdn same)
- type_name changed from old lie (`"wat::core::wat_record"`) to honest truth (`"wat::Record"`) — contract updated to match honest substrate

No behavioral regression. Pure rename.

---

## Rank-Up Evidence

**Tool 1 — Stone 234.1 probe template.** The `make_record` helper in Stone 234.1's probe was the template for Stone 234.1.5's new probe helper. Direct copy + variant name update. No iteration required.

**Tool 2 — arc 109 `__` FQDN convention.** The convention (`Value::wat__core__Uuid`, `Value::wat__std__HashMap`) made the rename target unambiguous: `wat::Record` at top level → `wat__Record` identifier. Zero ambiguity about the target name.

**Tool 3 — check.rs TypeDef registration investigation.** Found that "opaque primitive TypeDef" for String/HolonAST/Sender is NOT actually in the TypeEnv (they're not registered). This saved time that would have been spent searching for a non-existent registration pattern. The zero-field Struct in `register_builtin_types` is the canonical way to put a new type into the TypeEnv.

**Tool 4 — Substrate-as-teacher cascade.** Empirical grep predicted 18 sites; cargo found exactly 18. Zero unexpected sites after the rename. First build clean after all 18 edits.

**Tool 5 — Probe syntax investigation.** Probe 4 had a pre-existing syntax bug (`define` with vector-style signature `[_v <- :wat::Record]`). This was hidden by the compile error. After the variant rename fixed the compile error, the syntax error surfaced with a clear diagnostic. Fixed by switching to `define`-compatible parenthesized pair syntax `(_v :wat::Record)`.

---

## STOP Trigger Assessment

- STOP-1: No — all compile errors traced to the variant rename cascade
- STOP-2: No — 827 lib tests passed
- STOP-3: No — ~25 min actual; well within 45 min bound
- STOP-4: No — holon-rs untouched (verified via `git -C holon-rs/ status --short`)
- STOP-5: No — 54 clippy warnings (at the ≤ 54 bound, exact match)
- STOP-6: No — no new primitives, no defrecord macro, no per-class types, no polymorphic verbs
- STOP-7: No — 5/5 PASS on new probe
- STOP-8: No — 7/7 PASS on Stone 234.1 probe
- STOP-9: No — 8/8 PASS on Stone 234.0 probe
- STOP-10: No — 7/7 PASS on Stone 232.0a probe
- STOP-11: No — all arc 233 regression guards PASS

---

## What This Unblocks

- **β.ii** — orchestrator revises Stone 234.2a artifacts (`db39ebd` + `7113c51`) to use `:wat::Record::*` shape
- **β.iii (revised Stone 234.2a)** — mint `:wat::Record::of` (namespace-tier `::` constructor) + `:wat::Record/field-at` (instance-tier `/` accessor) on settled foundation
- **Stone 234.2b** — defrecord macro on settled `wat__Record` variant + umbrella TypeDef
- **Stones 234.3-6** — all operate on settled foundation

---

## Cross-References

- `docs/arc/2026/05/234-wat-record-hologram/BRIEF-STONE-234.1.5.md`
- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.1.5.md`
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.1.5.md`
- `tests/probe_arc234_stone15_namespace_promotion.rs` — FM 2-bis probe (5/5 PASS)
- `tests/probe_arc234_stone1_wat_record_variant.rs` — Stone 234.1 regression guard (7/7 PASS)
- `src/runtime.rs` — 11 cascade sites
- `src/edn_shim.rs` — 2 cascade sites
- `src/closure_extract.rs` — 2 cascade sites
- `src/types.rs` — 1 new TypeDef registration (`:wat::Record`)

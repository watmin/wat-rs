# Stone 237.1 sub-DESIGN — typeunion substrate mint

**Status:** PENDING (sub-DESIGN authored 2026-05-25 late-late; FM 2-bis probe + BRIEF + EXPECTATIONS pending).

**Scope:** Mint `:wat::core::typeunion` as a substrate type-declaration primitive. NEW `TypeDef::Union` variant + `UnionDef` struct + registration + cycle detection + member type-checking + **bounded-existential unification extension**. The foundation stone of arc 237.

**Why this stone first:** Stone 237.2 (defclause) depends on typeunion for its variadic-rest typing (`[& rest <- :Vector<:TypeunionName>]`). Without typeunion, defclause cannot express the load-bearing acceptance probe case `(:wat::core::+ 0 1.5 2 3.14 5)`. Per stepping-stone discipline + umbrella DESIGN scope: typeunion goes FIRST.

**Out-of-scope (later arc 237 stones):**
- `:wat::core::defclause` primitive (Stone 237.2)
- defclause + typeunion integration (Stone 237.2-237.5)
- arc 146 Dispatch migration (Stone 237.6)
- arithmetic special-case retirement (Stone 237.7)
- AnyBanned message update (Stone 237.8)
- INSCRIPTION (Stone 237.9)

---

## Locked decisions

### Form syntax

```wat
(:wat::core::typeunion :Name [:T1 :T2 :T3 ...])
```

- `:Name` — user-coined keyword for the union (Pascal-Case capitalization per `feedback_wat_keyword_whitespace` doctrine for type names; e.g., `:Numeric`, `:Comparable`)
- Members are a **Vector literal** `[...]` of TypeExpr keywords — `Path`, `Parametric`, or `Tuple`. Per `feedback_clojure_not_scheme`: Vector literal signals "data/collection," consistent with defclause's `[x <- :T]` arg-binding shape.
- Parametric typeunions (e.g., `typeunion :Result<T,E> ...`) are OUT-OF-SCOPE arc 237; mint when use case surfaces. Stone 237.1 ships non-parametric only.

### Fractal composition (locked example)

```wat
(:wat::core::typeunion :Foo [:wat::core::i64 :wat::core::f64])
(:wat::core::typeunion :Baz [:Foo :wat::core::bool])
;;   :Baz members resolve transitively to {:i64, :f64, :bool}
;;   at type-check time via expand-on-use walk
```

typeunions can reference other typeunions; resolution at type-check time walks the member graph bounded by registration-time cycle check.

### Substrate registration shape

```rust
// New struct (parallels AliasDef)
pub struct UnionDef {
    pub name: String,
    pub type_params: Vec<String>,  // empty in arc 237; reserved for future parametric typeunions
    pub members: Vec<TypeExpr>,
}

// Add variant to existing TypeDef enum
pub enum TypeDef {
    Struct(StructDef),
    Enum(EnumDef),
    Newtype(NewtypeDef),
    Alias(AliasDef),
    Union(UnionDef),   // NEW (Stone 237.1)
}
```

TypeExpr stays at 5 variants. typeunion references at use sites are `TypeExpr::Path(":Name")` that resolve via `TypeEnv` lookup → `TypeDef::Union`. Parallels `TypeDef::Alias` registration model EXACTLY.

### Cycle detection (at REGISTRATION time)

Per `CyclicAlias` precedent at `src/types.rs:1406`: mint `TypeError::CyclicUnion { name, span }`. Detected during registration by walking the member graph; cycle through any registered union name → reject.

### Member type-check (at REGISTRATION time)

| Member shape | Verdict | Reason |
|---|---|---|
| `TypeExpr::Path` (concrete or another typeunion or alias) | ACCEPT | Concrete + recursive-but-cycle-checked + alias-expanded-on-use are all sound |
| `TypeExpr::Parametric` (e.g., `:Vector<:i64>`) | ACCEPT | Bounded structural shape; sound |
| `TypeExpr::Tuple` (e.g., `:(T,U)`) | ACCEPT | Bounded structural shape; sound |
| `TypeExpr::Fn` | REJECT | `InvalidUnionMember`. Weird dispatch semantics; revisit if use case surfaces |
| `TypeExpr::Var` | REJECT | `InvalidUnionMember`. Synthetic; should never appear in user-written declarations |

### Rejections at declaration time

- **Empty members** — `(typeunion :Empty ())` → `EmptyUnion` error (use case unclear; mirror `:Any` rejection rationale)
- **Single member** — `(typeunion :Foo (:i64))` → `SingleMemberUnion` error with diagnostic recommending `:wat::core::typealias` (one canonical path per `feedback_wat_llm_first_design`)
- **Cyclic** — `CyclicUnion` per above
- **Invalid member shape** — `InvalidUnionMember` per above

### Unifier extension — bounded existential typing

The new `unify` arm (added to `src/check.rs:13953`):

**Conceptual rules:**
- `unify(:Numeric, :i64)` where `:Numeric = typeunion (:i64 :f64)` → SUCCEED; resolved type = `:i64`
- `unify(:Numeric, :String)` → FAIL (`UnifyError`)
- `unify(:Numeric, :Numeric)` → SUCCEED; resolved type = `:Numeric` (identity)
- `unify(:Numeric, :Other)` where `:Other = typeunion (:f64 :String)` → SUCCEED only if member sets INTERSECT; resolved type = intersection union (or singleton if size 1)
- `unify(:Numeric, :i64)` then later `unify(:Numeric, :f64)` in the SAME unification context — must FAIL the second (already resolved to :i64)

**Insertion point:** the `reduce` function (called at start of `unify`) is the natural place. When reduce encounters a Path that resolves to TypeDef::Union, the result is wrapped in a "union reference" that the unify-on-children logic special-cases.

**Substitution semantics:** typeunion is NOT a `Var` (it has no fresh-binding semantics). It's a STRUCTURAL type that constrains acceptable matches. The resolved member becomes the type for downstream inference.

**Performance:** member-set check is O(|members|) per unify-arm invocation. For small typeunions (Numeric = 2 members) negligible. For larger ones (future), still O(N) linear scan per unification step. Acceptable.

**Symmetric behavior:** unify is symmetric — `unify(:i64, :Numeric)` must succeed identically to `unify(:Numeric, :i64)`.

### Diagnostics — new TypeError variants

```rust
pub enum TypeError {
    // ... existing variants ...

    // Stone 237.1 — typeunion declaration errors
    CyclicUnion { name: String, span: Span },
    EmptyUnion { name: String, span: Span },
    SingleMemberUnion { name: String, span: Span },  // diagnostic recommends typealias
    InvalidUnionMember { union_name: String, member_form: String, reason: String, span: Span },
}
```

Per arc 138 + 233.3 discipline: every error carries `span`; messages teach (per `docs/SUBSTRATE-AS-TEACHER.md`).

### Evaluator (runtime)

**NONE.** typeunion is type-only; no runtime artifact. The substrate evaluator (`eval_inner` family in `src/runtime.rs`) does NOT need a new arm for `:wat::core::typeunion` at the value layer — it's parsed and consumed by the type-checker only. Stone 237.1's runtime impact is ZERO new value variants + ZERO new eval dispatch arms.

(Aside: at *declaration time*, the typeunion form must be PARSED + REGISTERED via the type-environment loading pipeline. That's check-layer plumbing, not eval-layer. Stone 237.1 wires the parse → register path; the runtime is unaffected.)

---

## Substrate work breakdown

| # | File | Work | Lines (estimate) |
|---|---|---|---|
| 1 | `src/types.rs` | Add `UnionDef` struct + `TypeDef::Union` variant + accessor in TypeDef::name + Display impl | ~30 |
| 2 | `src/types.rs` | Add `CyclicUnion` / `EmptyUnion` / `SingleMemberUnion` / `InvalidUnionMember` to TypeError + Display + span accessor | ~30 |
| 3 | `src/types.rs` | Parser: parse_typeunion (mirrors parse_typealias); wire into `parse_decl_form` dispatch table | ~40 |
| 4 | `src/types.rs` | Registration + cycle detection (`register_union` parallel to `register_alias`; walk_union_cycle helper) | ~50 |
| 5 | `src/types.rs` | Member validation (reject Fn/Var; check at registration) | ~20 |
| 6 | `src/check.rs` | Extend `reduce` to surface typeunion-resolution intent (when path resolves to TypeDef::Union, mark as union-reference) | ~20 |
| 7 | `src/check.rs` | Extend `unify` with typeunion arms (Union/Concrete, Concrete/Union, Union/Union with intersection) | ~80 |
| 8 | `src/check.rs` | Substitution behavior — resolved member updates `subst` so downstream inference sees concrete type | ~20 |

**Total: ~290 lines of substrate work.** Mid-size stone. Calibration 60-120 min Mode A; 240 STOP.

---

## FM 2-bis probe — pre-stone authoring

**File:** `tests/probe_arc237_stone1_typeunion_substrate.rs`

**Rust probe (parallels Stone 236.0's Rust probe):** tests the substrate-internal shape at the Rust API layer, not the wat surface. Verifies TypeDef::Union registration + cycle detection + unify behavior IN RUST UNIT TESTS before sonnet writes the surface plumbing.

**Probe contracts:**

```rust
// Probe 1 — TypeDef::Union variant exists + TypeEnv registers + reads back
#[test] fn union_def_registers_in_type_env() { ... }

// Probe 2 — Cycle detection at registration
#[test] fn cyclic_union_rejected_at_registration() { ... }

// Probe 3 — Empty members rejected
#[test] fn empty_union_rejected_at_registration() { ... }

// Probe 4 — Single-member rejected; diagnostic recommends typealias
#[test] fn single_member_union_rejected_with_typealias_hint() { ... }

// Probe 5 — Fn member rejected
#[test] fn fn_member_rejected_with_invalid_union_member_error() { ... }

// Probe 6 — Var member rejected
#[test] fn var_member_rejected_with_invalid_union_member_error() { ... }

// Probe 7 — Path member (concrete type) accepted
#[test] fn path_member_accepted() { ... }

// Probe 8 — Parametric member accepted
#[test] fn parametric_member_accepted() { ... }

// Probe 9 — Tuple member accepted
#[test] fn tuple_member_accepted() { ... }

// Probe 10 — Recursive union (typeunion-of-typeunions) accepted with cycle check
#[test] fn typeunion_of_typeunions_accepted_when_acyclic() { ... }

// Probe 11 — unify(Union, ConcreteMember) succeeds; subst records concrete
#[test] fn unify_union_with_member_concrete_succeeds() { ... }

// Probe 12 — unify(Union, NonMember) fails
#[test] fn unify_union_with_non_member_fails() { ... }

// Probe 13 — unify(Union, Union) intersects member sets
#[test] fn unify_two_unions_intersects_members() { ... }

// Probe 14 — Symmetric unify(Concrete, Union) behaves identically to unify(Union, Concrete)
#[test] fn unify_symmetric_under_union_arg_order() { ... }
```

**14 probes.** Pre-stone: ALL FAIL (typeunion primitive doesn't exist). Post-stone: ALL PASS.

The probe is committed BEFORE the BRIEF per FM 2-bis. BRIEF cites the probe verbatim: "make these 14 tests pass."

---

## Trap-door audit (pre-emption analysis)

Per recovery-doc FM 2-bis + arc 234's pre-emption discipline:

1. **Unifier `reduce` step ordering.** `reduce` currently walks `Var` substitution + alias expansion. typeunion adds a third resolution path. Must ensure typeunion resolution does NOT short-circuit alias expansion or Var-following. ORDER: walk Var → expand_alias → check_union_reference. If both Var and Union: Var wins (synthetic; needs to bind to concrete first).

2. **Substitution semantics.** When `unify(Union, Member)` succeeds, the substitution must record the SPECIFIC member, not the union. Otherwise downstream `unify(union, OtherMember)` would succeed wrongly. The `subst` map needs to handle this: typeunion-typed positions get bound to their resolved member in the same way Vars do, but typeunion is NOT a Var — needs a parallel mechanism or extending `subst`'s entry-shape.

3. **Recursive typeunion expansion.** If `:Comparable = typeunion (:Numeric :String)` and `:Numeric = typeunion (:i64 :f64)`, then `unify(:Comparable, :i64)` must succeed (`:i64` is transitively a member). Cycle-check at registration prevents infinite recursion; resolution at unify time walks the graph bounded by registration-time cycle check.

4. **Parametric interaction (out-of-scope but worth noting).** `Vector<:Numeric>` should accept `Vector<:i64>` and `Vector<:f64>` but NOT `Vector<:String>`. This means Parametric unification must call into the typeunion arm for its type-arg positions. Stone 237.5 will exercise this; Stone 237.1 must NOT regress it.

5. **Display + error rendering.** typeunion-typed positions in errors should render as `:UnionName (members: :T1 | :T2 | ...)` for clarity. NOT just `:UnionName`, which hides what's actually expected. Per arc 233.3 ValueSnapshot + EDN-error doctrine — errors teach.

6. **Doctrine evolution announcement.** Stone 237.1 introduces typeunion BUT the AnyBanned error message still recommends "named enum for closed heterogeneous sets" exclusively. Stone 237.8 updates the message; Stone 237.1 may want to add an INTERIM comment or doc note explaining typeunion's relationship to the still-current named-enum recommendation. (Decision: NOT in Stone 237.1 scope; the doctrine update is structural at 237.8.)

---

## Tests (load-bearing for SCORE)

Per FM 9: SCORE row tests = LOAD-BEARING. Sonnet's verification must independently exercise:

**Substrate Rust probes (14 contracts, per FM 2-bis):**
- All 14 probes from the FM 2-bis probe file ABOVE pass

**Lib tests (must stay GREEN; no regressions):**
- `cargo test --release --lib` 827 PASS / 0 FAIL (pre-stone baseline)
- Post-stone delta: 0 (typeunion does not touch existing primitives)

**Clippy:**
- `cargo clippy --release -- -D warnings --all-targets` count UNCHANGED from baseline 52

**Integration tests (smoke):**
- All `tests/probe_arc234_*` tests pass (no regression to recently-shipped arc 234 substrate)
- All `tests/probe_arc236_*` tests pass (no regression to recently-shipped arc 236 substrate)

---

## Calibration

| | Estimate |
|---|---|
| Predicted cascade rounds | 2-3 (modest; primarily check.rs + types.rs additions) |
| Predicted runtime | **60-120 min Mode A** |
| STOP | **240 min** |
| HARVEST sites | ~5-10 (new error variants + dispatch arms) |
| New CheckError variants | 0 (TypeError variants instead) |
| New TypeError variants | 4 (CyclicUnion / EmptyUnion / SingleMemberUnion / InvalidUnionMember) |
| New TypeDef variants | 1 (Union) |
| New TypeExpr variants | 0 (typeunion refs reuse TypeExpr::Path + TypeEnv lookup) |
| Test rot risk | LOW (additive; no existing primitive touched) |

**Revision rationale:** initial umbrella estimate was 40-90 min Mode A. Revised UP to 60-120 because the unifier extension (bounded existential typing) is genuinely new machinery — not just registration plumbing. arc 234's 234.0 was at 38 min (in band); arc 236's 236.0 was at ~25 min (under band). Stone 237.1 is HEAVIER than either — it touches unification, which is the hot path. The 60-120 band reflects the additional unifier complexity.

---

## Substrate dependencies (all GREEN)

- arc 234 closed at `02f927a4` — Pascal-Case + ::/⁠/ split + auto-dispatch doctrines applied here
- arc 236 closed at `1e24907f` — CheckResult<T> sum-type pattern doesn't apply to Stone 237.1 (TypeError, not CheckError)
- arc 233 closed — ValueSnapshot + Provenance + EDN errors available for TypeError variants
- arc 232.0 closed — `:wat::core::apply` exists (used by downstream Stone 237.5+, not 237.1)
- holon-rs untouched since `530650c` (STOP-4 clean) — Stone 237.1 is wat-rs only

---

## Cross-references

### Within arc 237
- `DESIGN.md` (umbrella) — particularly the "Substrate diagnosis findings" section
- Stone 237.0 — intueri cast that locked the name `typeunion` (task #552; completed)

### Substrate precedents to mirror
- `src/types.rs:1406` — CyclicAlias error pattern
- `src/types.rs:2629` — expand_alias resolution shape
- `src/types.rs:1643-1674` — typealias parsing dispatch
- `src/check.rs:13953` — unify entry point (insertion site for typeunion arms)

### Doctrine
- `project_typed_entities_doctrine` — substrate algebra honesty
- `feedback_no_new_types` — STOP signal; typeunion EARNS its mint via diagnosis-validated structural need
- `feedback_wat_llm_first_design` — one canonical path; brutal honesty
- `feedback_verbose_is_honest` — verbose error messages that teach

### Related (downstream consumers)
- Stone 237.5 — variadic rest with typeunion-typed Vector (consumes Stone 237.1's typeunion + unifier extension)
- Stone 237.7 — `:Numeric` mint as the first user-facing typeunion (consumes Stone 237.1)
- arc 232.1 defprotocol — may benefit from typeunion for protocol-bounded type params (consumer; later arc)
- arc 235 records-with-rich-VSA-encodings — may benefit from typeunion for kind-bounded field types (consumer; later arc)

---

## Next moves (after sub-DESIGN nod)

1. Author `tests/probe_arc237_stone1_typeunion_substrate.rs` — FM 2-bis probe with 14 contracts
2. Commit probe (BEFORE BRIEF per FM 2-bis discipline)
3. Author `BRIEF-STONE-237.1.md` — sonnet brief citing the probe verbatim
4. Author `EXPECTATIONS-STONE-237.1.md` — calibration band + 12-row scorecard
5. Commit BRIEF + EXPECTATIONS
6. Spawn sonnet with `model: "sonnet"` per FM 12; `run_in_background: true`
7. Schedule wakeup at 2× upper-bound (240 min × 2 = 480s)
8. While sonnet runs: orchestrator does non-overlapping work (e.g., draft Stone 237.2 sub-DESIGN)
9. On sonnet return: SCORE + commit + update CLIFFNOTES Currently

---

*The dungeon's first chamber. typeunion is the cornerstone of the polymorphism consolidation.*

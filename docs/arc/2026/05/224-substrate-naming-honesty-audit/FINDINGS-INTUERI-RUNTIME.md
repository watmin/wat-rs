# Intueri Findings — `wat-rs/src/runtime.rs`

**Spell:** intueri (datamancy grimoire)
**Target:** `/home/watmin/work/holon/wat-rs/src/runtime.rs`
**Size:** 28,916 lines, 252 `eval_*` functions, 485 top-level declarations
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-22 (very-late)
**Cast by:** orchestrator (claude-opus-4-7) per `feedback_spells_cast_via_subagent`
**Duration:** ~5.5 min wall-clock (structural pre-scan + prioritized depth + report)

## Spell verdict

**Spark verdict: mumbles, with one confirmed Level 1 lie + a family of stale names.**

The algebra core (`:wat::holon::*` section, lines 14906–16090) is where the spark lives strongest — doc comments cite arc numbers, capture rejected alternatives, explain capacity math + dim-router logic. A reader learns WHY, not just WHAT.

The spark dims in two regions:
1. **Channel infrastructure** (~18141-18411) — `type_name()` fossils leak Rust library internals (`rust::crossbeam_channel::*`) into FIVE user-visible TypeMismatch error messages
2. **`eval_vec_*` / `eval_list_*` naming split** — post-arc-220 (when `List<T>` was minted), the four remaining `eval_list_*` functions that operate on `Vector` create genuine ambiguity

## Level 1 Findings (lies)

### L1-1 — `:wat::holon::Atom` verb is a polymorphic dispatcher disguised as a single name (CONFIRMED)

**File:line:** `src/runtime.rs:13820` (dispatcher: `eval_algebra_atom`) + `13838` (body: `value_to_atom`)

The verb name imports the `HolonAST::Atom` variant name and overloads it across **9 input-type arms**, most of which produce shapes that are NOT `HolonAST::Atom`:

| Input | Output | Is it Atom? |
|---|---|---|
| `Value::i64/f64/bool/String/keyword/Unit/Char` | typed primitive leaves | NO — `HolonAST::I64/F64/...` |
| `Value::wat__core__Uuid` | `Bind(Tag("uuid"), String(hex))` | NO — composite |
| `Value::holon__HolonAST` | `HolonAST::Atom(inner)` | YES — opaque-identity wrap |
| `Value::wat__WatAST` | `watast_to_holon` structural lowering | NO — full HolonAST tree |
| `Value::wat__std__HashSet` | `HolonAST::bundle(...)` | NO — Bundle |
| `Value::Vec/Tuple` | `HolonAST::bundle(positional-Bind pairs)` | NO — Bundle composite |
| `Value::wat__std__HashMap` | `HolonAST::bundle(K-V Bind pairs)` | NO — Bundle composite |

For the vast majority of inputs the verb produces something that is NOT `HolonAST::Atom`. Arc 221 split off `:wat::holon::leaf` (one arm) + `:wat::holon::from-watast` (one arm); the remaining polymorphism stays under the misleading `Atom` name.

**Honest direction:** `:wat::holon::atomize` — "produce the algebraic coordinate for any runtime value." The verb is doing the boundary-crossing UP into the algebra; the name should signal that, not promise a specific HolonAST variant.

### L1-2 — `Value::type_name()` for Sender/Receiver returns retired transport name

**File:lines:** `src/runtime.rs:1105–1106`

```rust
Value::wat__kernel__Sender(_) => "rust::crossbeam_channel::Sender",
Value::wat__kernel__Receiver(_) => "rust::crossbeam_channel::Receiver",
```

The doc comment at lines 1100–1104 explicitly states: *"both tier-1 and tier-2 backed senders/receivers report the same type_name. The wat-level type checker enforces the tier distinction structurally; runtime type_name names the user-visible kind, not the internal transport."*

The doc says "user-visible kind"; the string returns the internal implementation crate name. **Five `expected:` strings across the file leak this fossil into TypeMismatch errors users will read** (lines 18160, 18252, 18320, 18406, 18821).

**Honest direction:** return `"wat::kernel::Sender"` and `"wat::kernel::Receiver"`; update the 5 expected-string call sites to match. The doc already says the right intent; the string values just haven't caught up since arc 170 slice 1c added the `PipeFd` tier-2 transport.

### L1-3 — `holon_item_to_value` error path mislabels `op` (latent)

**File:line:** `src/runtime.rs:13605–13610`

The fallthrough `Err` arm names `op: ":wat::core::atom-value"`, but `holon_item_to_value` is an INTERNAL HELPER called from multiple sites (including recursively during Bundle decoding). Any future verb that uses `holon_item_to_value` outside the `atom-value` path will surface error messages naming the wrong operation.

**Honest direction:** Pass `op: &str` through the helper signature (mirrors `require_holon`, `require_vec` pattern). The current hardcoded string is latent — works today, lies whenever a sibling verb routes through this helper.

## Level 2 Findings (mumbles)

### L2-1 — `require_vec` vs `require_vector` — two names for "require a container from the caller"

**File:lines:** `7842` (`require_vec`) and `16435` (`require_vector`)

Structurally identical helpers for two semantically distinct types (`Value::Vec` = `wat::core::Vector`; `Value::Vector` = `wat::holon::Vector`). The name pair `vec` vs `vector` doesn't surface the tier distinction.

**Proposed:** `require_wat_vec` / `require_holon_vector`, OR `require_core_vector` / `require_algebra_vector`. Tier visible in name.

### L2-2 — `eval_list_*` family operates on Vector (post-arc-220 ambiguity)

**File:lines:** `9944`, `9972`, `10005`, `12509` — `eval_list_zip`, `eval_list_window`, `eval_list_remove_at`, `eval_list_map_with_index`

All four operate on `Value::Vec` via `require_vec`. Arc 220 minted `Value::wat__core__List` (line 634) — the word "list" in these function names is now genuinely ambiguous. Sibling functions use `eval_vec_*` (reverse, range, take, drop) — these four are stragglers from before the type was distinct.

**Proposed:** rename to `eval_vec_zip`, `eval_vec_window`, `eval_vec_remove_at`, `eval_vec_map_with_index` to align with the established `eval_vec_*` convention.

### L2-3 — `eval_list_ctor` builds a Vector

**File:line:** `7776`

Function named `eval_list_ctor` dispatches `:wat::core::Vector` verb (line 7785) and returns `Value::Vec`. After arc 220 minted `Value::wat__core__List`, the name `eval_list_ctor` actively suggests the wrong type.

**Proposed:** `eval_vec_ctor` or `eval_vector_ctor`.

### L2-4 — `eval_config_noise_floor_default_shim` — "shim" suffix is internal commentary in a public name

**File:line:** `18015`

Function is the sole dispatcher for `:wat::config::noise-floor`. The `_default_shim` suffix describes why it exists, not what it does. Internal meta-comment in a public name.

**Proposed:** `eval_config_noise_floor` — shim rationale belongs in a WHY comment, not the function name.

### L2-5 — Duplicate/contradictory doc on `Value::wat__std__HashMap`

**File:lines:** `427–432`

Doc comment contains two unmerged sentences:

```
/// A `:HashMap<K,V>` — Rust std's `HashMap` backing, wrapped for
/// cheap Arc-cloning. Keys are serialized to type-tagged strings
/// at insertion so heterogeneous-K programs don't collide
/// A `:HashMap<K,V>` — Rust std's HashMap natively; stored as
/// `Arc<HashMap<Value, Value>>` using Stone 216.5a's ...
```

First three lines describe the OLD design (canonical-key crutch); rest describes the NEW design (Stone 216.5a native hash). Old fragment is now a lie — keys are NOT serialized to strings.

**Proposed:** delete lines 427-429.

### L2-6 — `eval_form_ast` vs `:wat::eval-ast!` parse inversion (minor)

**File:line:** `20921`

Function name reads "evaluate a form that is an AST"; verb name reads "evaluate an AST as a form." Pattern is internally consistent with `eval_form_edn` / `eval_form_file` (all `eval_form_*` mean "evaluate code arriving in form X"). Module-level comment explaining the prefix family would resolve the slight double-take.

### L2-7 — `step_form` and `eval_form_step` — naming inversion

**File:lines:** `21060` (`eval_form_step`, public verb dispatcher) and `21311` (`step_form`, internal recursive worker)

Public dispatcher in the `eval_form_*` family pattern, but its semantic is about stepping not source-form evaluation. Parses as "evaluate the form step" rather than "step an evaluation form."

**Proposed:** rename public dispatcher to `eval_step_form` — mirrors `:wat::eval-step!` verb structure.

### L2-8 — `value_to_holon` vs `value_to_atom` — same concept, divergent names

**File:line:** `20983`

`value_to_holon` (used by `eval_form_ast` for arc 066 lift) does nearly the primitive arm of `value_to_atom`. Three functions doing overlapping primitive-lift work: `value_to_holon`, `value_to_atom` primitive arm, `eval_holon_leaf`. Names don't signal the relationship; difference (collection inputs accepted vs rejected) is invisible.

**Proposed:** `value_to_holon` → `promote_primitive_to_holon` — narrower scope visible.

## Rune Evaluations

**None.** Zero `// rune:intueri(...)` runes in the file.

## SUBSTRATE VERB-NAME FAMILY PATTERN (the load-bearing finding)

The known lie at `:wat::holon::Atom` belongs to a recognizable family: **verb names that import a HolonAST constructor/node name and then overload it as a polymorphic dispatcher across all input types.**

The family extends to the boundary-inverse: `:wat::core::atom-value`. The name implies "extract the value from an Atom" — but `eval_atom_value`'s `HolonAST::Bundle` arm dispatches across FOUR distinct output shapes (Vec / HashMap / HashSet / three-way discrimination by key shape). A `Bundle` is not an `Atom`; the function also decodes Bundles.

**The HONEST verb-name family pair:**

| Current verb | Proposed honest name | What it actually does |
|---|---|---|
| `:wat::holon::Atom` | `:wat::holon::atomize` | "lift any runtime value to its algebraic coordinate" |
| `:wat::core::atom-value` | `:wat::holon::materialize` | "lower any HolonAST back to a runtime value" |

Both verbs do boundary-crossing — lift UP into algebra, lower DOWN to runtime. The `Atom`/`atom-value` symmetry was born when the only case was primitive leaves. Arc 216 added collection support (Bundles ↔ Vec/HashMap/HashSet); the names stopped being symmetric. A caller doing `(atom-value bundle)` today gets a Vec or HashMap — not an "atom's value."

The `leaf` and `from-watast` verbs split off from `:wat::holon::Atom` in arc 065/221 are the right direction: one verb, one input type, one output behavior. The remaining work is to give the polymorphic "any value" case (`Atom` over collections) and the Bundle decoding case (`atom-value` over Bundles) names that admit their actual scope.

## Spark Assessment

> *"The spark lives strongest in the algebra core (`:wat::holon::*` section, lines 14906–16090). The doc comments there are genuine WHY signals: they cite arc numbers, capture rejected alternatives, explain the design decisions behind Kanerva capacity enforcement, the coincident-floor vs presence-floor distinction, and the dim-router logic. A reader who arrives at `eval_algebra_bundle` learns not just what the code does but why the capacity math works the way it does."*
>
> *"The spark dims in two regions. First, the channel infrastructure (lines 18141–18411): the `type_name()` fossil names (`rust::crossbeam_channel::Sender/Receiver`) bleed through five error messages that users will read. Second, the `eval_vec_*` / `eval_list_*` naming split: the file consistently uses `eval_vec_*` for Vector operations except for four functions (`list_zip`, `list_window`, `list_remove_at`, `list_map_with_index`) that operated on Vectors before arc 220 minted `List<T>`."*
>
> *"The most important finding remains the Level 1 lie at `:wat::holon::Atom`: a verb named after a single HolonAST constructor that actually dispatches across nine input-type arms, most of which produce nodes other than `HolonAST::Atom`. The channel fossil names are a close second — they reach the user as error messages, making the lie visible outside the codebase."*

## Disposition

This cast surfaced the genuine substrate naming flaws that the doctrine dialogue predicted. Per arc 224's Phase 2 plan, these findings drive fix-arc decisioning:

**Highest priority (Level 1):**
1. The `:wat::holon::Atom` / `:wat::core::atom-value` boundary-pair rename (`atomize` / `materialize`) — substrate verb-name honesty
2. `Value::type_name()` channel fossil — leaks to user error messages; quick rename
3. `holon_item_to_value` op-name parameter threading — latent lie

**Medium priority (Level 2):**
4. `eval_list_*` → `eval_vec_*` family rename (5 functions including ctor)
5. `require_vec` vs `require_vector` tier-visible rename
6. Various doc/name cleanup

**Recommendation:** wait for Stone 224.3 (check.rs cast) findings before opening fix-arcs — the type-checker side may surface additional or contradictory naming signals; aggregate first per arc 224 Phase 2 design.

## Cross-references

- arc 224 DESIGN.md — substrate naming honesty audit scope
- FINDINGS-INTUERI-HOLON-AST.md — Stone 224.1 (substrate algebra: 0 L1, 4 L2)
- intueri SKILL.md — `~/work/holon/datamancy/intueri/SKILL.md`
- arc 221 DESIGN.md — substrate-doctrine work that surfaced this audit
- `feedback_inscription_immutable` — findings stay as historical record even after fixes ship

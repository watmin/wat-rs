# Intueri Findings — `wat-rs/src/check.rs`

**Spell:** intueri (datamancy grimoire)
**Target:** `/home/watmin/work/holon/wat-rs/src/check.rs`
**Size:** 18,361 lines (largest file in the substrate)
**Agent:** claude-sonnet-4-6
**Date:** 2026-05-23 morning
**Cast by:** orchestrator (claude-opus-4-7) per `feedback_spells_cast_via_subagent`
**Duration:** ~7 min wall-clock

## Spell verdict

**Spark verdict: lives in most of the file, dims at the pre-arc-109 legacy boundary.** The module doc is exemplary; `CheckError` variants carry WHY comments with arc references; inference helpers (`infer_let`, `infer_match`, etc.) are well-named and bounded. The spark dims specifically where the QueueSender/QueuePair vocabulary (retired by arc 109 slice K.kernel-channel) was renamed in code but NOT in adjacent doc comments — those docs now actively lie about what the code does.

## Level 1 Findings (lies)

### L1-A — `type_contains_sender_kind` doc says "QueueSender"; code detects "Sender"/"Channel"/"crossbeam::Sender"

**File:lines:** `check.rs:3675–3699` (doc), `3706–3718` (body)

Doc header: *"Does this type … contain a `:wat::kernel::QueueSender<T>` ANYWHERE in its structure? Returns 'QueueSender' on hit."* Plus rationale paragraph saying *"Why QueueSender (and not also bare Sender, HandlePool): …"*

Code body matches `"wat::kernel::Channel"`, `"wat::kernel::Sender"`, `"rust::crossbeam_channel::Sender"`. Never matches `QueueSender`. The QueueSender vocabulary was retired by arc 109 slice K.kernel-channel; the migration walker is even in this same file (line 3241). Body updated; doc stayed.

**Active lie.** Reader follows the doc, looks for QueueSender in the type algebra, finds nothing, concludes the check is broken.

**Honest direction:** rename to `type_bears_sender` or `sender_kind_in_type`; rewrite doc to: *"Detects whether a type structurally contains a Sender-bearing parametric (`Channel<T>`, `Sender<T>`, `rust::crossbeam_channel::Sender<T>`, `HandlePool<T>` where T is Sender-bearing)."* Retire the QueueSender rationale section entirely.

### L1-B — `ScopeDeadlock` CheckError variant doc references retired QueueSender + QueuePair

**File:line:** `check.rs:143`

Variant doc says the sibling binding's type *"contains a Sender-bearing parametric (Sender, QueueSender, QueuePair, HandlePool)."* QueueSender and QueuePair are retired (arc 109 K.kernel-channel). The actual detector at line 3706 checks `Channel`, `Sender`, `rust::crossbeam_channel::Sender`, `HandlePool` — not QueueSender or QueuePair.

**Honest direction:** update variant doc to canonical names: `Sender`, `Channel`, `HandlePool`. Drop QueueSender + QueuePair.

### L1-C — `symbol_ty` local closure names a keyword type "symbol"

**File:line:** `check.rs:15624`

```rust
let symbol_ty = || TypeExpr::Path(":wat::core::keyword".into());
```

Adjacent comment hedges: *"takes a :Symbol (keyword name)."* The author knew the type was `:keyword` but kept the closure name `symbol_ty`. Four TypeScheme registrations (lines 15633, 15642, 15651, 15700) carry this mislabeled type. In wat, `:wat::core::keyword` and symbols are distinct types — the closure name lies.

**Honest direction:** rename to `keyword_ty` (mirrors `holon_ty`, `bool_ty`, `i64_ty`, `f64_ty` family). Update the adjacent comment.

## Level 2 Findings (mumbles)

### L2-A — `infer_list_constructor` doesn't name what it constructs (Vector, not List)

**File:line:** `check.rs:11934` — handles `:wat::core::vec`, `:wat::core::list`, `:wat::core::Vector` (post-arc-220 the latter is canonical); body returns `Parametric { head: "wat::core::Vector", ... }`. No doc comment.

**Proposed:** `infer_vector_constructor` + one-line doc.

### L2-B — Four `#[allow(dead_code)]` retired walkers without intueri runes

**File:lines:** `2629–2631`, `3424–3425`, `3504–3505`, `3564–3565`

Each is documented as "kept as reference" / "dead code intentional." Without `// rune:intueri(comment-style)` runes, each cast must re-verify the intent.

**Proposed:** add proper intueri runes citing the arc 113 precedent for retained-but-dead historical references.

### L2-C — Retired-walker block comments in `check_program` are absence-documentation

**File:lines:** `1895–1900`, `1901–1915`, `1917–1930`

Three block comments document walkers that no longer exist with detailed pre-arc-133/154/155 context. Reader scanning `check_program` must parse all three to discover no code is there.

**Proposed:** trim to one sentence per retired walker; leave the WHY in DESIGN docs.

### L2-D — `arc_114_migration_hint` actually checks TWO shapes

**File:line:** `check.rs:942`

Function name says "arc 114 migration hint." Body detects (a) bare-spawn callees AND (b) ProgramHandle↔Thread type-annotation leftovers (line 954). Two distinct predicates fused into one function under one arc-number name.

**Proposed:** split, or expand the doc to name both triggers.

### L2-E — `dispatchs` used as a count noun + local variable name

**File:lines:** `1615`, `1675`, `4927`, `16553`, `16557`, `16561`, `16580`

Not a word. At line 16553 it's a local variable: `let mut dispatchs = crate::dispatch::DispatchRegistry::new()`. Domain concept is "dispatch declarations" or "dispatch registrations."

**Proposed:** in comments use "dispatch declarations" or "registered dispatches"; for the local var use `dispatch_reg` / `dispatch_registry`.

### L2-F — `type_is_thread_kind` (bool) vs `type_contains_sender_kind` (Option<&str>) naming asymmetry

**File:lines:** `3589`, `3700`

One says "is" (returns bool); the other says "contains" but returns the KIND on hit (Option, not bool). Both used adjacently in scope-deadlock checks.

**Proposed:** rename the Option-returner to `sender_kind_in_type` or `find_sender_kind` (Rust convention: `find_` prefix for Option-returning search).

### L2-G — Arc 117 dead-code section header references retired QueuePair vocabulary

**File:lines:** `check.rs:2598–2622`

Section header `// ─── Arc 117 — scope-deadlock prevention ───` then describes the pre-arc-133 algorithm: *"At every `:wat::kernel::spawn-thread` call whose body fn closure-captures a Receiver from a sibling `:wat::kernel::QueuePair`..."* — uses retired QueuePair name AND describes a retired algorithm as if it's live.

**Proposed:** update section header to *"Arc 117 dead-code cluster — retired by arc 133; see `check_let_for_scope_deadlock_inferred` in `infer_let` for the live rule."* Strip the QueuePair walkthrough.

## Rune evaluations

**None.** Zero `// rune:intueri(...)` runes in check.rs. The four `#[allow(dead_code)]` + doc-comment pairs function as informal runes but lack the required reason field + category.

## Cross-file convergence with runtime.rs cast

**`:wat::holon::Atom` TypeScheme is HONEST at check.rs layer:**
- Registered scheme (line 13558): `∀T. T → HolonAST` — makes no claim about the shape
- Special-case in `infer_list` (line 5326): routes `:wat::holon::Atom | :wat::holon::leaf` through `is_atomizable(T)` as VALUE-DOMAIN constraint (not type-unification)
- `is_atomizable` (line 3623) doc names all four allowed categories, conservatism documented

**The runtime.rs lie does NOT amplify into check.rs.** The check layer handles the polymorphic verb structurally soundly. The lie lives only in the runtime body where the verb name (`Atom`) implies a specific output that doesn't match the actual dispatch.

**`:wat::core::atom-value` similarly honest at check.rs layer:**
- Scheme: `∀T. HolonAST → T`
- Handler returns fresh type variable per "the inferred T from the holon's shape" comment
- The runtime decoding (Vec/HashMap/HashSet/three-way) happens BELOW the type-check; can't be statically resolved

**NEW FINDING — `:wat::runtime::rename-callable-name` TypeScheme is DECORATIVE:**
- Scheme registration (lines 15697–15703): claims `params: vec![holon_ty(), symbol_ty(), symbol_ty()]` — three typed parameters
- Special-case handler in `infer_list` (line 5260): *"We infer all args for side-effects but do not enforce type constraints — the runtime does its own validation."*
- The handler calls `infer(arg, ...)` for each arg but ignores the return type; does NOT unify against the scheme's declared params
- **Reader who sees the scheme expects the checker to validate arg[1] is a keyword; it won't.**

Level 2 mumble, not L1: scheme is in the right direction; special-case arm makes it a dead letter. The comment is honest about the deferral. Stone 221.4b registered this correctly; the bypass is the acknowledged limitation.

## Aggregate observation across the three casts

The substrate-naming-honesty audit shows a layered honesty pattern:

| Layer | File | L1 lies | What lies |
|---|---|---|---|
| **Algebra primitives** | `holon-rs/src/kernel/holon_ast.rs` | 0 | nothing — variant names speak truth |
| **Verb dispatchers** | `wat-rs/src/runtime.rs` | 3 | verb names borrow variant names + overload polymorphically; channel type-name fossils leak to users |
| **Type checker** | `wat-rs/src/check.rs` | 3 | retired-vocabulary doc comments not refreshed when code was renamed; one closure mislabel |

**The substrate algebra is the strongest layer.** It has 16 honest variants. The wat-rs surface that wraps it has lying verb names + stale documentation. The fixes are at the surface layer, not the algebra.

The verb-name family pattern (`Atom`/`atom-value` boundary pair) is the most consequential finding — it's the conceptual cleanup that touches both runtime.rs (the verb body lies) and check.rs (the verb scheme is decorative). The honest pair `atomize`/`materialize` would land at both layers.

## Disposition

Three casts complete. Stone 224.4 (aggregate findings + fix-arc planning) is the next step. Findings categorize as:

**Highest priority (Level 1, user-visible):**
- L1-2 from runtime.rs cast: `type_name()` Sender/Receiver fossils — leaks into 5 user error messages
- L1-A from check.rs cast: `type_contains_sender_kind` doc lie — misdirects deadlock-rule traces

**Medium priority (Level 1, substrate-doctrine):**
- L1-1 from runtime.rs cast: `:wat::holon::Atom` polymorphic dispatch — the boundary-crossing family pattern (proposed `atomize`/`materialize`)
- L1-3 from runtime.rs cast: `holon_item_to_value` op-name parameter threading (latent)
- L1-B + L1-C from check.rs cast: stale variant doc + symbol_ty closure mislabel

**Low priority (Level 2):**
- Multiple `eval_list_*` → `eval_vec_*` renames (runtime.rs)
- Multiple naming-convention cleanups across both files
- Dead-code rune additions (check.rs)

## Cross-references

- arc 224 DESIGN.md — full audit scope
- FINDINGS-INTUERI-HOLON-AST.md — Stone 224.1 (algebra honest, 0 L1, 4 L2)
- FINDINGS-INTUERI-RUNTIME.md — Stone 224.2 (verb layer lies, 3 L1, 8 L2 + family pattern)
- intueri SKILL.md — `~/work/holon/datamancy/intueri/SKILL.md`
- [[atom-is-holder]] memory — the substrate doctrine driving the audit
- INTERSTITIAL § 2026-05-22 very-late → 2026-05-23 — the realization arc

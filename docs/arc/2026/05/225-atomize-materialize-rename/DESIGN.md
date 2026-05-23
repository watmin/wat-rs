# DESIGN — Arc 225 — Bridge naming honesty (narrow Atom + `to-holon` / `from-holon` mint + `to-wat` / `from-wat` rename)

> **SPAWN-BLOCK STATUS (2026-05-22 late, post-bridge-naming-intueri):** Arc 225 spawned by arc 224. Arc 224's INSCRIPTION blocked on arc 225 closing. The chain: arc 220 ← arc 221 ← arc 224 ← arc 225. Arc 225's spawn children: **arc 228** (collection classifier-wrap) → arc 230 → arc 226 → arc 227. Arc 225's INSCRIPTION blocked on arc 228 closing.

> **DOCTRINE EVOLUTION (forward-corrections preserved in chronological order; DESIGNs are living per recovery doc § FM 11):**
>
> **2026-05-23 morning (Option A):** Original proposal `:wat::holon::Atom` → `:wat::holon::atomize` REJECTED. `atomize` is still polymorphic; doesn't always produce HolonAST::Atom. Narrow `:wat::holon::Atom` to single-shape HolonAST → HolonAST::Atom constructor instead.
>
> **2026-05-23 evening (typed-entities doctrine):** Every typed value at user-surface compiles to `(Bind (Atom <ClassName>) (Atom <data>))`. Atom is the substrate holder primitive. Wat-tier and holon-tier coexist as parallel layers.
>
> **2026-05-22 (post-compaction, intueri bridge-naming cast):** The proposed name `materialize` for the wat-verb is honest as an operation-name, BUT user observation surfaced asymmetry: existing `from-watast` / `to-watast` use "watast" but we don't say "holonast" anywhere — we say "holon" as the layer name. The honest family uses layer-names + direction throughout:
>
> ```
> :wat::holon::from-wat    (renamed from from-watast)    WatAST       → HolonAST
> :wat::holon::to-wat      (renamed from to-watast)      HolonAST     → WatAST
> :wat::holon::to-holon    (NEW; replaces UP arm of Atom) runtime Value → HolonAST
> :wat::holon::from-holon  (NEW; replaces atom-value)    HolonAST     → runtime Value
> ```
>
> Plus narrow `:wat::holon::Atom h` constructor stays (Pascal-Case sibling to Bundle/Bind/Permute/Thermometer/Blend/Tag).

**Opened:** 2026-05-22 (post-arc-224 audit aggregate)
**Branch:** `arc-170-gap-j-v5-deadlock-state`
**Depends on:** arc 224 audit complete (Stones 224.1/.2/.3 + AGGREGATE-FINDINGS + FINDINGS-INTUERI-BRIDGE-OPS all shipped).

## Mission — final shape

Resolve the bridge-naming honesty findings from the arc 224 audit + the typed-entities doctrine + the intueri bridge-naming cast. **Substrate-wide rename + mint + narrow**, all under the symmetric layer-name + direction discipline.

The five concrete deliverables:

1. **Narrow `:wat::holon::Atom`** to single-shape constructor
2. **Mint `:wat::holon::to-holon`** (replaces the polymorphic UP arm of the original Atom verb)
3. **Mint `:wat::holon::from-holon`** (replaces `:wat::core::atom-value`)
4. **Rename `:wat::holon::from-watast` → `:wat::holon::from-wat`** (cosmetic; same semantics)
5. **Rename `:wat::holon::to-watast` → `:wat::holon::to-wat`** (cosmetic; same semantics)

Plus the L1-3 doc-lie from intueri bridge-naming cast: fix the (renamed) `eval_holon_from_holon` doc comment to honestly describe the polymorphic decode (currently says "Bundle → error" but body handles three-way Bundle dispatch since arc 216 Stones 1/2/3).

## The doctrine map (final)

```
wat-tier:    (quote      wat-form)  -> :WatAST       ;  hold source unevaluated (wat op)
              (to-wat     holon)    -> :WatAST       ;  lower algebra → source AST
              (from-wat   wat-ast)  -> :HolonAST     ;  lift source AST → algebra

holon-tier:  (Atom       holon)    -> :HolonAST     ;  hold with VSA opaque-identity (holon op)
              (to-holon   value)   -> :HolonAST     ;  lift runtime → algebra
              (from-holon holon)   -> :Value        ;  lower algebra → runtime
```

Five verbs (Atom + 4 bridge ops). Each verb has unique input/output types. Type-system dispatches unambiguously. Symmetric naming: layer + direction. No variant-name borrowing. No AST-suffix asymmetry.

## Spawn-block status (load-bearing)

Arc 225 is the entry point of a 5-arc chain. Arc 225's INSCRIPTION blocked on arc 228 closing. Arc 228's INSCRIPTION blocked on arc 230. Arc 230 on arc 226. Arc 226 on arc 227. The chain implements the typed-entities doctrine progressively across the substrate.

```
arc 225 — bridge naming (THIS ARC; active head)
  └→ arc 228 — substrate collection classifier-wrap
       └→ arc 230 — substrate variant retirement (Symbol/Keyword/Tag/Nil retire)
            └→ arc 226 — type predicates as VSA similarity
                 └→ arc 227 — user-defined types via classifier-wrap
```

Independent / parallel-OK arcs (siblings of arc 224 under arc 221):
- arc 222 — EDN-form named constructors + 3×2 conversion topology
- arc 223 — WatAST primitive-layer honesty
- arc 229 — quasiquote evaluator + splice (deferred per user; small + concrete)

## Stones

### Stone 225.1 — substrate rename + mint (v3, under resolved naming)

Substrate Rust changes in `src/runtime.rs` + `src/check.rs`:

**A. Narrow `:wat::holon::Atom`** (`runtime.rs:13820` / `13838`):
- Accept ONLY `Value::holon__HolonAST` input
- Return `HolonAST::Atom(inner)` opaque-identity wrap
- DELETE all other input-arm branches (primitives, WatAST, collections, Uuid, etc.) — the polymorphic body retires
- Rename Rust fn `value_to_atom` → `wrap_holon_as_atom`
- TypeScheme update in `check.rs:13558` from `∀T. T → HolonAST` to `HolonAST → HolonAST`
- Update `infer_list` special-case at `check.rs:5326`

**B. Mint `:wat::holon::to-holon`** (NEW; absorbs the retired UP arms):
- New verb registration in `runtime.rs` dispatch table
- New Rust fn `eval_holon_to_holon` — accepts `Value` of any type; produces appropriate HolonAST (primitive → leaf; HolonAST → Atom-wrap; WatAST → structural lower; collections → bare Bundle for now — arc 228 extends with classifier-wrap; Uuid → Bind(Tag, String) per existing pattern)
- TypeScheme in `check.rs` — `∀T. T → HolonAST` (polymorphic; operation-name honest)
- Doc-comment explains: lowercase = operation = polymorphism honest per typed-entities doctrine

**C. Mint `:wat::holon::from-holon`** + retire `:wat::core::atom-value`:
- Rename verb: `:wat::core::atom-value` → `:wat::holon::from-holon` (namespace move + rename)
- Rename Rust fn: `eval_atom_value` → `eval_holon_from_holon`
- Body unchanged in semantics — still polymorphic decode (Atom unwrap; Bundle three-way → Vec/HashMap/HashSet; leaves → matching Value primitive)
- **Doc comment refresh** — fix L1-3 from intueri bridge-naming cast: current doc says "Composite (Bundle/...) → error" but body handles Bundle three-way; refresh to honestly describe the polymorphic decode
- Rename Rust helper `holon_item_to_value` → `from_holon_item` + thread `op: &str` parameter (closes arc 224 L1-runtime-3 latent lie)
- TypeScheme in `check.rs:13591` → `:wat::holon::from-holon` string key

**D. Rename `from-watast` → `from-wat`**:
- Verb registration: `:wat::holon::from-watast` → `:wat::holon::from-wat`
- Rust fn: `eval_holon_from_watast` → `eval_holon_from_wat`
- TypeScheme + special-case handler updates
- No semantic change; cosmetic family-consistency rename

**E. Rename `to-watast` → `to-wat`**:
- Verb registration: `:wat::holon::to-watast` → `:wat::holon::to-wat`
- Rust fn: `eval_holon_to_watast` → `eval_holon_to_wat`
- Same as D — cosmetic

**F. Substrate-as-teacher cascade**:
- `cargo build --release -p wat` after Phase A-E sites will emit many errors from old verb names + retired Atom arms
- Iterate per FM 15: read errors, apply rule, rerun until green

**G. Wat-side caller sweep**:
- All `wat/**/*.wat` + `wat-tests/**/*.wat` callers of any of the four old verbs need updating
- Atom polymorphic callers redirect to `to-holon` or to narrow constructors (leaf/Bundle/etc.) per the input type

Per pre-flight grep (orchestrator):
- ~31 Rust call sites for `:wat::holon::Atom` literal
- ~10 Rust call sites for `:wat::core::atom-value` literal
- ~54 wat-side caller sites for either verb
- Plus `:wat::holon::from-watast` / `to-watast` callers (need fresh grep)
- Total estimated: ~150-200 touch points

### Stone 225.2 — INSCRIPTION (after arc 228 closes per spawn-block)

- INSCRIPTION-225.md inscribes the bridge naming family + the verdict on `atomize` (rejected as lie)
- Cross-references arc 224 audit, intueri bridge-naming cast, typed-entities doctrine
- Closes arc 225; unblocks arc 224 INSCRIPTION

## Calibration

| Stone | Scope | Predicted |
|---|---|---|
| 225.1 | substrate rename + mint + cascade + sweep | 180-300 min Mode A |
| 225.2 | INSCRIPTION | 30-60 min orchestrator-direct (blocked on arc 228 closing) |

**225.1 total estimate:** 3-5 hours sonnet. Larger than original arc 225 plan because the resolved naming added 2 more renames + 1 more mint. Worth it for the symmetric honest family.

## What this arc does NOT do

- Touch holon-rs (the algebra primitives stay; arc 230 retires variants later)
- Implement collection classifier-wrap (arc 228's scope)
- Mint type predicates (arc 226)
- Touch the quasiquote evaluator (arc 229; deferred)
- EDN-form named constructors at wat-surface (arc 222)
- WatAST primitive-layer honesty (arc 223)

## Cross-references

- arc 224 FINDINGS-INTUERI-BRIDGE-OPS.md — the cast that resolved the bridge naming
- arc 224 FINDINGS-INTUERI-RUNTIME.md — the audit that surfaced the original Atom verb lie
- arc 224 AGGREGATE-FINDINGS.md — Group B scope (which this arc fulfills)
- arc 228 DESIGN.md — spawn child; collection classifier-wrap built on this arc's clean Atom semantics
- arc 222 DESIGN.md — sibling arc; EDN-form constructors use this arc's bridge verbs
- arc 230 DESIGN.md — descendant; variant retirement built on the typed-entities doctrine
- [[typed-entities-doctrine]] memory — load-bearing doctrine
- [[atom-is-holder]] memory — earlier framing; refreshed under typed-entities
- INTERSTITIAL § 2026-05-23 evening — doctrine landing
- INTERSTITIAL § 2026-05-22 (post-compaction) — bridge naming dialogue + timeline correction
- `feedback_spawn_block_winding` — parentage discipline
- `feedback_inscription_immutable` — doctrine for forward-correcting earlier arcs
- `feedback_substrate_as_teacher` (`docs/SUBSTRATE-AS-TEACHER.md`) — cascade methodology for Stone 225.1

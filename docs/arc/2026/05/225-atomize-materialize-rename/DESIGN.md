# DESIGN — Arc 225 — Narrow `:wat::holon::Atom` to constructor + `:wat::core::atom-value` → `:wat::holon::materialize`

> **SPAWN-BLOCK STATUS (2026-05-23):** Arc 225 is spawned by arc 224. Arc 224's INSCRIPTION (Stone 224.7) blocked on arc 225 closing. The chain: arc 220 ← arc 221 ← arc 224 ← arc 225 (depth 4).

> **DOCTRINE REFINEMENT 2026-05-23:** Original arc 225 DESIGN proposed renaming `:wat::holon::Atom` → `:wat::holon::atomize`. Mid-Stone-225.1 dialogue surfaced that this was still partially dishonest — `atomize` doesn't always return `HolonAST::Atom`. User probe: *"so :wat::holon::{Atom,Bundle,Bind,...} still exist?.. atomize returns an Atom?.."* + *"I would say Atom's sig is (Atom :HolonAST) -> :Atom?"* The honest direction: **narrow `:wat::holon::Atom` to a single-shape constructor matching the sibling Pascal-Case verb family.** Sonnet's in-flight rename work reverted; arc 225 reshaped.

> **FURTHER DOCTRINE CONVERGENCE 2026-05-23 afternoon:** Continued dialogue surfaced the layered honesty framing for the entire quote-family + macro sigils. See INTERSTITIAL § 2026-05-23 afternoon for the full narrative. Key resolution: **substrate stays at 16 HolonAST variants — no expansion needed.** Macro sigils `'` `` ` `` `~` `~@` encode as Bundle-of-verb at source-form layer (consistent shape with other verb applications) AND evaluate to Atom-wrapped substrate forms at evaluated-form layer (consistent "this is held" semantic via Atom). Tag is reserved for EDN tagged literals only (`#name value`); reusing Tag for macro sigils was dishonest. Arc 225's scope is unchanged: narrow `:wat::holon::Atom` + rename `:wat::core::atom-value` → `:wat::holon::materialize`. The EDN-form named constructors and the quasiquote-expansion-evaluator work belong to arc 222.

> **FULL DOCTRINE LANDED 2026-05-23 evening (supersedes afternoon framing):** Through 7 rounds of dialogue, the substrate found itself. See INTERSTITIAL § 2026-05-23 evening + [[typed-entities-doctrine]] memory entry. **Materialize is the substrate's UNQUOTE PRIMITIVE** — pair with Atom (which is quote at substrate-operation level). The rename `:wat::core::atom-value` → `:wat::holon::materialize` is even more load-bearing now: it names a foundational substrate operation, not just a decode convenience. The 12-true-primitives doctrine + the uniform `(Bind (Atom class) (Atom data))` typed-entity shape + the type-as-VSA-similarity insight all rest on Atom/Materialize being the substrate's quote/unquote dual.
>
> **Arc 225's scope remains:** narrow `:wat::holon::Atom` to single-shape constructor + rename `:wat::core::atom-value` → `:wat::holon::materialize`. The doctrine context expands the WEIGHT of the rename without expanding the work.

## Triggering observation

Arc 224 Stone 224.2 (intueri on runtime.rs) found:
- `:wat::holon::Atom` borrows the HolonAST variant name + dispatches polymorphically across 9 input types; most arms produce shapes that are NOT `HolonAST::Atom`
- Sibling Pascal-Case verbs (`:wat::holon::Bundle`, `Bind`, `Permute`, `Thermometer`, `Blend`, `Tag`) are HONEST constructors — each takes one input shape + produces the matching variant

The lie is specifically in `Atom`'s overload + in `:wat::core::atom-value`'s Bundle-decoding path (sibling lie at the inverse direction).

## The doctrine — verb-naming honesty per intueri + the holder framing

Two naming patterns, both legitimate, but used DIFFERENTLY:

**Pattern 1 — Pascal-Case variant-name verbs = CONSTRUCTORS:**
- Each verb takes the variant's natural input shape, produces that variant
- `:wat::holon::Bundle xs` → `HolonAST::Bundle(xs)` — single shape, no polymorphism
- `:wat::holon::Bind a b` → `HolonAST::Bind(a, b)` — same
- etc.
- THE PATTERN matches the variant-name : output-type contract
- **`:wat::holon::Atom h` SHOULD follow this: `(Atom :HolonAST) -> HolonAST::Atom(h)` — single shape, honest**

**Pattern 2 — lowercase action-name verbs = OPERATIONS:**
- Name describes WHAT THE VERB DOES, not the output type
- Output may be polymorphic if the operation is naturally polymorphic
- `:wat::holon::leaf v` (primitives → matching leaf variant)
- `:wat::holon::from-watast w` (WatAST → structural HolonAST)
- `:wat::holon::encode h` (HolonAST → encoded bytes)
- `:wat::holon::materialize h` (HolonAST → runtime Value) — polymorphic by output, but name is honest about operation

## Mission

**Restore the verb-naming family invariant: Pascal-Case = constructor; lowercase = operation.**

Specifically:
1. **Narrow `:wat::holon::Atom`** to `(Atom :HolonAST) -> HolonAST::Atom(_)` — matches sibling constructor pattern
2. **Rename `:wat::core::atom-value` → `:wat::holon::materialize`** — operation name (lowercase); polymorphic decode is HONEST as a single direction operation; namespace move into `:wat::holon` to colocate with the algebra
3. **Verify narrow-verb coverage for the input types that fell out of polymorphic Atom:**
   - Primitives → `:wat::holon::leaf` (already exists, arc 065)
   - WatAST → `:wat::holon::from-watast` (already exists, arc 221)
   - Collections (Vec/Tuple/HashSet/HashMap) → likely fold into `:wat::holon::Bundle` if it accepts shape variation, OR mint new constructor verbs
   - Uuid + future tagged literals → `:wat::holon::Bind` + `:wat::holon::Tag` composition (already honest)

## Scope

### Phase 1 — Substrate narrowing (Stone 225.1)

#### A. Narrow `:wat::holon::Atom` verb

`src/runtime.rs:13820` (`eval_algebra_atom`) / `:13838` (`value_to_atom`):
- Accept ONLY `Value::holon__HolonAST(_)` input
- Return `HolonAST::Atom(inner)` opaque-identity wrap
- DELETE all other input-arm branches (primitives, WatAST, collections, Uuid) — those become out-of-scope-for-Atom; consumers must use narrow verbs
- Rename `value_to_atom` Rust fn → `wrap_holon_as_atom` (since "atomize" is misleading and the fn now only does Atom-wrap)
- Update doc comments to reflect single-shape constructor

`src/check.rs:13558` TypeScheme:
- Change from `∀T. T → HolonAST` to `HolonAST → HolonAST` (narrow input type)
- Update `infer_list` special-case at `:5326` accordingly

#### B. Rename `:wat::core::atom-value` → `:wat::holon::materialize`

`src/runtime.rs:13633` (`eval_atom_value`):
- Rename verb: `:wat::core::atom-value` → `:wat::holon::materialize`
- Rename Rust fn: `eval_atom_value` → `eval_holon_materialize`
- Body unchanged — still polymorphic decode (HolonAST::Atom → inner; Bundle → Vec/HashMap/HashSet/Tuple by key shape; leaves → matching Value primitive)
- Doc comment refresh: name is honest now (operation, not borrowed variant)

`src/runtime.rs:13504` `holon_item_to_value`:
- Rename → `materialize_holon_item`
- Thread `op: &str` parameter (closes arc 224 L1-runtime-3 latent lie)
- All callers updated to pass their own op name

`src/check.rs:13591` TypeScheme:
- Keep `∀T. HolonAST → T` (honest); just change the verb-name string
- Update `infer_list` special-case at `:5362`

#### C. Verify narrow-verb coverage for retired Atom arms

For each input type that USED to go through `:wat::holon::Atom`, identify the honest narrow verb:

| Input type | Honest verb | Status |
|---|---|---|
| Value primitives (i64/f64/bool/String/keyword/Unit/Char) | `:wat::holon::leaf` | exists (arc 065); covers |
| Value::wat__core__Uuid | `:wat::holon::Bind` with `:wat::holon::Tag` composition | exists; consumers compose explicitly |
| Value::holon__HolonAST | `:wat::holon::Atom` (newly narrowed) | THIS STONE |
| Value::wat__WatAST | `:wat::holon::from-watast` | exists (arc 221); covers |
| Value::wat__std__HashSet | `:wat::holon::Bundle` (set-shape) | NEEDS VERIFICATION — does Bundle accept HashSet input? |
| Value::Vec / Tuple | `:wat::holon::Bundle` (positional/sequence) | needs verification |
| Value::wat__std__HashMap | `:wat::holon::Bundle` of `:wat::holon::Bind` pairs | needs verification |

If `:wat::holon::Bundle` doesn't currently accept Value-tier collection inputs (it currently takes `Value::Vec<HolonAST>`), Stone 225.1 may need a sub-stone to extend Bundle's constructor (OR mint dedicated `:wat::holon::Set` / `:wat::holon::Map` constructor verbs that lift collections into Bundle-composition shape).

### Phase 2 — Consumer sweep (Stone 225.2)

After Phase 1, every wat caller of `:wat::holon::Atom` that was passing non-HolonAST input will fail to type-check. Substrate-as-teacher cascade: read errors, replace each call with the appropriate narrow verb (`:wat::holon::leaf`, `:wat::holon::Bundle`, `:wat::holon::Bind`+`Tag`, etc.). Rename `:wat::core::atom-value` callers to `:wat::holon::materialize`.

Per pre-flight grep: ~31 Rust call sites for `:wat::holon::Atom`, ~10 for `:wat::core::atom-value`, ~54 wat-side caller sites total. Many will need REPLACEMENT (not just rename) under the narrowed-Atom doctrine.

### Phase 3 — Doc + USER-GUIDE + 058 spec (Stone 225.3)

Inscribe the verb-naming-family invariant in USER-GUIDE: Pascal-Case = constructor; lowercase = operation. Update verb reference docs. Cross-reference arc 224 audit + intueri family-pattern finding.

### Phase 4 — INSCRIPTION (Stone 225.4)

Arc 225 closure. Names the verb-naming-family invariant explicitly. Cross-references arc 065/221 (the earlier right-shape splits that prefigured this doctrine) + arc 224 (the audit that surfaced the remaining lie) + arc 221 (the doctrine dialogue that started it all).

## What this arc does NOT do

- Touch holon-rs (the algebra primitive `HolonAST::Atom` STAYS — intueri 224.1 confirmed honest)
- Touch wat-edn wire format
- Other Group A small fixes from arc 224 AGGREGATE — those are Stone 224.5's scope
- L2 mumbles
- Audit the OTHER Pascal-Case sibling verbs for polymorphic-overload (preliminary check shows they're honest; if a future cast finds otherwise, that's a future arc)

## Calibration (revised under Option A shape)

| Stone | Scope | Predicted |
|---|---|---|
| 225.1 | substrate narrow (Atom + materialize rename + sub-stone for Bundle/Set/Map coverage if needed) | 60-150 min Mode A |
| 225.2 | consumer sweep with REPLACEMENT not just rename | 90-180 min Mode A — substrate-as-teacher cascade |
| 225.3 | doc + USER-GUIDE + 058 | 30-60 min |
| 225.4 | INSCRIPTION | 30 min |

**Total estimate:** 3.5-7 hours sonnet across the 4 stones.

The estimate is similar to Option B's was, but the work shape is DIFFERENT:
- Option B: pure rename (95% of sites are mechanical Atom → atomize replacements)
- Option A (current): rename ~10% (atom-value → materialize) + REPLACE ~90% with appropriate narrow verbs (caller-by-caller decision on which narrow verb fits each site)

Option A is more thinking work per call site but produces an honest verb family.

## Cross-references

- arc 224 AGGREGATE-FINDINGS.md — Group B scope that this arc addresses
- arc 224 FINDINGS-INTUERI-RUNTIME.md — L1-1 + family pattern finding
- arc 065 — the original `Atom` split into `leaf` / `from-watast` / narrowed Atom; this arc completes that work by also pulling the collection arms out
- arc 221 — substrate-doctrine arc that surfaced the verb-naming honesty question
- INTERSTITIAL § 2026-05-22 very-late → 2026-05-23 — the realization arc
- [[atom-is-holder]] — substrate doctrine
- [[spawn-block-winding]] — arc 225 parentage

## Open question for the BRIEF (one remaining)

**Does `:wat::holon::Bundle` accept collection-shape Value inputs (Vec/Tuple/HashSet/HashMap), or only `Value::Vec<HolonAST>`?** If the latter (preliminary grep suggests so), Stone 225.1 needs a sub-stone that either:
- Extends Bundle's constructor to recognize the collection types, OR
- Mints dedicated `:wat::holon::Set` / `:wat::holon::Map` constructor verbs that produce Bundle composition shapes

This is an Option A implementation detail the BRIEF must resolve before sonnet runs. Pre-Stone-225.1 grep needed.

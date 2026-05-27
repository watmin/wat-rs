# Arc 237 — Remaining stone order (LIVE TRACKER)

**This is a LIVE roadmap, NOT an immutable record.** Refactor it freely as stones ship
(same status as the cliffnotes index; the `feedback_inscription_immutable` exception —
SCOREs/INSCRIPTIONs are frozen, this tracker is not). A fresh post-compaction self:
read this to know what's left, the order, and **why** the order — then verify against
`git log --oneline | grep 237` before acting (git is truth; this tracker can lag).

> **▶ RESUMED 2026-05-27 — ARC 238 CLOSED** (`docs/arc/2026/05/238-core-equality-completeness/INSCRIPTION.md`).
> `:wat::core::=` is now deep-structural over all EDN data incl records/maps/sets (was erroring;
> proven + fixed; 834/0). The user's clean split is locked: **`=` owns type-strict** (same type +
> same values — done by 238); **`same-data?` is the type-BLIND tool** (the 2×2 flavor grid /
> cross-type field comparison), still a real distinct verb. REMAINING in the records thread:
>   - **S-C.2d** ✓ SHIPPED `6ea2270e` — `:wat::Record/same-data?` (type-blind record data-equality;
>     name-keyed via record->map + values_equal). FM-9: probe 6/6, lib 834/0. The clean split is live:
>     `=` type-strict (238) + `same-data?` type-blind. USER-GUIDE row added.
>   - **S-C.3** ✓ SHIPPED `e9e24139` — macro split: `:wat::Record::def`=BASE / `:wat::holon::Record::def`=HOLONIC;
>     constructor split (`:wat::Record::of` 2-arg / `:wat::holon::Record::of` 3-arg); recordtype-parent
>     Liskov (base-defined REJECTED at `:wat::holon::Record` param — proven, probe contract 13). FM-9:
>     18/18, lib 834/0, workspace 0-FAILED.
>   - **S-D** ✓ ABSORBED into S-C.3's cascade — 5 test files migrated to holonic (holon-op use), rest base.
>
> **▶▶ RECORDS FLAVOR THREAD CLOSED.** What remains of arc 237 = the ARITHMETIC TAIL (disjoint machinery):
> 237.7 (arc-146 Dispatch entities → defclauses) → 237.8 (arithmetic/comparison/holon-pair/time-arith →
> concrete-per-type defclauses; DELETE widest-contagion; HARD-CUT arc-146 Dispatch) → 237.9 INSCRIPTION
> (TERMINAL — folds records S-E + arc 146 + arc 148 closures + the USER-GUIDE records section).

Arc 237 is a **two-boss level** (no one-boss-per-level rule): the records-first-class
thread AND the arithmetic/dispatch consolidation (the original spine). The records
*dragon* is slain; what remains is one records *refinement* + the whole arithmetic boss
+ the shared closure.

---

## Ordering principle (user direction, 2026-05-26)

> **Momentum: finish the paths we're on, then loop back onto consumers.**

We are ON the records path (just shipped S-A1). Finish it (S-C → S-D), close the thread,
THEN move to the arithmetic tail (the consumer-migration boss), THEN the shared closure.

**Honesty on the cross-boss order:** records (Record.wat macro + `holon_form`) and
arithmetic (Dispatch + widest-contagion) touch **disjoint** machinery — neither boss
produces an artifact the other needs. So records-first is NOT a hard stepping stone;
it's the momentum + recency-of-context tiebreaker (the is-a hierarchy S-C rides on is
hot from S-A/S-A1/S-B.2). The *intra-boss* orderings below ARE hard.

---

## The order (✓ = shipped; verify via git)

```
RECORDS PATH (finish what we're on)   [S-C re-sliced 2026-05-26 — two Value variants; field-access-via-struct; see DESIGN CORRECTION 1 + 2]
  S-C.1 ✓ SHIPPED 0c574661 — RENAME Value::wat__Record → Value::wat__holon__Record (the dual-form IS holonic; freed the name)
  --- the field-access model (CORRECTION 2): access via the STRUCT for BOTH flavors; holon-ops holonic-only; field NAMES are a class property ---
  S-C.2ab ✓ SHIPPED eda4d6cd — field names → RecordDef.field_names (3-arg recordtype + macro emits) + 4 name→index sites re-routed off holon_form (keyword_accessor_record + record→map + assoc + struct-destructure) + FM-9 multi-field name-order guard (probe_arc237_sC2ab_field_order 5/5). PARITY preserved; baseline-preserving.
  S-C.2c ✓ SHIPPED 601c892d — base Value::wat__Record {class_fqdn, struct_form}: structural Eq/Hash; field access variant-agnostic via 2ab field_names path; holon-ops teaching-error (base wat-local, no holon flavor). FM-9 verified: build 0 / lib 828 / probe 6/6 / regressions green. 1 cascade round. (3 base-structural arms + 14 or-pattern reads + 4 holon-op error arms across runtime.rs/edn_shim.rs/closure_extract.rs.)
  S-C.2d MINT :wat::Record/same-data? — flavor-agnostic record DATA-equality. User-surfaced (validates Seam-1: = is flavor-sensitive, so a "same data, any flavor" reach was missing). Compares class_fqdn + struct_form across all 4 (base|holonic)² pairs; distinct from = (identity). Depends on S-C.2c. NAME via intueri (re-cast): "data=?" REJECTED (double-sigil cold-read fail "wtf"), converged on same-data? (one sigil, word-carried, lexer-safe kebab+?). Needs sub-DESIGN + FM-2-bis probe.
  S-C.3 macro split: :wat::Record::def → base / :wat::holon::Record::def → holonic; static type distinction = constructor return type; wat-surface proof   ← NEXT (records substrate)
  S-D   migrate existing :wat::Record::def callers → base vs holonic (HARD CUT)   ← the records "consumer" loop-back
        └─ records thread CLOSED (inscription deferred to 237.9)

ARITHMETIC / DISPATCH BOSS (loop back onto consumers)
  237.7 arc-146 Dispatch entities → defclauses (length/empty?/contains?/get/conj/concat/assoc/dissoc/keys/values)
  237.8a arithmetic + comparison + holon-pair + time-arith → concrete-per-type defclauses (ADDITIVE; old path still standing)
  237.8b DELETE widest-contagion (infer_arithmetic / eval_arithmetic_variadic / is_numeric) + HARD CUT arc-146 Dispatch + update AnyBanned

SHARED CLOSURE
  237.9 INSCRIPTION + arc closure — folds arc 146 + arc 148 + records S-E. TERMINAL.
```

(8a/8b is a recommended split to FINALIZE at 237.8 design-time with a blast-radius grep —
not yet locked; see "Hard dependencies" for why the build/cut boundary matters.)

---

## Hard dependencies — DO NOT reorder across these

1. **237.7 → 237.8.** *Forced:* 237.8 HARD-CUTs the arc-146 `Dispatch` entity; you cannot
   delete the entity while collection ops still live in it — 237.7 evacuates its last
   tenants. *De-risking:* 237.7 proves the `Dispatch → defclause` recipe on the SAFE
   family before 237.8 applies it to the dangerous families + pulls the deletion trigger.
   (Grep-confirm at 237.7 time that those 10 ops are the COMPLETE Dispatch tenant set —
   FM-2: don't trust the list is exhaustive.)
2. **237.8a → 237.8b (build, then cut).** The widest-contagion deletion + Dispatch HARD
   CUT is tractable ONLY after the new concrete defclauses prove green coverage. Never
   delete blind. THE DECISION is locked: no implicit numeric coercion — `(+ 1 2.0)` →
   ERROR (no clause matches); homogenize explicitly. (`feedback_no_implicit_coercion`.)
3. **S-C.1 ✓ → S-C.2a → S-C.2b → S-C.2c → S-C.3 → S-D.** Forced chain. S-C.1 (rename, shipped)
   freed the `wat__Record` name. S-C.2a puts field names on the class (RecordDef) so name-based
   access has a variant-agnostic source; S-C.2b re-routes keyword-access onto that source
   (off `holon_form`) so it works for both flavors; S-C.2c mints base on the freed name (field
   access via 2b; holon-ops error); S-C.3's macros can't dispatch to a base variant that
   doesn't exist; S-D can't migrate to macros that don't exist. Two rejected shapes, for the
   record: `holon_form: Option` (semantic abuse — flavor is the *variant*, decoded by `match`;
   `feedback_no_semantic_abuse_of_option`) AND on-demand projection (holonic *stores* both
   flavors; base *has only* the struct — no projection; CORRECTION 2).
4. **237.9 LAST, gated on BOTH bosses.** Spawn-block winding: the arc cannot close until
   every thread under it closes. 237.9 is the single closure for the whole level
   (absorbs arc 146 + arc 148 + records S-E).

---

## Shipped so far (arc 237) — git-confirmed

```
Machinery + conformance doctrine:
  237.1 typeunion (TypeDef::Union + bounded-existential unify)              d40eb4a3
  237.2 defclause foundation (arity + type dispatch)                        bdd9eb6c
  237.3 :guard + :ensure clause-keywords                                    ee5e892c
  237.4 rich :NoMatchingClause + :PostconditionFailed                       5f7bb6e5
  237.5 :wat::core::conforms? general conformance primitive                 5d667123
  237.5.fix one wildcard-free Value::declared_type_name authority (✅✅✅)    990542a9
  237.6 auto-mint is-<Name>? as named convenience over conforms?            3ae844cb

Records thread (the dragon — SLAIN):
  S0   gate probe (macro-emitted type decls go first-class)                 da059f42
  S-A  is-a hierarchy mechanism (typesub + subtype? + is_subtype + roots)   d1e9cbe9
  S-B.1 records become first-class types (recordtype + TypeDef::Record)     89c01888
  S-B.2 defrecord emits recordtype + drops its predicate                    86aebfcb
  S-A1  assignable choke point — subtyping at the arg boundary (Liskov)     531ba9b7
```

Records dragon ("records aren't first-class types") = slain: records are TypeDefs, `is-X?`
synthesizes ∀T, and the is-a hierarchy is consulted at the argument boundary (Liskov,
Convergence #17). **S-C is a refinement** the slain dragon enables (base records stop
paying for `holon_form` they don't use; holonic ones still substitute for base via S-A1)
— real arc-237 work, but NOT load-bearing for "first-class."

---

## Next room

**S-C.1** — RENAME `Value::wat__Record` → `Value::wat__holon__Record` (the existing
dual-form variant IS the holonic one — it implements the hologram; it was doing both
jobs under the wrong name). Pure mechanical sweep (~72 `holon_form` sites, mostly
`runtime.rs`), **baseline-preserving** — the safe foundation that frees the `wat__Record`
name for the base variant. Same shape as S-A1: inert + baseline-preserving by
construction. Then S-C.2 mints base, S-C.3 splits the macros, S-D migrates callers, the
records thread closes; then the arithmetic boss; then 237.9.

NOT a small macro split — it's a runtime reshape (the `holon_form: Option` shape was
REJECTED as semantic abuse; flavor is the *variant*). See § DESIGN CORRECTION in
`DESIGN-RECORDS-AS-FIRST-CLASS-TYPES.md`.

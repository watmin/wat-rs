# 293 — Declaration unification: one `aggregatetype`, the holder DERIVED from the parent

> **Status: STRIKE DRAWN — lair studied + grounded 2026-06-28.** The next step in `CLOSE-SEQUENCE-293-294.md`.
> Builder catch: *"recordtype is the only varying value … why are struct and record being tolerated as yet another
> split not a unification."* The audit's principle at the declaration layer: the holder is a passing policy; the
> declaration is holder-agnostic `aggregate`.

## The bug (grounded on the disk)
Three declaration macros over TWO type-reg primitives over TWO parse fns — all producing ONE `TypeDef::Aggregate`,
differing only by holder:
- `defstruct` (`core.wat:1046`, thin macro) → `structtype` (`types.rs:1732 → parse_defstruct`) → `AggregateDef{holder:Struct, parent:":wat::core::Value", restrictions}`
- `defrecord` (`Record.wat:91`) / `holon::defrecord` (`Record.wat:130`) → `recordtype` (`types.rs:1740 → parse_recordtype`) → `AggregateDef{holder: Record|HolonRecord, parent, restrictions:None}`

The two `parse_*` fns (grounded `types/defstruct.rs:328` + `types.rs:2102`) differ ONLY in:
| axis | parse_defstruct | parse_recordtype |
|---|---|---|
| holder | hardcoded `Struct` | **derived from parent**: `:wat::holon::Record`→HolonRecord, else Record |
| parent | hardcoded `:wat::core::Value` | the `:Parent` arg |
| metadata/restrictions | optional `{…}` 2nd arg → `StructRestrictions` | none |
| field parse | `parse_defstruct_fields` | inline groups-of-3 (**a duplicate field-parser**) |
| name parse | `parse_declared_name` (shared) | `parse_declared_name` (shared) |

The two record macros are already byte-identical except the `recordtype` holder keyword (confirmed by 294.c.2a).

## The contract (pinned) — the holder is DERIVED from the parent's root
**The categorical position IS the parent.** Unify on ONE primitive whose holder is *computed*, not passed:
```
(:wat::core::aggregatetype :Name :Parent {metadata}? [fields])
   holder = root_holder_of(:Parent):  :wat::core::Value → Struct
                                       :wat::Record       → Record
                                       :wat::holon::Record→ HolonRecord
```
`parse_recordtype` ALREADY derives holder from parent; this completes it (struct's `:Value → Struct`). No holder
argument — the parent keyword carries it, which is honest (the holder IS the categorical-root position). Metadata
optional for ANY holder (a record could carry restrictions too — feature union, not struct-only). ONE field-parser.

## Decomposition (decl-a then decl-b — resolves the builder's open "type-reg-first" question)
- **decl-a (Rust — the primitive + parse).** Mint `(:wat::core::aggregatetype …)` → ONE `parse_aggregate(args)`:
  `parse_declared_name` (shared) + optional metadata + ONE field-parser (the `parse_defstruct_fields` dedup) +
  `root_holder_of(parent)`. `structtype` / `recordtype` become **thin aliases** routing to `parse_aggregate`
  (additive — both still parse). Gate: a `(aggregatetype :T :wat::core::Value [..])` /
  `:wat::Record` / `:wat::holon::Record` each register the right holder; existing defstruct/defrecord/holon tests
  stay GREEN; SET-diff ∅.
- **decl-b (the macros + annihilation).** `defstruct` / `defrecord` / `holon::defrecord` emit
  `(:wat::core::aggregatetype :Name :Parent {meta}? [fields])` — the three become thin holder-keyed delegations over
  ONE shared emission (differ ONLY in the `:Parent` keyword passed). `structtype` / `recordtype` annihilated
  (dispatch arms + the alias) + retirement-table the heads + `deporder.wat` updated (it lists them as dep heads).
  The `defrecord`/`holon::defrecord` `syms` field-extraction `let` (duplicated, confirmed by c.2a) collapses into
  the one shared emission.

## STOP triggers
- **STOP-PARENT:** if `root_holder_of` can't cleanly map every legal parent (e.g. a record extending a non-root base
  like `:wat::program::Env` — `parse_recordtype:2128` notes holder is the **root** of the parent chain, not the
  immediate parent) — the derivation must follow the parent chain to its root, not just match the immediate keyword.
  If that root-walk isn't available at parse time, STOP and surface (it may need registration-time resolution, like
  the existing parent-validity check at `register_with_span`).
- **STOP-META:** if unifying the metadata path changes struct restriction behavior (the `StructRestrictions` /
  ctor-whitelist semantics) — keep it behavior-identical; metadata stays optional + struct-only-in-practice until a
  record needs it. Surface if the union isn't free.
- **STOP-FIELD:** if `parse_defstruct_fields` and the recordtype inline field-parse are NOT actually identical
  (different error messages / edge cases) — reconcile explicitly, don't silently pick one.

## Blast radius
`types.rs` (dispatch 1730-1740 + `parse_recordtype`), `types/defstruct.rs` (`parse_defstruct` → `parse_aggregate`),
`core.wat` (`defstruct` macro), `wat/Record.wat` (both record macros), `wat/deporder.wat` (dep heads), any direct
`.wat` `structtype`/`recordtype` callers (the macros expand to them; check for direct uses). The builtin-struct
registrations (`types.rs:552+`, ~15 `register_builtin(AggregateDef{holder:Struct,…})`) are Rust-direct, unaffected.

## Pairs
`CLOSE-SEQUENCE-293-294.md` (step 1) · `AGGREGATE-AUDIT.md` (the holder-is-passing principle) · `DESIGN.md`
(HOLDER × SURFACE) · `NOTE-base-struct-horizon.md` (the base-struct foundation this realizes) ·
`feedback_replicate_is_a_duplication_smell`.

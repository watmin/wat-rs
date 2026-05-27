# EXPECTATIONS — Stone S-B.1

Mode A: 6/6 on the probe + clean baseline + records-become-types (`recordtype`
mints `TypeDef::Record`, wires the `typesub` edge, synthesizes `is-X?` ∀T), with
`conforms?` staying tier-3 (nominal Record arm only) and Record.wat untouched.

## Scorecard

| # | Row | Verification | Expected |
|---|---|---|---|
| 1 | Compile clean | `cargo build --release -p wat 2>&1 \| tail -5` | 0 errors |
| 2 | **S-B.1 probe 6/6** (LOAD-BEARING) | `cargo test --release --test probe_arc237_sB1_recordtype 2>&1 \| tail -3` | `6 passed; 0 failed` |
| 3 | Lib baseline held | `cargo test --release --lib -p wat --no-fail-fast 2>&1 \| tail -3` | ≥ 827 passed; 0 failed |
| 4 | 237.1 typeunion regression | `cargo test --release --test probe_arc237_stone1_typeunion_substrate 2>&1 \| tail -3` | `14 passed; 0 failed` |
| 5 | 237.5 conforms? regression | `cargo test --release --test probe_arc237_stone5_conforms 2>&1 \| tail -3` | `12 passed; 0 failed` |
| 6 | 237.6 is-predicate regression | `cargo test --release --test probe_arc237_stone6_is_predicate 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 7 | S-A hierarchy regression | `cargo test --release --test probe_arc237_sA_hierarchy 2>&1 \| tail -3` | `10 passed; 0 failed` |
| 8 | is-X? ∀T (asymmetry dead) | probe 2 | `(:my::is-Circle? 42)` → `false`, NOT a type error |
| 9 | edge wired + transitive | probe 3/5 | `subtype? :my::Circle :wat::Record` → true; Sphere→holon→Record true |
| 10 | unknown parent rejected | probe 6 | `recordtype` w/ unknown parent → startup error |
| 11 | holon-rs untouched | scope | STOP-5 — zero holon-rs changes |
| 12 | files in scope | `git status --short` | `src/types.rs`, `src/runtime.rs` (+ check.rs/closure_extract.rs cascade only) + SCORE doc |

**Clippy NOT a ceiling concern** per standing direction.

## Independent prediction

**Target band: 45–75 min Mode A. STOP-3: 100 min. STOP-4 (hard kill): 130 min.**

Mirror of the 237.1 typeunion mint (new `TypeDef` variant + decl-form parse +
register + cascade), which shipped in-band, plus one addition (edge-wiring via
`register_subtype`) and two small arms (`register_type_predicates`,
`conforms_check`). Cascade: 2–3 rounds; a handful of forced `TypeDef`-exhaustiveness
sites (Union precedent: ~4); 0 new files.

## Risks / trap-doors

1. **Record-as-Struct shortcut** — registering the class as `TypeDef::Struct` trips
   `register_struct_methods` (iterates all Struct TypeDefs, freeze.rs:853) into a
   spurious `:my::Circle/new` + accessors. Dedicated `TypeDef::Record`, never fed to
   that pass. (BRIEF STOP-8.)
2. **Hierarchy walk creeping into `conforms?`** — tier-3 stays nominal/union/structural;
   the lineage walk is the separate `subtype-of?` stone. B.1's conforms? Record arm
   is nominal-exact only.
3. **Cascade misread as crisis** — adding a `TypeDef` variant fails many exhaustive
   matches; that's substrate-as-teacher, bounded. Add Record arms mirroring Struct.
4. **Edge cycle / unknown parent** — `register_subtype` cycle-checks; ALSO verify the
   parent resolves to a known type before wiring (probe 6 asserts the reject).
5. **is-X? collision** — once the record class is a TypeDef, `register_type_predicates`
   synthesizes `:my::is-Circle?`. Record.wat ALSO still emits its own `is-Circle?`
   today → **DuplicateDefine** would fire. BUT B.1 does NOT change Record.wat, and the
   probe uses `recordtype` DIRECTLY (no defrecord), so no class is declared via BOTH
   paths → no collision in B.1. (The collision is resolved in S-B.2 when the macro
   drops its hand-emitted predicate. If any existing lib test defrecord's a class AND
   B.1's synthesis now also fires for it — it won't, because defrecord does NOT emit a
   `recordtype` form until B.2, so defrecord'd classes are still NOT TypeDefs in B.1.)

## SCORE

`SCORE-STONE-S-B1.md` (NEW). 12-row scorecard verbatim + Final API shape
(`recordtype` form + `TypeDef::Record`/`RecordDef` + the two arms + edge-wiring) +
line counts + cascade depth (list forced sites) + honest deltas + working tree.
Mirror SCORE-STONE-237.1 shape.

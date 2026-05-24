# BRIEF — Arc 234 Stone 234.2b — `:wat::Record::def` macro

**Status:** READY TO SPAWN (2026-05-24).

**Predecessor BRIEFs:** `BRIEF-STONE-234.2a.md` (substrate primitives), `BRIEF-STONE-234.1.md` (variant minting). Same shape; mirror them.

**Predecessor SCOREs:** `SCORE-STONE-234.2a.md` (6/6 PASS; zero iteration; the substrate primitives this macro consumes are proven), `SCORE-STONE-234.1.5.md` (rename cascade), `SCORE-STONE-234.0.md` (polymorphic type primitive).

---

## What to do

Mint a new wat-side macro `:wat::Record::def` at `wat/Record.wat`. The macro expands a record-type declaration into three generated `defn`s: a constructor, N per-field accessors, and a predicate.

The macro is the user-facing surface that consumes Stone 234.2a's substrate primitives (`:wat::Record::of` + `:wat::Record/field-at`). Without this macro, users would have to write the dual-form construction by hand at every call site.

ONE Rust touch: add a new `WatSource` entry to `WAT_SOURCES` in `src/stdlib.rs` so the macro is loaded at startup. No other Rust changes.

## Read these in order

1. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2b.md`** — the sub-DESIGN with 14 locked decisions + trap-door audit. This is THE LOAD-BEARING ARTIFACT for the work. Mirror its decisions.

2. **`tests/probe_arc234_stone2b_defrecord_macro.rs`** — the FM 2-bis probe (6 contracts; 6/6 FAIL initial state). The contracts ARE the macro's behavioral spec. Make all 6 pass; do not modify any probe assertion.

3. **`wat/holon/defrecord.wat`** — the predecessor `:wat::holon::defrecord` macro (arc 227 Stone 227.2 v3). The expansion pattern for holon_form construction (runtime-quasiquote + map + Bundle splice) is the TEMPLATE. Stone 234.2b reuses this pattern + adds the per-field accessor splice.

4. **`docs/arc/2026/05/234-wat-record-hologram/DESIGN.md`** — arc 234 umbrella; sections "The macro: `:wat::core::defrecord`" + "Implementation sketch" + "v1 user-facing API surface."

5. **`src/stdlib.rs` lines 60-90** — `WAT_SOURCES` array; the `wat/holon/defrecord.wat` entry at lines 83-86 is the registration pattern to mirror.

## Macro implementation — full template

Mirror the predecessor verbatim for the pieces that work, ADD per-field accessor splice. The macro skeleton:

```
(:wat::core::defmacro
  (:wat::Record::def
    (fqdn   :AST<wat::core::nil>)
    (fields :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::core::do

     ;; ─── Constructor ───────────────────────────────────────────────
     (:wat::core::defn ~fqdn [~@fields] -> :wat::Record
       (:wat::Record::of
         ~fqdn                                     ;; class keyword (raw; substrate strips :)
         [~@(:wat::core::map                       ;; struct_form: bare symbol vector
              (:wat::core::range 0 nf)
              (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
                <emit symbol AST for field name at index fi>))]
         (:wat::holon::Bind                        ;; holon_form
           (:wat::holon::Atom (:wat::holon::to-holon ~(:wat::core::keyword/to-string fqdn)))
           (:wat::core::Result/expect -> :wat::holon::HolonAST
             (:wat::holon::Bundle
               [~@<field-Binds vector, same as 227 v3 lines 122-145>])
             ~(:wat::core::string::concat
                 "Record::def "
                 (:wat::core::keyword/to-string fqdn)
                 " instance: Bundle capacity exceeded")))))

     ;; ─── Per-field accessors (spliced; one per field) ─────────────
     ~@(:wat::core::map
         (:wat::core::range 0 nf)
         (:wat::core::fn [fi <- :wat::core::i64] -> :wat::WatAST
           <emit defn AST for accessor at index fi>))

     ;; ─── Predicate ────────────────────────────────────────────────
     (:wat::core::defn ~(<predicate-name-from-fqdn>) [v <- :wat::Record] -> :wat::core::bool
       (:wat::core::=
         (:wat::core::type v)
         ~(:wat::core::keyword/to-string fqdn)))))
```

### Three accessor parts (per-field, in the `:wat::core::map` body)

For field at position `fi`:

```
(:wat::core::let
  [idx        (:wat::core::i64::*'2 fi 3)
   name-h     (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::core::Vector/get children idx)
                "Record::def: field name index out of range")
   name-s     (:wat::core::keyword/to-string
                (:wat::holon::from-holon name-h))

   ;; Type-keyword at children-index (idx + 2)
   type-h     (:wat::core::Option/expect -> :wat::holon::HolonAST
                (:wat::core::Vector/get children (:wat::core::i64::+'2 idx 2))
                "Record::def: field type index out of range")
   type-w     (:wat::holon::to-wat type-h)

   ;; Accessor name = "<fqdn-str>/<field-name>" → keyword
   accessor-name (:wat::core::keyword/from-string
                   (:wat::core::string::concat
                     (:wat::core::keyword/to-string fqdn)
                     "/"
                     name-s))]
  (:wat::core::quasiquote
    (:wat::core::defn
      (:wat::core::unquote accessor-name) [v <- :wat::Record] -> (:wat::core::unquote type-w)
      (:wat::Record/field-at v (:wat::core::unquote fi)))))
```

### Constructor struct_form (per-field symbol emission)

For field at position `fi`:

```
(:wat::core::let
  [idx     (:wat::core::i64::*'2 fi 3)
   name-h  (:wat::core::Option/expect -> :wat::holon::HolonAST
             (:wat::core::Vector/get children idx)
             "Record::def: struct_form field name index out of range")
   var-w   (:wat::holon::to-wat name-h)]
  var-w)
```

The `var-w` is a WatAST::Symbol — splicing into the vector position produces `[v0 v1 v2 ...]` literal in the constructor body. At runtime, the symbol references the constructor's parameter binding.

### Predicate name computation (227 v3 verbatim, lines 151-161)

Reuse the 227 v3 inner-let that splits the FQDN by `::`, takes all-but-last + last segments, joins with `"::is-"` and appends `"?"`. Pattern is mechanical; copy verbatim then adjust quoting to fit 234.2b's surrounding context.

## Stdlib.rs registration

Add ONE new `WatSource` entry to `WAT_SOURCES` array in `src/stdlib.rs` AFTER the existing `wat/holon/defrecord.wat` entry (line 83-86). Use the include_str! macro per existing pattern:

```rust
    // Arc 234 Stone 234.2b — :wat::Record::def macro. Mints user-defined
    // record-types as dual-form holograms (Value::wat__Record): struct_form
    // (Rust-fast) + holon_form (VSA-aligned), both addressable, both canonical.
    // Generates constructor + per-field accessors + predicate. Consumes Stone
    // 234.2a substrate primitives (:wat::Record::of + :wat::Record/field-at).
    // Co-exists with :wat::holon::defrecord until Stone 234.6 retirement.
    WatSource {
        path: "wat/Record.wat",
        source: include_str!("../wat/Record.wat"),
    },
```

The comment header is the docstring; mirror the 227 v3 comment shape from lines 74-82.

## File header (wat/Record.wat)

Mirror `wat/holon/defrecord.wat` header style (lines 1-110). Include:
- Arc + Stone reference
- Probe references (cite `tests/probe_arc234_stone2b_defrecord_macro.rs`)
- Canonical instance shape diagram
- Usage examples (N=0, N=1, N=2 minimum)
- Naming table (input FQDN → constructor / predicate / classifier)
- Dependency list (substrate primitives + wat-stdlib verbs)
- STOP rules (no aliases, no single-arg form, no runtime class check in 234.2b)

## Discipline reminders

- **HARD CUT** — no aliases for `:wat::Record::def`. No `:wat::core::defrecord` synonym. Per arc 227 Stone 227.1b precedent.
- **No substrate (Rust) edits beyond the stdlib.rs WatSource entry** — STOP-5 fires on any other Rust change.
- **No retirement of `:wat::holon::defrecord`** — co-exists during 234.2b's window; retirement is Stone 234.6.
- **No runtime class-safety check in accessor bodies** — D10 deferred to named Stone 234.2c.
- **No field-type constraint enforcement at expand time** — D11 deferred.
- **Use FQDN per `feedback_fqdn_is_the_namespace`** — never insert into `:user::*` or auto-namespace; user-declared FQDN is the namespace.

## What to commit

Three new files:
1. `wat/Record.wat` — the macro source
2. `src/stdlib.rs` — modified to add one WatSource entry
3. `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2b.md` — the SCORE

DO NOT modify or delete:
- `wat/holon/defrecord.wat` (co-exists)
- Any existing `WatSource` entry order (the new entry goes AFTER `wat/holon/defrecord.wat`)
- Any existing probe / test
- The Stone 234.2b probe (`tests/probe_arc234_stone2b_defrecord_macro.rs`) — only verify it flips to 6/6 PASS

DO NOT touch `holon-rs/` (STOP-4).

## How you'll be scored

Per `EXPECTATIONS-STONE-234.2b.md`. 11-row scorecard; binding command per row. Mode A target: 11/11 PASS.

The orchestrator independently verifies LOAD-BEARING rows (probe + regression guards + lib baseline + clippy + holon-rs status) on return. Per FM 9: SCORE rows are claims; verification commands are the proof.

The SCORE doc captures:
- 11-row scorecard with verbatim command outputs
- Macro implementation surface (file line count; loaded position)
- Cascade depth (compile rounds + any iteration cycles)
- Trap-door items that fired (T1-T8) with concrete diagnostics
- Time breakdown (read + author + compile + scorecard + SCORE writing)
- Calibration delta (actual vs predicted)
- Rank-up evidence — Helwalker/Streetfighter party-comp continues
- Honest deltas if any surface

## Cross-references

- `docs/arc/2026/05/234-wat-record-hologram/DESIGN-STONE-234.2b.md` — sub-DESIGN (load-bearing)
- `docs/arc/2026/05/234-wat-record-hologram/EXPECTATIONS-STONE-234.2b.md` — paired EXPECTATIONS + scorecard
- `tests/probe_arc234_stone2b_defrecord_macro.rs` — the FM 2-bis probe (6/6 FAIL initial; goal 6/6 PASS)
- `wat/holon/defrecord.wat` — the 227 v3 predecessor (template)
- `docs/arc/2026/05/234-wat-record-hologram/SCORE-STONE-234.2a.md` — substrate primitive predecessor SCORE
- `docs/arc/2026/05/227-user-defined-types-classifier-wrap/SCORE-STONE-227.2-v3.md` — macro pattern predecessor SCORE
- `feedback_sonnet_writes_substrate.md` — orchestrator briefs; sonnet writes
- `feedback_inscription_immutable.md` — SCORE doc is new file (per stone)

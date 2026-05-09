# Arc 169 slice 1 — substrate consumer + walker + tests

## Goal

Mint `WatAST::StructPattern(Vec<WatAST>, Span)` as a first-class
substrate AST node and consume it at let-binding-position to
implement the struct-destructure form A:

```scheme
(:wat::core::let [{outcome grace-residue} p]
  (:io::print outcome))
```

The 12-word user-authored rule (DESIGN's anchor):

> **bind the field's value to the field's name in this scope**

Each bare symbol inside `{}` IS both:
- the field name (resolved against the prior binding's struct
  type via TypeEnv struct registry)
- the binding name (in the let's local scope)

After this slice, the substrate accepts the new shape; arc 169
slice 2 ships closure paperwork.

## Branch + commit policy

- **Active branch**: `arc-169-struct-destructure-form-a` (already
  checked out; carries the DESIGN amendment locking in StructPattern)
- Multiple WIP commits + pushes welcome
- DO NOT push to main; orchestrator merges atomic to main as one
  squash commit after slice 2 closure paperwork ships

## Background context (read these first)

- `docs/arc/2026/05/169-struct-destructure-form-a/DESIGN.md` —
  full arc scope, settled four-questions evaluation, semantic
  alignment with arc 098, out-of-scope items
- Arc 167 INSCRIPTION (`docs/arc/2026/05/167-fn-flat-signature/INSCRIPTION.md`)
  — minted `WatAST::Vector` as the precedent for adding a
  first-class substrate AST node; arc 169 mirrors the pattern
- Arc 168 INSCRIPTION (`docs/arc/2026/05/168-let-flat-shape/INSCRIPTION.md`)
  — `parse_let_binding` Symbol + Vector arms; arc 169 adds the
  StructPattern third arm
- Arc 098 (`docs/arc/2026/04/098-wat-form-matches/DESIGN.md`) —
  the field-name contextual reading precedent (`(= ?var :field)`
  inside matches?). Arc 169 extends the same field-name semantic
  to a flatter syntactic surface in let bindings.
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § 6 (FM 5, FM 9, FM 10,
  FM 11) — discipline floor

## End-state shape

```scheme
;; Single field
(:wat::core::struct :test::PaperResolved
  (outcome       :wat::core::String)
  (grace-residue :wat::core::f64))

(:wat::core::define
  (:user::report (p :test::PaperResolved) -> :wat::core::nil)
  (:wat::core::let
    [{outcome grace-residue} p]
    (:io::print outcome)
    (:io::print-f64 grace-residue)))

;; Mixed with regular bindings
(:wat::core::let
  [whole p
   {outcome grace-residue} p]
  (:io::print outcome))

;; Nested let — outer destructure + inner uses bindings
(:wat::core::let [{outcome grace-residue} p]
  (:wat::core::let [doubled (:wat::core::f64::* grace-residue 2.0)]
    (:io::print-f64 doubled)))
```

## Substrate edits

### 1. `src/lexer.rs` — token mint

Add `Token::LBrace` and `Token::RBrace` for `{` and `}`.

These tokens have no lexer meaning today (verified 2026-05-08).
The lexer needs to recognize the chars + emit the tokens. Pattern
mirrors `Token::LBracket`/`RBracket` from arc 167 slice 1.

### 2. `src/holonast.rs` (or wherever `WatAST` lives) — AST variant

Add:

```rust
WatAST::StructPattern(Vec<WatAST>, Span)
```

Plus:
- `TAG_STRUCT_PATTERN: u8` — distinct tag for hash identity
  (mirrors arc 167 slice 1's `TAG_VECTOR`). Pick the next free tag
  byte after existing variants.
- `WatAST::span()` arm
- `Display`/`Debug` arms if applicable
- Hash arm carrying `TAG_STRUCT_PATTERN` then the elements

The Vec carries the bare-symbol field-name children. Validation
that all inner items are bare Symbols happens at PARSE time;
non-Symbol contents → clean `MalformedForm` from the parser.

### 3. `src/parser.rs` — parser path

Add a parse path for `LBrace ... RBrace` producing `WatAST::StructPattern`.

- Empty `{}` → clean `MalformedForm` (degenerate; no use case)
- Non-Symbol inside `{}` (number, string, keyword, list, vector,
  another struct-pattern) → clean `MalformedForm` naming the
  position + offending shape
- All-Symbol contents → `WatAST::StructPattern(symbols, span)`

The parser does NOT validate field names against any type
registry — that's the consumer's job at check time.

### 4. `src/runtime.rs::parse_let_binding` — third arm

Currently handles `WatAST::Symbol` (single) and `WatAST::Vector`
of Symbols (tuple destructure). Add a third arm:

```rust
WatAST::StructPattern(field_names, span) => {
    LetBinding::StructDestructure { field_names, rhs }
}
```

Where `field_names: Vec<Identifier>` carries the bare-symbol field
names in declaration order.

### 5. `src/runtime.rs::eval_let` (and step_let) — StructDestructure handling

For a `StructDestructure { field_names, rhs }` binding:
1. Evaluate `rhs` to a Value
2. Verify Value is a `Value::Struct(StructValue)` (runtime panic
   on mismatch — arc 098 precedent: type-checker should have
   caught this earlier; defense-in-depth panic acceptable)
3. For each field_name in field_names:
   - Look up the field in the StructValue's field map
   - Bind the field's value to the field_name in the local scope

Edge: a field name that isn't on the struct → check time should
have caught it; runtime panic with diagnostic (mirror arc 098
matches?'s posture).

### 6. `src/check.rs::infer_let` / `process_let_binding` — third arm

Currently handles Symbol + Vector binders. Add StructPattern:

1. Infer rhs's type — must be a `:Type/Struct(StructDef)`.
   Mismatch → `TypeMismatch` with span pointing at rhs.
2. For each field_name in StructPattern's children:
   - Look up the field on the struct's `fields` (StructDef.fields)
   - If not found → `MalformedForm` naming the offending field +
     listing the struct's actual field names (substrate-as-teacher
     pattern)
   - If found → push (field_name, field_type) into the local type
     scope

Mirror the runtime arm's structure exactly so type-check and
runtime stay in lock-step.

### 7. Tests `tests/wat_arc169_struct_destructure.rs`

New file mirroring `tests/wat_arc168_let_flat_shape.rs` shape.
Required cases:

1. **Single field** — `[{outcome} p]` binds `outcome :String`,
   body uses outcome
2. **Multi field** — `[{outcome grace-residue} p]` binds both
3. **Mixed with regular bindings** — `[whole p {outcome residue} p]`
   binds whole + outcome + residue
4. **Nested let** — outer destructure + inner uses the bindings
5. **Field order matches struct declaration** — `[{grace-residue
   outcome} p]` works (bindings can list fields in any order)
6. **Unknown field name** — `[{nonexistent} p]` produces clean
   `MalformedForm` naming the field + listing the struct's actual
   field names
7. **Non-struct subject** — `[{outcome} 42]` produces clean
   `TypeMismatch` (rhs is i64, not a struct)
8. **Empty `{}`** — `[{} p]` produces clean `MalformedForm`
   (degenerate)
9. **Non-Symbol inside `{}`** — `[{42} p]` produces clean
   `MalformedForm` from parser
10. **Multi-form body** — destructure + multi-form body works
    together (regression for arc 168 implicit-do)
11. **Hyphenated field names work** — `grace-residue` binds as a
    legal local

## `wat/core.wat` defn macro

No change needed. `defn` doesn't interact with let-binding shape.

## Discipline reminders

- DO NOT push to main; only push to slice branch
- DO NOT modify slice-1-substrate from arc 167 (Vector mint) or arc
  168 (let flat-shape consumer) — those are settled; mirror their
  shapes
- DO NOT introduce a generic Map node — DESIGN settled on
  StructPattern via four-questions; Map's flexibility is YAGNI per
  user direction 2026-05-09 *"long term strategy (shouldn't need
  to ever change after we make the choice)"*
- DO NOT validate field names at PARSE time — parser only checks
  shape (all-Symbol contents); semantic validation is the
  consumer's job at check / runtime
- DO NOT silently accept non-Struct rhs at runtime — defense-
  in-depth panic acceptable (arc 098 precedent)
- USE the inline cargo+grep+awk verification pipeline (no
  scripts; sonnet-friendly)
- DO NOT pipe `cargo test` through anything beyond the documented
  awk pattern

## FM 5 GUARDRAIL — explicit

If a substrate quirk surfaces (struct registry lookup mechanism
has a gap; field-name resolution requires hooks the substrate
doesn't provide; some path requires a parser-time validation
that conflicts with the "all symbols" rule):

- STOP and report
- DO NOT bridge by special-casing the test
- DO NOT modify substrate code outside the listed scope to "make
  it work"
- The right answer is always: STOP, name the gap, let
  orchestrator decide

## Report shape

When complete, report:

1. Final cargo test summary via the inline pipeline
2. AST tag byte chosen for `TAG_STRUCT_PATTERN`
3. Site count by file for substrate edits
4. Honest deltas — substrate quirks discovered, hidden
   dependencies
5. Test sample output for at least one MalformedForm error
   (paste the actual diagnostic text so we can verify the
   substrate-as-teacher message is clear)
6. Branch state confirmation
7. Actual runtime in minutes vs predicted band

## Time-box

Per EXPECTATIONS-SLICE-1.md (30-90 min predicted, 180 min hard cap).
If you exceed the upper bound still iterating, STOP and report
current state.

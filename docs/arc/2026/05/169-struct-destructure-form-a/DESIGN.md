# Arc 169 — struct-destructure form A in let bindings

**Status:** DESIGN settled via four-questions discipline 2026-05-08
during arc 168 in-flight (slice 4 sonnet sweep running). User
direction: *"let's get our struct destructuring noted in a new
arc — i think we can act on that one quickly later and nothing
depends on it. we'll block 109's closure on it."*

**Blocker:** arc 109 v1 milestone closure depends on this arc
shipping.

## Goal

Mint a struct-destructure form for let bindings that pulls
multiple field values from a struct in one binding-position. The
form is **A** in the four-questions evaluation: `{field1 field2
...}` — bare symbols inside braces, where each symbol is BOTH
the field-name (looked up against the prior binding's struct
type) AND the binding-name in the let scope.

The 12-word rule (user-authored): *"bind the field's value to
the field's name in this scope."*

## End-state shape

```scheme
(:wat::core::struct :test::PaperResolved
  (outcome       :wat::core::String)
  (grace-residue :wat::core::f64))

(:wat::core::define
  (:user::report (p :test::PaperResolved) -> :wat::core::nil)
  (:wat::core::let
    [{outcome grace-residue} p]                    ;; ← arc 169 form A
    (:io/println outcome)
    (:io/println-f64 grace-residue)))
```

The substrate, given `p :test::PaperResolved`:
1. Looks up the struct registry for `:test::PaperResolved` fields
2. For each bare symbol inside `{}`, validates the symbol matches
   a registered field name
3. Emits a field-read; binds the local of the same name into the
   let scope
4. Subsequent body forms see `outcome :String` and
   `grace-residue :f64` as locals

## Settled answers (four-questions)

| Q | Score | Why |
|---|-------|-----|
| Obvious? | YES with 12-word rule | "bind the field's value to the field's name in this scope" — fits in one breath, fully describes the form |
| Simple? | YES | One mode. Each symbol does exactly one thing. |
| Honest? | YES | The form can't lie about renames because it doesn't permit them. Constraint, not deception. |
| Good UX? | Wins common case (no rename) | The rename case has an existing escape hatch: regular let binding with the auto-derived `:Type/field` accessor verb. The destructure form serves the common case beautifully; the verbose path remains for renames. |

Forms B (explicit pairs `{outcome :outcome}`) and C (hybrid) were
rejected during the same conversation: B for ceremonious
duplication; C for failing both Obvious + Simple.

Conversation thread on disk: SCORE-SLICE-3.md "What's next"
section + this DESIGN.md.

## Semantic alignment with arc 098

Arc 098 (`:wat::form::matches?`) introduced a contextual reading:
inside the matches? form, bare keyword `:field` is a NAME-token
that the substrate validates against a registered struct's
fields. The matches? walker emits the field-read; the keyword
doesn't call anything.

Arc 169 form A extends that contextual reading: inside the
`{symbol ...}` shell next to a struct-typed expression, each
bare symbol IS a field-name on that struct's type AND a binding-
name in the let scope.

Where matches? ceremoniously names both the variable and the
field (`(= ?var :field)`), arc 169's same-name discipline
collapses the two into one token. The substrate machinery
underneath is the same: struct-registry lookup + field-read
emission.

This is consistent. Arc 169 doesn't introduce a new substrate
mechanism — it offers a flatter syntactic surface on the
existing field-name semantic.

## Out of scope

- **Rename support** — escape hatch already exists:
  `(:wat::core::let [renamed (:Type/field p)] ...)` uses the
  auto-derived accessor verb. The destructure form does NOT
  need to handle every case; it handles the common case
  beautifully and lets the verbose path serve renames.
- **Nested destructure** — `{outer.inner}` or similar. Future
  arc if/when surfaces; this arc covers single-level only.
- **Default values** — `{outcome :default "none"}`-style. Not in
  scope; struct fields don't have defaults today.
- **Whole-struct rename** — destructure binds fields, not the
  struct itself. To keep the struct, just use a regular let
  binding alongside: `[whole p {outcome residue} p]`.

## Substrate edits (anticipated; concrete sizing happens at slice
spawn time)

### `src/runtime.rs::parse_let_binding`

Currently handles two binder shapes (per arc 168):
- `WatAST::Symbol` → `LetBinding::Single`
- `WatAST::Vector` of Symbols → `LetBinding::Destructure` (tuple)

Add a third arm:
- `WatAST::Map` (or whatever shape the parser yields for `{}`) of
  Symbols → `LetBinding::StructDestructure { fields: Vec<Symbol>, rhs }`

The `LetBinding::StructDestructure` variant carries field names;
the runtime resolves each name against the RHS's struct type at
evaluation, emits a field-read, binds the local.

### `src/lexer.rs` + `src/parser.rs`

Currently `{` and `}` have no token meaning (verified
2026-05-08). Need to mint `LBrace`/`RBrace` tokens + a Map-or-
similar AST variant for `{symbols}`.

The AST shape choice (Map vs StructPattern) wants design-time
attention at slice spawn — the struct-pattern variant might
deserve a dedicated AST node since it's not a generic map.

### `src/check.rs::infer_let` / `process_let_binding`

Mirror runtime: when the binder is the new struct-destructure
shape, look up the struct type registered for the RHS,
validate each field name exists on the struct, push each as a
local of inferred field type.

### Tests `tests/wat_arc169_struct_destructure.rs`

- Single field: `[{outcome} p]` binds `outcome :String`
- Multi field: `[{outcome grace-residue} p]` binds both
- Type-check error: unknown field name produces clean
  `MalformedForm` naming the field + listing struct's actual fields
- Type-check error: wrong-type subject (non-struct) produces
  clean diagnostic
- Multi-binding let: regular destructure mixed with struct
  destructure
- Multi-form body: regression that body forms still see the
  bindings
- Nested let: outer destructure + inner uses the bindings

## Slice plan stub

- Slice 1: substrate consumer (lexer/parser/AST/runtime/check
  + tests)
- Slice 2: closure paperwork (SCORE + INSCRIPTION + 058 row +
  USER-GUIDE update)

Slices may split if substrate work surfaces friction; the four-
questions framework decides at spawn time.

## Cross-references

- Arc 098 (matches?) — the field-name contextual reading
  precedent (`docs/arc/2026/04/098-wat-form-matches/DESIGN.md`)
- Arc 167 (fn-flat-signature) — minted `WatAST::Vector` for
  flat-shape parsing
- Arc 168 (let-flat-shape) — current arc; adds tuple destructure
  via Vector-of-Symbols binder. Arc 169 adds struct destructure
  via Map-of-Symbols binder. The two shapes are parallel.
- Arc 109 (kill-std) — v1 milestone closure depends on arc 169
  shipping

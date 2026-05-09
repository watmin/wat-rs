# Arc 169 — INSCRIPTION

**Inscribed 2026-05-09 by orchestrator.** All slices shipped.

## What shipped

`:wat::core::let` accepts a struct-destructure binder shape
`{field-names...}` that pulls multiple fields from a struct
value into local bindings of the same names:

```scheme
(:wat::core::struct :test::PaperResolved
  (outcome       :wat::core::String)
  (grace-residue :wat::core::f64))

(:wat::core::define
  (:user::report (p :test::PaperResolved) -> :wat::core::nil)
  (:wat::core::let
    [{outcome grace-residue} p]
    (:io::print outcome)
    (:io::print-f64 grace-residue)))
```

The 12-word user-authored rule, captured in DESIGN as the form's
anchor:

> **bind the field's value to the field's name in this scope**

Each bare symbol inside `{}` is BOTH:
- the field name (resolved against TypeEnv struct registry at
  check time)
- the binding name (in the let's local scope)

Unknown field name → check-time `MalformedForm` listing the
struct's actual declared fields. Non-struct subject →
`TypeMismatch`. Empty `{}` → parse-time `MalformedForm`. Non-
Symbol contents inside `{}` → parse-time `MalformedForm`. The
substrate refuses every illegal shape with a clear diagnostic;
the substrate-as-teacher posture is intact.

## Slices

| Slice | Commit(s) | What landed |
|---|---|---|
| 1 | `3c154fc` | `WatAST::StructPattern` AST variant minted; lexer LBrace/RBrace tokens; parser path; `parse_let_binding` third arm; `eval_let` runtime arm; `infer_let` / `process_let_binding` check arm; 11 integration tests |
| 2 | (this commit) | Closure paperwork: SCORE-1 + INSCRIPTION + 058 row + USER-GUIDE update + atomic squash-merge to main |

The slice branch (`arc-169-struct-destructure-form-a`) carries 3
commits since main; main untouched until atomic squash-merge.

## Substrate impact

| Surface | Pre-arc-169 | Post-arc-169 |
|---|---|---|
| `WatAST` variants | 6 (post-arc-167 Vector + base) | 7 (StructPattern added) |
| Lexer tokens | no `{` `}` | `Token::LBrace` / `Token::RBrace` |
| `let` binder shapes | Symbol + Vector | Symbol + Vector + StructPattern |
| Field-by-name destructure | required user-authored verb chains: `[outcome (:Type/outcome p) residue (:Type/grace-residue p)]` | one form: `[{outcome grace-residue} p]` |
| `WatAST::StructPattern` legal positions | n/a | `let` binder position only |
| Hash tag identity | n/a | `TAG_STRUCT_PATTERN = 0x19` distinct from `TAG_LIST = 0x16` and `TAG_VECTOR = 0x18` |
| Workspace test count | 2080/0 (post-arc-168) | 2091/0 (post-arc-169 slice 1; +11 new tests in `tests/wat_arc169_struct_destructure.rs`) |

## Settled design

### Why `WatAST::StructPattern` over generic Map (four-questions, 2026-05-09)

User direction: *"brutal honesty and long term strategy
(shouldn't need to ever change after we make the choice; we
expect we need to evolve over time but we want to engineer the
path that doesn't need an evolution as we made the best starting
choice for longevity)."*

Map: failed Obvious (mental model gap — substrate sees "Map of
N entries"; reader sees "list of fields to destructure"). Failed
Simple (multi-consumer disambiguation by shape inspection isn't
atomic). Borderline failed Honest (asserts speculative future-
use without proven need; YAGNI optimism).

StructPattern: passed Obvious + Simple + Honest cleanly. Each
AST node = one purpose; substrate dispatches on node kind, not
shape inspection.

The Clojure precedent for Map-overload (used for both literals
and destructure-via-keys) is a dynamic-language move that
doesn't translate to a static-substrate's AST kinds — Clojure's
runtime can disambiguate `{}` per use site; wat's check time is
stricter. Each kind = one meaning gives the path that doesn't
need evolution.

### Why bare symbols (form A) over explicit pairs (form B)

User direction 2026-05-08: *"i'm questioning the ergonomics of
B still... it feels like... its ceremonious duplication.... A is
more obvious and friendly to me... 'bind the field's value to
the field's name in this scope'."*

Form A's 12-word rule is shorter to state AND shorter to write
than form B's keyword-pair ceremony. Renames remain available
via existing accessor verbs (`(:wat::core::let [renamed
(:Type/field p)] ...)`); the destructure form serves the common
case beautifully without forcing rename-ceremony at every site.

### Why parse-time shape validation + check-time semantic
validation

Parse time validates *shape*: all-Symbol contents inside `{}`,
non-empty. Anything else → `MalformedForm` from the parser.

Check time validates *semantics*: rhs is a Struct; each field
name exists on the struct's `fields`. Both errors carry navigable
spans + substrate-as-teacher messages.

This split mirrors the substrate's existing posture: parser
errors are syntactic; check errors are semantic. No new pattern
introduced.

### Why nested structures stay single-level

Out of arc 169's scope. A struct field whose value is itself a
struct stays a single binding (not auto-destructured). Arc 169
intentionally does NOT cover nested patterns and does NOT reserve
a number for that work; if a caller surfaces demand, that arc
opens with its own number assigned at start.

### Why renames are not part of this form

Out of arc 169's scope. The auto-derived accessor verb
`:Type/field` already provides clean rename via regular let
binding: `[renamed (:Type/field p)]`. No second syntactic
mechanism needed; no number reserved for one.

## Semantic alignment with arc 098

Arc 098 (`:wat::form::matches?`) introduced a contextual reading
where bare keyword `:field` is a NAME-token resolved against a
registered struct's fields. Arc 169 extends the same field-name
semantic to a flatter syntactic surface in let bindings — bare
SYMBOLS inside `{}` ARE field-names AND binding-names. Same
substrate machinery (TypeEnv struct registry lookup + field-read
emission); flatter user-facing surface.

## FM discipline log

- **No FM 5 incidents.** Opus held the discipline floor cleanly
  throughout slice 1.
- **No FM 10 reach.** Substrate gained a new ENTITY KIND
  (`WatAST::StructPattern` AST variant), not a type-system
  feature. Map-overload-via-shape-inspection was the FM 10 trap
  the four-questions surfaced and rejected.
- **FM 9 honored** — local cargo test re-run between slice 1 ship
  and SCORE write verified 2091/0.
- **FM 11 honored** — pre-INSCRIPTION grep clean.

## Workspace state at INSCRIPTION

`passed: 2091 failed: 0`. Clean.

## Cross-references

- DESIGN.md — settled four-questions evaluation, scope,
  out-of-scope items, slice plan
- BRIEF-SLICE-1.md + EXPECTATIONS-SLICE-1.md + SCORE-SLICE-1.md
  — substrate consumer + walker + tests
- arc 167 INSCRIPTION (`docs/arc/2026/05/167-fn-flat-signature/INSCRIPTION.md`)
  — Vector mint precedent that arc 169 mirrors
- arc 168 INSCRIPTION (`docs/arc/2026/05/168-let-flat-shape/INSCRIPTION.md`)
  — let consumer shape that arc 169 extends with the third
  binder arm
- arc 098 DESIGN (`docs/arc/2026/04/098-wat-form-matches/DESIGN.md`)
  — field-name contextual reading precedent
- arc 109 INVENTORY § M (`docs/arc/2026/04/109-kill-std/INVENTORY.md`)
  — arc 169 v1-closure-blocker tracker; this INSCRIPTION
  unblocks arc 109 v1 milestone closure

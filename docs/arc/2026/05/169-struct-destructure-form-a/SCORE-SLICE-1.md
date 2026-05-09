# Arc 169 slice 1 — SCORE

Substrate consumer + walker + tests for struct-destructure form
A in let bindings. Mode A clean, ~70 min opus (within 30-90 min
predicted band; same order as arc 167 slice 1 Vector mint
precedent). Branch `arc-169-struct-destructure-form-a` carries
slice 1 commit `3c154fc` + this SCORE.

## Scope as shipped

`WatAST::StructPattern(Vec<WatAST>, Span)` minted as a first-
class substrate AST node. Lexer mints `LBrace`/`RBrace` tokens.
Parser produces StructPattern from `{symbols}`. The let consumer
gains a third binding-shape arm:

```scheme
(:wat::core::let
  [{outcome grace-residue} p]
  (:io::print outcome))
```

The 12-word user-authored rule:

> bind the field's value to the field's name in this scope

Each bare symbol is BOTH:
- the field name (resolved against TypeEnv struct registry at
  check time)
- the binding name (in the let's local scope)

### Substrate site count (per opus's report)

| File | Sites |
|---|---|
| `src/lexer.rs` | 4 (Token variants + lex char arms + symbol-break + keyword-body brace handling) |
| `src/ast.rs` | 3 (variant + span() arm + struct_pattern() ctor) |
| `src/hash.rs` | 2 (TAG_STRUCT_PATTERN const + write_canonical_wat arm) |
| `src/parser.rs` | 5 (ParseError variants + Display arms + parse arm + parse_brace_body + cross-delimiter mismatch arms) |
| `src/runtime.rs` | 8 (eval, ast_variant_name, step_form, try_recognize_holon_value, watast_to_holon, try_match_pattern, try_match_pattern_ast, parse_let_binding + bind_let_binding + LetBinding enum extension) |
| `src/check.rs` | 7 (infer's value-position arm, check_subpattern, ast_variant_name_check, infer_let::binding_names extraction, process_let_binding StructPattern arm, infer_make_channel error formatter, check_let_for_scope_deadlock_inferred binding_names extraction) |
| `src/types.rs` | 1 (ast_variant_name) |
| `src/config.rs` | 1 (variant_name) |
| `src/load.rs` | 1 (variant_name) |
| `src/lower.rs` | 1 (lower's StructPattern arm) |
| `src/macros.rs` | 1 (ast_variant_name) |
| `tests/wat_arc169_struct_destructure.rs` | 11 tests (new file) |

### Tag byte

`TAG_STRUCT_PATTERN: u8 = 0x19`. Distinct from `TAG_LIST = 0x16`
+ `TAG_VECTOR = 0x18`. Hash identity preserved across the three
delimiter shapes.

### Test sample MalformedForm output (substrate-as-teacher)

For `[{nonexistent} p]` against `:test::PaperResolved (outcome
:String) (grace-residue :f64)`:

```
check:
1 type-check error(s):
  - <entry>:10:14: malformed :wat::core::let form: struct-destructure: field "nonexistent" is not declared on struct :test::PaperResolved (declared fields: outcome, grace-residue)
```

Substrate-as-teacher property held: names the offending field,
lists declared fields, points at a navigable span.

## Scorecard

| Row | Verified by | Pass |
|-----|-------------|------|
| A — `WatAST::StructPattern` variant minted | grep `WatAST::StructPattern` src/ast.rs: variant exists with `Vec<WatAST>` + `Span` fields | ✓ |
| B — `TAG_STRUCT_PATTERN` distinct tag | tag byte `0x19`; distinct from existing tags | ✓ |
| C — Lexer mints `LBrace`/`RBrace` tokens | `{` and `}` produce Token::LBrace / Token::RBrace | ✓ |
| D — Parser produces StructPattern from `{...}` | `{outcome residue}` parses to `WatAST::StructPattern([Symbol, Symbol], span)` | ✓ |
| E — Empty `{}` → clean MalformedForm | parser rejects with diagnostic naming the position | ✓ |
| F — Non-Symbol inside `{}` → clean MalformedForm | non-Symbol content rejected at parse time | ✓ |
| G — `parse_let_binding` StructPattern arm | third arm added; produces `LetBinding::StructDestructure { field_names, rhs }` | ✓ |
| H — `eval_let` / `step_let` runtime arm | each field name resolved against struct value; bindings emitted into local scope | ✓ |
| I — `infer_let` / `process_let_binding` check arm | rhs must be Struct; each field validated; unknown field → MalformedForm; type-mismatch on rhs → TypeMismatch | ✓ |
| J — All 11 test cases pass | tests/wat_arc169_struct_destructure.rs 11/11 green | ✓ |
| K — Workspace stays clean | post-slice-1 verified locally: `passed: 2091 failed: 0` (was 2080/0 pre-slice; +11 = +11 tests, all green) | ✓ |
| L — Slice branch on remote | branch carries `3c154fc`; main untouched | ✓ |
| M — `wat/core.wat` defn macro untouched | git diff shows no changes | ✓ |
| N — No FM 10 reach | substrate gains a NEW ENTITY KIND (`WatAST::StructPattern` AST variant); no type-system feature added; no Map-overload-via-shape-inspection bridge | ✓ |

## Honest deltas

**None.** No FM 5 traps surfaced; no FM 10 reach.

Two minor observations opus flagged (not blockers):

- `register_value` / pretty-print in `runtime.rs:12700-area`
  renders `Value::Struct` independently of `WatAST::StructPattern`.
  The brace pretty-print already exists for struct values; no
  `render_value` sweep needed for arc 169.
- Macro-template walking (`expand_form`, `walk_template`,
  `substitute_bindings`) doesn't recurse into `StructPattern`
  children. For arc 169 slice 1's scope this is correct (brace-
  forms are user-source binders, not macro-expanded targets). If
  a future arc needs macro-introduced field names, those walkers
  will need a recursion arm — affirmatively out of arc 169 scope;
  no current caller surfaces this need.

The TypeEnv's `get(name) -> Option<&TypeDef>` plus
`StructDef.fields: Vec<(String, TypeExpr)>` was the canonical
lookup mechanism — already exposed, no new infrastructure needed.

## Calibration row

| Predicted | Actual | Mode |
|-----------|--------|------|
| 30-90 min opus, 180 min hard cap | ~70 min | A clean (within band) |

Mode A clean: deletion-list-like mechanical work; compiler's
E0004 non-exhaustive match errors gave a precise punch-list of
consumer sites; mirroring of arc 167's Vector mint pattern with
arc 169-specific consumer wiring. No detours.

## Discipline check

- ✓ FM 5 caught + held (no detours)
- ✓ FM 9 honored — local cargo test verified 2091/0 post-spawn
- ✓ FM 10 honored — substrate gained a new AST entity kind, not
  a type-system feature; no Map-overload bridge
- ✓ FM 11 — pre-INSCRIPTION grep deferred to slice 2 closure
- ✓ FM 16 honored — BRIEF didn't preempt tool availability
- ✓ Branch isolation held — main untouched

## What's next

Slice 2 — closure paperwork (orchestrator-side):
- INSCRIPTION (this arc's closure record)
- 058 changelog row (FOUNDATION-CHANGELOG.md in trading lab)
- USER-GUIDE update (let section gets struct-destructure example)
- Atomic squash-merge to main as one squash commit

When slice 2 ships, arc 109 v1 milestone closure unblocks per
arc 109 INVENTORY § M.

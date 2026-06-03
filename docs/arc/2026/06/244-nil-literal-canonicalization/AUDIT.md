# Arc 244.0 — AUDIT: the nil lifecycle, the synthesis heresy, the NilLit cascade scope

Read-only scoping crawl. Every claim cites a current-working-tree `file:line`. The implementation is Stone 244.1+ (NilLit mint + cascade + sweep) and 244.3 (gate). The per-`match`-arm cascade is compiler-enumerated at 244.1 build time; this audit captures the **semantic** scope the compiler can't see, plus a near-complete arm map.

## 1. Current bare-nil lifecycle

- **Lex/parse:** the lexer (`src/lexer.rs:377-387`) has NO nil case — `match sym.as_str()` only tests `"true"`/`"false"`; `nil` falls through to `Token::Symbol("nil")`. The parser (`src/parser.rs:315`) maps it to `WatAST::Symbol(Identifier::bare("nil"), span)`. **Bare `nil` → `Symbol("nil")` today** (because NilLit doesn't exist yet).
- **Check:** `infer_expr`'s generic `WatAST::Symbol` arm (`src/check.rs:3394-3401`) returns a `fresh.fresh()` var for `nil` (not a user local); the type lands by unification with the declared return type (`src/function/infer.rs:127`), which `reduce`/`expand_alias` (`src/types.rs:645-649`, `2554-2558`) collapses `:wat::core::nil` → `Tuple([])`. Inference is **indirect** (fresh-var unify), not a direct nil-type result.
- **Eval:** `src/runtime.rs:5121-5123` — `WatAST::Symbol(ident,_) if ident.as_str()=="nil" => Value::Unit`. The special eval arm (Stone 242.2).

## 2. "Symbol-nil-is-special" handling (migrates to NilLit)

- `runtime.rs:5121` — eval `Symbol("nil") → Value::Unit`. After the parser emits NilLit, this becomes dead for user source; keep it coexisting until the 244.2 sweep retires the last `Symbol("nil")` synthesis, then retire (244.4/N).
- `closure_extract.rs:1549` — `Value::Unit => WatAST::Symbol(Identifier::bare("nil"), span)` (the value-form synthesis; correct today, migrates to `NilLit`).
- `runtime.rs:15048`, `runtime.rs:16669` — HolonAST→Value (`if s=="nil" => Value::Unit`); operate on HolonAST, NOT WatAST. **Untouched.**

## 3. The synthesis sweep — 9 sites (Stone 244.2)

All are VALUE positions (function/let bodies, round-trip values). `ret_type` fields stay `TypeExpr::Tuple([])`.

| # | Site | Now | → | What it is |
|---|---|---|---|---|
| 1 | `closure_extract.rs:1994` | `Keyword(":wat::core::nil")` | `NilLit(span)` | `strip_do_prelude` residual body (all-prelude do) |
| 2 | `runtime.rs:2771` | `Keyword(...)` | `nil()` | **defclause pre-register stub body — THE confirmed failing path** |
| 3 | `runtime.rs:3739` | `Keyword(...)` | `nil()` | defalias unknown-target stub body |
| 4 | `runtime.rs:6548` | `Keyword(...)` | `nil()` | `synthesize_fn_body([])` empty-body singleton |
| 5 | `runtime.rs:17893` | `Keyword(...)` | `NilLit(Span::unknown())` | `holon_to_watast` round-trip (HolonAST nil → WatAST) |
| 6 | `runtime.rs:25523` | `Keyword(...)` | `NilLit(outer_span)` | `synthesize_let_body([])` empty-body |
| 7 | `check.rs:7442` | `Keyword(...)` | `nil()` | `infer_let` empty-body (fed to infer_expr → fires Doctrine 1) |
| 8 | `closure_extract.rs:1549` | `Symbol("nil")` | `NilLit(span)` | `encode_value_with_path` `Value::Unit` (already value-form; align to canonical) |
| 9 | `runtime.rs:11987-12001` | *(missing arm)* | add `Value::Unit => NilLit(span)` | `value_to_watast` — latent gap (quasiquote `~nil` errors today) |

## 4. The gate's legit-vs-heresy line (Stone 244.3)

- **Legit:** `WatAST::Keyword(":wat::core::nil")` from user source via `src/parser.rs:314` / `src/lexer.rs`. EXEMPT.
- **Heresy:** every `WatAST::Keyword(":wat::core::nil".into()/.to_string(), …)` construction in `src/` *outside* parser/lexer. Type annotations in synthesis use `TypeExpr` (e.g. `ret_type: TypeExpr::Tuple([])` at `runtime.rs:2777`/`3745`), never `WatAST::Keyword` — so a literal `Keyword(":wat::core::nil")` construction in synthesis is ALWAYS the value-heresy.
- **One nuance:** `closure_extract.rs:1564` (+1560/1605/1640/1852) build `WatAST::Keyword(elem_kw, …)` where `elem_kw` is a string VARIABLE landing in a **type-parameter** position of a synthesized `(:wat::core::Vector T …)` call — structurally sound (type position). The gate targets the literal `":wat::core::nil"` string in `Keyword(...)` constructions; the variable-driven type-param cases are not literal matches and remain sound.
- Gate scope: a build-failing test grepping `src/` (excluding `parser.rs`/`lexer.rs`) for `Keyword(":wat::core::nil"` literal constructions; expected count 0 after the sweep.

## 5. The `ast.rs` impl cascade (Stone 244.1)

`WatAST::NilLit(Span)` — a leaf literal joining `IntLit/FloatLit/BoolLit/StringLit`.
- `span()` (`ast.rs:104`): add `NilLit(s) => s` to the chain.
- `variant_name()` (`ast.rs:201`): add `NilLit(_, _) => "nil"`. *(note: NilLit has ONE field (Span); arm is `NilLit(_)` — reconcile arity in impl.)*
- `impl Hash` (`ast.rs:244`): add `NilLit(_) => {}` (discriminant already hashed).
- `children()` (`ast.rs:187`): leaf → falls to `_ => &[]`; no arm needed (note in comment).
- `PartialEq`/`Debug`/`Clone`: derived — auto.
- Add a `WatAST::nil()` constructor (the value-family's missing member; `Span::unknown()`), beside `int/float/bool/string`. Plus allow span-carrying construction (`NilLit(span)`) at sweep sites that have a real span.
- **Parser:** `src/parser.rs` (or lexer) emits `NilLit(span)` for bare `nil` instead of `Symbol("nil")`.

## 5a. Downstream exhaustive matches (compiler-enumerated at 244.1 build; pre-mapped here)

| File:line | Arm |
|---|---|
| `check.rs:3284` (`infer_expr`) | `NilLit(_) => CheckResult::ok(TypeExpr::Path(":wat::core::nil".into()))` |
| `check.rs:6585` (match-pattern checker) | `NilLit(_) => nil-type expected? Some(false) : type-error` |
| `check.rs:10704` (error-msg inline) | `NilLit(..) => "nil"` |
| `runtime.rs:4998` (`eval_inner`) | `NilLit(span) => Value::Unit (Provenance::Literal{span})` |
| `hash.rs:124` (`write_canonical_wat`) | `NilLit(_) => out.push(TAG_NIL)` (new `TAG_NIL` constant) |
| `config.rs:614`, `load.rs:857` (local variant_name) | `NilLit(..) => "nil literal"` |
| `lower.rs:155` | `NilLit(_, span) => Err(LowerError{...})` (bare literal unsupported in lower context, like its siblings) |
| `macros.rs:646` (`ast_kind`) | `NilLit(..) => "nil-literal"` |
| `closure_extract.rs:524/975/2036` (walk_free_symbols / collect_pattern_bindings / rewrite_with_scope) | leaf → `Ok(())` / `node.clone()` (join the literal group) |

Catch-all (`_ =>`) matches elsewhere absorb NilLit automatically.

## 6. Risks (all manageable)

1. **Empty-body type inference preserved:** `synthesize_fn_body([]) → NilLit → infer_expr → Path(:wat::core::nil) → unify with `-> :wat::core::nil` ret → both reduce to `Tuple([])` → succeeds.** No regression.
2. **Dead `Symbol("nil")` eval arm:** after parser→NilLit + sweep, `runtime.rs:5121` is dead for user source. Coexist 244.1→244.2; retire at 244.4/N (do not remove before the sweep lands).
3. **`value_to_watast` (`runtime.rs:11987`) missing `Value::Unit`:** pre-existing latent gap (quasiquote `~nil`); add `Value::Unit => NilLit(span)` in the sweep.
4. **`holon_to_watast` round-trip** (`HolonAST::nil() → NilLit → Value::Unit → HolonAST::nil()` via `value_to_holon` `runtime.rs:24530`): intact.
5. **Canonical hash (`hash.rs` TAG_NIL):** AST containing nil now hashes differently (Symbol→NilLit). No known signed source carries bare `nil` in value position; low practical risk, note it.
6. **Macros/quasiquote:** `walk_template` (`macros.rs:847+`) + `substitute_bindings` pass leaves through via clone; NilLit is a leaf → unaffected.
7. **`tests/` constructions:** `tests/probe_sender_receiver_from_pipe.rs:193` builds `Keyword(":wat::core::nil")` for a TYPE arg — outside `src/`, outside the gate; leave.

## Verification gates (Stone 244.4)

- `tests/probe_nil_return_value_position_bug.rs` 4/4 green (the locked repro).
- The synthetic `:wat::core::nil` value-position error gone from `tests/probe_arc237_8b_defclause_arithmetic.rs` (its remaining failures should now be ONLY the genuine 237.8b work — `&`/recipe — not the nil heresy).
- `cargo test --release --lib -p wat` green (895/0/1 maintained).
- `cargo clippy` clean.
- The 244.3 gate: 0 literal `Keyword(":wat::core::nil"` constructions in `src/` outside parser/lexer.

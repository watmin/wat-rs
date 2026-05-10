# Arc 173 — Clojure macro feature parity

**Status:** opened 2026-05-10. Promoted from arc 172's slice 3
per user direction *"make an arc for this - do not forget."*

**Predecessor:** arc 172 (Scheme → Clojure macro flavor swap;
slices 1+2 shipped at `82d37c3` as atomic-commit pair). Arc
172 closed the lexical pivot (`,`/`,@` → `~`/`~@`); arc 173
adds the Clojure macro NICETIES that complete feature parity
with Clojure's reference implementation.

## Motivation

Per arc 170 REALIZATIONS pass 14 (locked 2026-05-10):

> *"Clojure's `'foo` quote, `` `foo `` syntax-quote with
> auto-namespace + auto-gensym-on-`#`, `~foo` unquote,
> `~@foo` unquote-splicing, `gensym`, `&form`/`&env`"*

Arc 172 shipped the unquote / splicing source-syntax. Arc
173 ships the rest:

1. **Auto-gensym** — `name#` inside a syntax-quote scope
   resolves to a fresh symbol; identical references within
   the same syntax-quote share the gensym; cross-syntax-quote
   references are independent
2. **`&form` / `&env`** — implicit macro arguments populated
   at expansion time; `&form` is the calling form (the macro
   invocation as data); `&env` is the lexical environment
3. **`gensym`** — manual fresh-symbol primitive callable
   from any context (not just syntax-quote scope)

These features make wat's macro layer match Clojure's
contract — a wat author who knows Clojure macros can
transfer expectations directly.

## Why arc 173, not arc 172 slice 3

Arc 172 slices 1+2 delivered the EDN-compliance contract
(commas as whitespace; `~`/`~@` as unquote tokens). That's
ONE coherent change.

Arc 173 delivers macro-feature ADDITIONS (auto-gensym,
implicit args, primitive). That's a DIFFERENT coherent
change. Promoting it from "arc 172 slice 3" to its own arc:

1. **Lets each arc deliver clean closure separately.** Arc
   172's INSCRIPTION shipped 2026-05-10; arc 173 ships on
   its own terms when authored.
2. **Honors the four-questions discipline.** Arc 172 was
   "swap the syntax." Arc 173 is "add the features."
   Different concerns; honest naming.
3. **Stops the deferral language** in arc 172's INSCRIPTION
   from being read as a queued-but-unnamed follow-up. Arc
   173 gives it a number + dir + DESIGN.

## Scope

### Substrate edits

**Lexer** (`src/lexer.rs`):
- Recognize `name#` (trailing `#`) as auto-gensym in
  syntax-quote scope. Likely a new `Token::AutoGensym(name)`
  variant or pass through to the parser as part of the
  symbol grammar.
- The `#` suffix is only meaningful WITHIN a backtick
  quasiquote scope; outside it should fall back to the
  symbol parsing rules currently in place (verify lexer-
  internal precedent; bare `name#` outside backtick may
  already be valid for some other reason — surface as
  honest delta).

**Parser** (`src/parser.rs`):
- Parse `name#` according to the Token shape

**Macro evaluator** (`src/macros.rs`):
- Auto-gensym table per syntax-quote scope: every `name#`
  inside one syntax-quote shares one fresh symbol; nested
  syntax-quotes get their own table per arc 029 depth
  semantics
- `&form` populated as the macro invocation form (the
  caller's source AST)
- `&env` populated as the calling lexical environment (the
  symbols visible at the call site)
- These implicit args bind alongside the user-declared
  defmacro args at expansion time

**`gensym` primitive**:
- Eval arm in `src/runtime.rs` for `(:wat::core::gensym
  prefix-string)` → fresh symbol with that prefix
- Type-check arm in `src/check.rs`: `:wat::core::String ->
  :wat::core::Symbol` (verify the wat-side Symbol type
  representation)

### Migration

Existing macros migrate to use auto-gensym where they
manually managed symbol-uniqueness:
- Audit `wat/test.wat`'s deftest / deftest-hermetic /
  make-deftest / make-deftest-hermetic for spots where they
  reference the macro's parameter names verbatim — those
  are POTENTIAL gensym sites if the parameter is meant to
  not collide with user code. Surface as honest delta during
  authoring.
- `wat/runtime.wat::define-alias` may also benefit.
- Other `wat/*.wat` macros — audit at slice author time.

### What does NOT change

- Source-level `~` / `~@` unquote tokens (arc 172's contract)
- Macro evaluator's depth tracking per arc 029
- `:wat::core::quote` / `:wat::core::unquote` /
  `:wat::core::unquote-splicing` reader macros — the AST shapes
  produced by the lexer/parser stay the same; only `name#` is
  new
- `'foo` quote shorthand — arc 172 confirmed no collision
  with apostrophe-in-keyword; verify still holds for
  `'name#` shape

## Slicing plan

Predicted: 2-4 hours total mixed opus + sonnet across
multiple slices.

### Slice 1 — `gensym` primitive (atomic; standalone)

Smallest piece; no syntax-quote dependency. Mints
`(:wat::core::gensym prefix)` → fresh `:wat::core::Symbol`.
Eval + type-check arms; tests.

Sonnet. ~30-60 min.

### Slice 2 — Auto-gensym lexer + parser + evaluator

Lexer recognizes `name#` token; parser carries through;
macro evaluator's expand_template adds per-syntax-quote
gensym table.

Opus + sonnet. ~90-150 min.

### Slice 3 — `&form` / `&env` implicit macro args

Macro evaluator populates these when binding the macro's
formal args at expansion time. Tests + USER-GUIDE update.

Opus. ~60-90 min.

### Slice 4 — Migration sweep + closure

Audit existing macros; migrate any that benefit from
auto-gensym. Tests; INSCRIPTION + 058 row + USER-GUIDE.

Sonnet + orchestrator. ~30-60 min.

## Dependencies

- Arc 172 slices 1+2 SHIPPED (`82d37c3`) — `~`/`~@` already
  the canonical unquote tokens; this arc builds atop the
  Clojure-syntax foundation.
- No collision with arc 170 slice 1f-* services work; this
  arc is orthogonal.

## Risks

- **`name#` lexer-level collision** — if any existing wat
  source uses `name#` for a non-gensym purpose, that breaks.
  Pre-grep at slice 2 author time to enumerate.
- **`&form` / `&env` naming collision** — `&` is not
  currently a special character in wat. If it's used in any
  existing symbols, surface.
- **Auto-gensym scope bookkeeping in nested syntax-quotes**
  per arc 029 depth semantics — the macro evaluator has to
  track depth correctly. Test thoroughly.

## Ship criteria (whole-arc)

- `(:wat::core::gensym prefix)` produces fresh symbols at
  runtime
- `name#` inside syntax-quote scope produces a gensym;
  identical references share; nested syntax-quotes
  independent
- `&form` and `&env` implicit args available in defmacro
  bodies
- Existing macros that benefit migrated (sweep)
- Workspace at baseline (or +N for new tests; 0 regression)
- Cargo check + cargo test green
- INSCRIPTION per FM 11 (no deferral language)

## Cross-references

- Arc 170 REALIZATIONS pass 14 — the parent decision (named
  these features explicitly)
- Arc 172 (`82d37c3` — slices 1+2) — the syntax-pivot
  predecessor; INSCRIPTION names slice 3 / arc 173 as the
  follow-on
- Arc 029 (nested-quasiquote) — depth-tracking semantics
  this arc preserves + extends to gensym scope
- Arc 010 (variadic-quote) — quasiquote primitive precedent
- Memory: `feedback_apostrophe_dispatch_separator.md`
  (sibling EDN-compliance lock-in; unchanged)
- Memory: `project_arc_173_clojure_macro_features.md`
  (this arc's tracker; created 2026-05-10)

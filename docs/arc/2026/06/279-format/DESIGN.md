# Arc 279 — `format`: the opinionated named-template printf (a macro over `concat`)

> **STATUS: SHIPPED (`d117c54e`, 2026-06-17).** `format` macro in `wat/core.wat` + the `:wat::core::str`
> intrinsic (unquoted polymorphic render — a rubric-grow) + `eval_ast_name` reads StringLit. Weighed:
> probe 3/3, 5 format deftests, deporder gate 0, lib 929/36. **The literal-brace escape — RESOLVED:
> `{{`/`}}` doubling (Rust/Python convention), NOT `\{`.** The `\{` route hit a real lexer STOP (`\` is
> the lexer's escape trigger, so `\{` → `UnknownEscape` in `src/lexer.rs lex_string`), but `{{`/`}}`
> needs **zero lexer change**: `{`/`}` are ordinary string chars, so `"{{foo}}"` lexes straight through
> as literal text and the **format macro** collapses `{{`→`{` / `}}`→`}` at expand time. The escape is
> pure macro-grammar. (The earlier "deferred on the lexer" framing over-stated the blocker — the
> no-lexer route was always available.) **Follow stones (next, in order):** (1) **`{{`/`}}` literal-brace
> handling** in the format macro's template parser (a small addition — collapse doubled braces, treat a
> single `{`/`}` mid-template that isn't a placeholder as an error; the macro currently guards against
> `{`/`"` in templates, which this supersedes); (2) the **concat-abuse lint rule** (a 277 stone) that
> detects a literals+values `concat`-chain and suggests `format` — the self-fixing-toolchain's RULE half
> for this tool.
>
> Opened 2026-06-17. The first TOOL of the self-fixing-toolchain doctrine
> (SELF-FIXING-TOOLCHAIN.md): the `string::concat`-chain interleaving literals and values is awful
> (proven explicitly by `wat/lint.wat`'s `violation->finding`); `format` is the cure. It ships first as
> the tool; the concat-abuse lint *rule* that suggests it is a later 277 stone (report-only until
> `ast-end-span`).

## The shape (builder-specified, opinionated, no config)

```clojure
(:wat::core::format
  "{greeting}, {name}! you have {count} messages"
  :name "ada" :greeting "hello" :count 3)
;=> "hello, ada! you have 3 messages"
```

- **NAMED placeholders** `{name}` — never positional `{}`. The template documents itself.
- **Filled by trailing `:name val` kwarg pairs** — a kwarg-hash, out-of-order allowed.
- **Rendered UNQUOTED** — a String fills as itself (`ada`, not `"ada"`), an i64 as its digits, etc.
  (display, not EDN `show`).
- **`\{` escapes to a literal brace** — `"\{not-a-placeholder}"` → `{not-a-placeholder}`.
- **Strict + opinionated** — every `{name}` MUST have a matching `:name` (else macro-error); every
  `:name` MUST be consumed by a placeholder (else macro-error). No config, no flavors, no silent skips.

## It is a MACRO (the kwargs doctrine)

A named-kwarg template is a kwargs surface, and kwargs-is-always-a-macro: `format` parses the template
**at expand time** and emits a lean `(:wat::core::string::concat <static> (<display> val) <static> …)`.
The template, the `{name}`s, and the labels **evaporate** — the runtime sees only `concat` + display
calls; zero runtime template-parsing cost; the gross concat is *generated*, never written. Home:
`:wat::core::format` in `wat/core.wat` (a core macro, beside `cond`/`->` and the defn-kwargs branch).

## Feasibility (grounded) + the rubric for grows

The macro-eval allow-list (`is_pure_total`, `src/macros/eval.rs`) already carries the template-parsing
ops: `string::split`, `concat`, `contains?`, `starts-with?`/`ends-with?`, `length`, `join`, plus
`keyword/to-string` + the `*::to-string` family. So the expand-time parser is feasible TODAY. **Rubric
(builder, blessed): "does a macro need it?" — if the parser needs an op not on the allow-list
(`subs`/`index-of`/`char-at`), ADD it as a macro-eval intrinsic.** That is the correct boundary, not a
blocker. (STOP-to-report only if a grow turns out bigger than a thin intrinsic.)

## Two things to GROUND FIRST (they shape the contract)

1. **The wat string lexer's `\{` handling.** For `\{` to reach the template parser as an escape, the
   lexer must PRESERVE `\{` (the two chars) in the string value. Ground it (`src/parser.rs` / the
   tokenizer's string-escape handling): does `\{` pass through, or does the lexer eat/reject it? If it
   passes through → implement `\{` → literal `{`. If the lexer eats it (can't distinguish escaped from
   placeholder) → **STOP and report**; we decide `{{` (Rust/Python doubling) vs a lexer change. Do NOT
   guess.
2. **The unquoted-display fn.** `format` emits `(<display> val)` per placeholder — a RUNTIME fn that
   renders ANY value unquoted (String→itself, i64→digits, bool→`true`/`false`, …). Ground what exists
   (`show` quotes strings — wrong; look for `str`/`to-string`/a Display dispatch). If none renders
   unquoted polymorphically, ADD a thin `:wat::core::str` (or `display`) — "does a macro need it?": yes,
   to render the substituted value. Report which you used.

## Implementation sketch

1. `(:wat::core::defmacro :wat::core::format [tmpl <- :wat::WatAST & opts <- :wat::core::Vector<wat::WatAST>] -> :wat::WatAST …)`
2. Extract the template string literal from `tmpl` (`ast-name` / the string value). If not a literal →
   macro-error ("format template must be a string literal").
3. Fold the trailing `:name val` pairs into a name→AST map (the defservice opts-map pattern,
   `wat/service.wat` ~67 — `known-opts` becomes "the placeholder names", validated strictly).
4. Parse the template into an alternating sequence of [static-text, placeholder-name, static-text, …]
   honoring `\{` escapes (via `string::split` on `{`/`}` + escape handling, or a char walk if you grow
   `subs`/`index-of`).
5. Emit `(:wat::core::string::concat <static-lit> (<display> <val-ast>) <static-lit> …)`. Empty statics
   may be elided.
6. Strict checks: a `{name}` with no `:name` → macro-error naming the placeholder; a `:name` not in the
   template → macro-error naming the unused kwarg.

## Rooms

- `wat/service.wat` ~55-110 — the expand-time kwargs-fold + `known-opts` + `macro-error` pattern (copy it).
- `wat/core.wat` — the existing macros (`cond`, `->`, the defn-kwargs branch ~266) for the
  defmacro-emitting-forms shape + expand-time string ops.
- `src/macros/eval.rs` `is_pure_total` (~344) — the allow-list; extend here if a string op is needed.
- `src/parser.rs` — the string-escape handling (ground point #1).

## Proof (wat deftests, `wat-tests/format.wat`)

1. Named substitution, out-of-order, heterogeneous: `(format "{a} {b}" :b 5 :a "x")` → `"x 5"`.
2. Unquoted: a String fills without EDN quotes; an i64 as digits.
3. Strict: missing `:name` → macro-error; unused `:name` → macro-error. (deftest the failure path per
   the test convention.)
4. Escape (if lexer preserves `\{`): `(format "\{lit} {x}" :x "y")` → `"{lit} y"`.
5. The Rust gate `tests/probe_arc279_format.rs` (un-ignore) goes green.

## Blast radius

- EDIT `wat/core.wat` (the `format` macro). EDIT `src/macros/eval.rs` only if a string op grow is needed.
  Maybe ADD a thin `:wat::core::str`/`display` (ground point #2). NEW `wat-tests/format.wat`. Un-ignore
  the probe. Nothing else.

## Four questions

- **Obvious?** YES — `(format "{name}" :name v)` reads like the sentence it produces.
- **Simple?** YES — one macro; it compiles to `concat`; no runtime template engine.
- **Honest?** YES — strict matching (no silent missing/unused), unquoted display is the truthful render,
  the template must be a literal (no hidden runtime parse).
- **Good UX?** YES — named + out-of-order + self-documenting; one correct behavior, no config; the right
  path is the only path.

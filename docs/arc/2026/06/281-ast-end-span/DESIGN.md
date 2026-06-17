# Arc 281 — `ast-end-span`: node end-position tracking (the structural-auto-fix keystone)

> **STATUS: STRIKE-READY (2026-06-17).** RED probe `tests/probe_arc281_ast_end_span.rs` (`#[ignore]`'d):
> `ast-end-span` is `UnknownFunction` at HEAD. The keystone that unblocks EVERY structural auto-fix —
> 277.1b (ladder fix), the concat→format fix (277.1c is report-only TODAY waiting on this), and the
> sweep. Arc-scale: lexer → SpannedToken → Span → parser → intrinsic.

## Why (the gap, grounded)

`fix.wat` already turns a `{:line, :col}` span into a flat char offset (`fix-text-offset-of`,
`fix.wat:148`: `offset = line-start(line) + (col-1)`), and computes a deletion/replacement `old-len`
as `(string::length (ast-name node))` (`fix.wat:167,186`). That works for an **atom** (old-len = the
token's char length) but **NOT for a structural node** (a whole `(...)` form): `ast-name` doesn't yield
the source extent of a list, and `ast-span` returns only the START (`edn_shim.rs:506-548`,
`Span{file,line,col}` at `span.rs:48-55`). So a rule that wants to replace a whole form (the if-ladder,
the concat-chain) cannot compute `old-len = end-offset - start-offset` — the END is unknown.

`ast-end-span` returns the node's END `{:line, :col}` (one char PAST the last char of the node — for
`(a b c)`, col 8, just after the `)`). Then `old-len = offset-of(ast-end-span) - offset-of(ast-span)`,
and the fix replaces exactly `[start, end)`.

## The mechanism (THE CONTRACT)

End-position flows from the lexer (which tracks char position precisely) up through the parser:

### 1. `Span` gains an end (additive, `src/span.rs`)
```rust
pub struct Span { file: Arc<String>, line: i64, col: i64, end_line: i64, end_col: i64 }
```
- `Span::new(file, line, col)` keeps its signature; sets `end_line = line, end_col = col` (degenerate
  "end == start" — every existing call-site keeps compiling unchanged; error spans don't care about end).
- Add `Span::with_end(file, line, col, end_line, end_col)` for the lexer/parser to set a real end.
- `is_unknown` unchanged (line/col == 0). `Display` unchanged (start only).
- The ~15 `Span::new` / `Span { … }` sites (`span.rs`, `lexer.rs`, `parser.rs`, `panic_hook.rs`,
  `macros/expand.rs`, `function/parse.rs`, `check.rs`, `ast.rs`) compile as-is via the default.

### 2. Lexer stamps each token's end (`src/lexer.rs`)
The lexer scans `src` by byte index and has `span_at(i)` (`:248`) → `{line, col}` for index `i`
(via `:443`). Each token covers `[start_i, end_i)`; stamp its span with both:
`Span::with_end(file, start.line, start.col, end.line, end.col)` where `end = span_at(end_i)` and
`end_i` is the index one past the token (i+1 for single-char delims like `(`/`)`; the scan-end index
for multi-char atoms/strings). `SpannedToken.span` now carries start AND end.

### 3. Parser combines open..close (`src/parser.rs`)
- **Atom nodes** (Symbol/Keyword/String/Int/Bool/Char in `parse_form`, `:200-264`): use the token's
  span directly — it already carries start+end from the lexer.
- **Structural nodes** (List/Vector/Map/Set): the body parsers (`parse_list_body :290`,
  `parse_vector_body :327`, `parse_brace_body :368`) consume the closing delimiter (`:294`, `:331`,
  `:372`). Thread the **close token's span** back so `parse_form` builds the node span as
  `Span::with_end(open.file, open.line, open.col, close.end_line, close.end_col)` — start at the `(`,
  end one past the `)`. (Change the body fns to return `(Vec<WatAST>, Span)` — items + close span — or
  set the end on the node after the body returns.)
- The reader-macro path (`parse_reader_macro :273`) and char-list (`:258`): end = the inner form's end.

### 4. The `ast-end-span` intrinsic (mirror `ast-span` exactly)
- Impl in `src/edn_shim.rs` beside `eval_ast_span` (`:514`): identical body, but read
  `span.end_line`/`span.end_col` into the `{:line, :col}` map. Name `:wat::core::ast-end-span`.
- Dispatch: `src/runtime.rs` beside `:3754` (`":wat::core::ast-span" => …`).
- Check scheme: `src/check.rs` beside `:16911` (copy `ast-span`'s scheme — `(:wat::WatAST) -> HashMap`).
- Macro-eval allow-list: `src/macros/eval.rs` beside `:579` (`ast-span` is there; `ast-end-span` is
  pure-total too — same justification).

## Proof

- **`tests/probe_arc281_ast_end_span.rs`** (un-ignore): `ast-end-span` of `(a b c)` → `:col` 8.
- **Rust unit tests** in `src/parser.rs` (or a probe): end spans for an atom (`foo` → end col =
  start+3), a nested list (`(a (b) c)` — inner `(b)` end, outer end), a multi-line form (end_line >
  line). Verify `ast-span` (start) is UNCHANGED for all (no regression).
- **Full floors** (the blast radius is wide — Span touches everything): lib 929/36, nursery 893/4,
  deftest 259/1, deporder 0. The 36/4/1 pre-existing failures must not move; the passed counts must not
  drop. This is the load-bearing weigh — a Span change that broke start-position would ripple into
  hundreds of span-asserting tests.

## Out of scope (rejected, not deferred)

- **The auto-fixes themselves** (277.1b ladder fix, the concat→format fix) — this arc ships ONLY the
  end-position primitive + intrinsic. The fixes consume it next.
- **A `Span` product-type record at the wat level** (the `ast-span` doc muses about one) — not now; the
  `{:line, :col}` map is the established shape; `ast-end-span` matches it.
- **`:file` in the map** — excluded, exactly as `ast-span` excludes it (the codemod holds its own path).
- **Byte offsets in Span** — NOT needed; `fix-text-offset-of` derives the offset from line/col + source.

## Four questions

- **Obvious?** YES — `ast-end-span` is the symmetric twin of `ast-span`; "where does this node end" is
  the obvious question a fixer asks after "where does it start."
- **Simple?** The intrinsic is trivial (mirror). The end-tracking is mechanical but WIDE (Span + lexer +
  parser). It is *one concept* (carry the end the lexer already knows) threaded through three layers —
  not braided. The additive `Span::new` default keeps the ripple from exploding.
- **Honest?** YES — the end is the real lexed position, not an estimate; `ast-span` (start) is provably
  unchanged (the weigh asserts it); no node fabricates a span.
- **Good UX?** YES — fixers get exact form extents; `old-len` becomes `end-offset - start-offset`, no
  fragile paren-counting in wat (the extirpare win: don't re-scan structure the parser already knows).

## Blast radius

- `src/span.rs` — add `end_line`/`end_col` + `with_end`; default in `new`.
- `src/lexer.rs` — stamp each token's end via `span_at(end_i)`.
- `src/parser.rs` — atoms use token end; structural nodes combine open.start..close.end.
- `src/edn_shim.rs` + `src/runtime.rs` + `src/check.rs` + `src/macros/eval.rs` — the `ast-end-span`
  intrinsic (impl + dispatch + scheme + allow-list).
- `tests/probe_arc281_ast_end_span.rs` (un-ignore) + parser unit tests.
- NO wat-source change (this is pure substrate). The fix.wat `old-len`-for-structural helper is the
  NEXT stone (277.1b), not this one.

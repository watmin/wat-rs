# Arc 279.1 — `format` literal-brace escape (`{{`/`}}`): the char-walk tokenizer

> **STATUS: STRIKE-READY.** Foundation shipped (`string::subs` added to `is_pure_total`) + proven
> (`tests/probe_arc279b_subs_tuple_macro_eval.rs` GREEN). Feature gate RED + `#[ignore]`'d
> (`tests/probe_arc279b_format_escape.rs`). The build = rewrite `format`'s template parse section in
> `wat/core.wat` as a single-pass char-walk state machine that collapses the doubles. Opened 2026-06-17.

## Why

`format` (arc 279) ships strict named placeholders but **cannot emit a literal brace**. The resolved
design (279 DESIGN): a literal brace is written **doubled** — `{{` → `{`, `}}` → `}` (Rust/Python
convention), collapsed by the macro at expand time. **Zero lexer change** — `{`/`}` are ordinary
string chars; the work is entirely in the macro's template parser.

The current parser (`wat/core.wat:597-705`) splits the template by `{` then by `}`. That naive split
**cannot** host doubling: `"{{x}}"` splits by `{` into `["", "", "x}}"]` and the empty chunk trips the
`n-cp >= 2` "unclosed `{`" guard (`core.wat:648`). Proven RED:
`tests/probe_arc279b_format_escape.rs` — `{{literal}}` → MacroError, `{name}}}` → `"v}}"` not `"v}"`.

A split-based fix degenerates into a fiddly state machine over ambiguous segments (`{{{x}`, `x}b}}c`).
The Obvious+Simple implementation is a **single linear character walk** — which needs char access.

## The intrinsic grow (already shipped, the foundation)

macro-eval (`is_pure_total`) had no char primitive (`subs`/`char-at`/`index-of` absent; `split ""`
refused, `string_ops.rs:438`). `string::subs` already exists — **char-indexed**, start-inclusive /
end-exclusive (Clojure `subs`), runtime+check wired (`string_ops.rs:363`, `runtime.rs:4081`,
`check.rs:14920`) — it was simply not on the allow-list. Added (`src/macros/eval.rs`, beside
`string::length`). "Does a macro need it?" — **yes**: the format parser walks chars at expand time.
`string::length` is char-based (`s.chars().count()`, `string_ops.rs:74`) — matches `subs`'s indexing.

The char vector is built with ops already allowed:
`(map (fn [i] (:wat::core::string::subs s i (:wat::core::i64::+ i 1))) (:wat::core::range 0 (:wat::core::string::length s)))`
→ `Vector<String>` of single chars. The fold state is a heterogeneous **`Tuple`** (on the allow-list,
`eval.rs:451`; constructed `(:wat::core::Tuple a b …)`, accessed `first`/`second`/…). Proven in the
foundation probe.

## The algorithm — a single-pass state machine (THE CONTRACT)

Decomplect into **tokenize → emit** (two passes, each does one thing):

### Pass 1 — tokenize chars → a segment list

`foldl` over the char vector. Accumulator = **`Tuple(mode, pending, buf, segments)`**:
- `mode` : String — `"text"` | `"name"`
- `pending` : String — `"none"` | `"open"` | `"close"` (a brace seen, awaiting disambiguation)
- `buf` : String — accumulated text (text mode) or placeholder name (name mode)
- `segments` : `Vector<Tuple>` — emitted tokens, each `Tuple(kind, payload)` where
  `kind` ∈ {`"text"`, `"slot"`}, `payload` = the literal text / the placeholder name.

Per-char transition `step((mode,pending,buf,segments), c)`:

**`mode == "text"`:**
- `pending == "open"` (previous char was `{`):
  - `c == "{"` → `{{` literal: `(text, "none", buf+"{", segments)`
  - `c == "}"` → **macro-error** `format: empty placeholder {} in template`
  - else → `{` opened a placeholder: flush `buf` as a `text` segment if non-empty, then
    `("name", "none", c, segments')` (name buffer starts with `c`)
- `pending == "close"` (previous char was `}`):
  - `c == "}"` → `}}` literal: `(text, "none", buf+"}", segments)`
  - else → **macro-error** `format: lone '}' in template — use '}}' for a literal brace`
- `pending == "none"`:
  - `c == "{"` → `(text, "open", buf, segments)`  (defer)
  - `c == "}"` → `(text, "close", buf, segments)` (defer)
  - else → `(text, "none", buf+c, segments)`

**`mode == "name"`** (`pending` is always `"none"` here):
- `c == "}"` → close placeholder: emit `Tuple("slot", buf)` to segments, `(text, "none", "", segments')`
- `c == "{"` → **macro-error** `format: '{' inside placeholder name — unclosed '{'?`
- else → `("name", "none", buf+c, segments)`

### Finalization (after the fold, inspect the returned accumulator)

- `pending == "open"` → **macro-error** `format: trailing lone '{' — use '{{' for a literal brace`
- `pending == "close"` → **macro-error** `format: trailing lone '}' — use '}}' for a literal brace`
- `mode == "name"` → **macro-error** `format: unclosed placeholder {<buf>`
- else (text) → flush `buf` as a final `text` segment if non-empty.

### Pass 2 — segments → concat pieces (`Vector<WatAST>`) + used-set

`foldl` over segments:
- `kind == "text"` → append a String-literal AST node for `payload` (see the helper below).
- `kind == "slot"` → validate `payload` is a key in `kwargs-map` (else **macro-error**
  `format: placeholder {<name>} has no matching kwarg`); append `` `(:wat::core::str ~val-ast) ``;
  record `payload` in the used-set.

Then the **existing** strict unused-kwarg check (`core.wat:707-722`) and the **existing** emit tail
(`core.wat:724-736`: empty → `` `"" ``, single → unwrap, else `` `(:wat::core::string::concat ~@pieces) ``)
are reused **unchanged**.

### The string-literal-node helper (reuse + factor)

The current code builds a String AST node twice via the read-string trick
(`core.wat:619-625` and `680-685`). Factor it into one expand-time `let`-bound lambda or inline reuse:
`(first (ast->children (read-string (string::concat "\"" (string::concat text "\"")))))`.
The `"`-in-template guard (`core.wat:556-561`) **stays** — it guarantees `text` never contains `"`,
so the re-wrap is safe.

## Worked cases (the probe asserts these)

| template | result | why |
|---|---|---|
| `"{{literal}}"` | `"{literal}"` | `{{`→`{`, text, `}}`→`}` — no placeholder |
| `"{{x}} = {name}"` (`:name "v"`) | `"{x} = v"` | literal `{x}` + live placeholder |
| `"{name}}}"` (`:name "v"`) | `"v}"` | placeholder closes at first `}`, then `}}`→`}` |

Plus the **preserved** arc-279 behavior: `"{a} {b}"` (`:a "x" :b 5`) → `"x 5"`; missing/unused kwarg →
macro-error; non-literal template → macro-error.

## Out of scope (rejected, not deferred)

- `\{` backslash escape — the `{{`/`}}` doubling is the chosen mechanism; `\{` hits a real lexer STOP
  (279 DESIGN) and is **not** built.
- No new runtime behavior — `:wat::core::str` and the concat emission are unchanged.
- Positional `{}` placeholders — `format` is named-only (arc 279), unchanged.

## Four questions

- **Obvious?** YES — a left-to-right char walk with a 3-state pending flag is how every template
  engine handles doubled-delimiter escapes; each transition is one line.
- **Simple?** YES — tokenize (chars→segments) then emit (segments→AST) are decomplected; the existing
  validation + emit tail is reused verbatim.
- **Honest?** YES — every malformed brace (lone `{`, lone `}`, empty `{}`, unclosed name) is a named
  macro-error, not a silent mangle; the doubling is the only escape and it is total.
- **Good UX?** YES — `{{`/`}}` is the convention Rust/Python users already know; one correct behavior,
  no config; the error messages name the exact fault and the fix.

## Blast radius

- EDIT `wat/core.wat` — the `format` macro's parse section (`:597-705`) → the two-pass tokenizer; the
  doc comment (`:506-541`) updated (the `\{`-not-supported note → `{{`/`}}` doubling). Emit tail +
  validation + kwargs-fold unchanged.
- `src/macros/eval.rs` — `subs` allow-list line (DONE).
- Un-ignore `tests/probe_arc279b_format_escape.rs` (3 tests).
- Nothing else. No Rust runtime/check change (subs already wired).

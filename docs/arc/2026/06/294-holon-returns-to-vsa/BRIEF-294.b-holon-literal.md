# BRIEF — 294.b: the `#holon` relaxed literal (the clj↔wat seam)

**The work, in one paragraph.** Add a reader tag `#holon <form>` that reads the next form as **heterogeneous EDN
data** and types/evaluates it as a `Hologram` — bypassing wat-core's monomorphic collection inference. It is a
**sibling of `quote`**: `#holon` desugars (at read time, via the existing `parse_reader_macro` machinery) to
`(:wat::holon::literal <form>)`; the checker types that head as `:wat::holon::HolonAST` **without recursing into the
body** (exactly as `:wat::core::quote` does); the runtime **captures the body as data** (`eval_quote` — no
evaluation) and **lowers it to a hologram** via `to_holon_inner` (which already structurally lowers a captured
`Value::wat__WatAST` through `watast_to_holon`, runtime.rs:14431). Net: `#holon {:kw ["a"] true #{1 :foo}}` — a
disparate-key, disparate-value map that `infer_map_literal` rejects — measures directly. The byte-identical clj
side (a one-line `{holon identity}` data-reader) is **out of scope for this strike** (the orchestrator lands it +
the cross-read after the Rust is green).

## THE ONE CONTRACT DECISION (pinned — four-questions-selected, Option A)
**`#holon <form>` is a reader macro that desugars to `(:wat::holon::literal <form>)`, a new special-form head that
is the data-typed sibling of `:wat::core::quote`.** It reuses `List` (NO new `WatAST` variant), the quote
checker/runtime/registration pattern, and the `to_holon_inner` codec. The enclosed form is **DATA, captured not
evaluated** — consistent with the quote-like checker (no body inference) and matching Clojure's data-reader
semantics (so the same bytes are byte-identical across wat + clj).

## Read in order (the rooms — every one a `quote` mirror, grounded this session)
1. **Reader — lexer.** `crates/wat-reader/src/lexer.rs:318` (the `#{` → `Token::LHashBrace` arm). Add a **clean
   `#holon` arm**: when `#` is followed exactly by `holon` AND the next char is a delimiter (whitespace / `(` / `{`
   / `[` / `#` / EOF — NOT an identifier char, so `#holonx` does NOT match), emit a new `Token::HolonLiteral`
   (mirror the `Token::Quote` family at lexer.rs:116–121, 349–354). Span covers the 6 chars `#holon`.
2. **Reader — parser.** `crates/wat-reader/src/parser.rs:267` (`Token::Quote => self.parse_reader_macro(...)`). Add
   `Token::HolonLiteral => self.parse_reader_macro(":wat::holon::literal", span)`. `parse_reader_macro` (parser.rs:293)
   already wraps the next form as `(head inner)` — nothing else to build.
3. **Checker.** `src/check.rs:4389` (the `:wat::core::quote` arm). Add a `:wat::holon::literal` arm beside it:
   arity-1 guard (same shape), then `return CheckResult::ok(TypeExpr::Path(":wat::holon::HolonAST".into()))` — **do
   NOT recurse into the body** (that is the whole point — it bypasses `infer_map_literal`). Type path confirmed at
   check.rs:4463.
4. **Runtime.** `src/runtime.rs:4007` (`":wat::core::quote" => eval_quote(args, list_span)`). Add
   `":wat::holon::literal" => to_holon_inner(eval_quote(args, list_span)?, list_span)` — capture-as-data via
   `eval_quote` (→ `Value::wat__WatAST`), then lower via `to_holon_inner` (the `Value::wat__WatAST => watast_to_holon`
   arm at runtime.rs:14431 already exists). Result is `Value::holon__HolonAST` — a measurable hologram.
5. **Registration mirrors (grep `":wat::core::quote"` and add a `:wat::holon::literal` sibling at each):**
   - `src/special_forms.rs:220` — `insert(&mut m, ":wat::holon::literal", &["<form>"]);`
   - `src/special_forms.rs:346` — add to the special-form keyword list.
   - `src/resolve/boundary.rs:57` — add `:wat::holon::literal` to the **`Boundary::AllData`** arm (body resolves as
     data, no symbol resolution — same as quote/forms).
   - `src/rete/purity.rs:214` — add to the pure-quote match (a holon literal is pure).
   - `src/macros/eval.rs:155` + `src/macros/expand.rs:195` — mirror the quote handling (a `#holon` form is data, not
     expanded/evaluated by the macro engine).
   - `src/runtime.rs:7852` — add to the dispatch/known-op list quote sits in.

## Implementation sketch (fill the mechanics; do not invent the shape)
- Reader: one new `Token::HolonLiteral` variant + one lexer recognition arm + one parser dispatch arm. Mirror the
  `Quote` token end-to-end.
- Checker: ~8-line arm, a near-clone of the quote arm, returning the `HolonAST` path instead of `:wat::WatAST`.
- Runtime: one dispatch arm, `to_holon_inner(eval_quote(args, list_span)?, list_span)`.
- Registration: one line each at the 6 mirror sites above.
- **Un-ignore the probe:** delete the `#[ignore = …]` on `tests/types/probe_arc294b_holon_literal.rs` — it flips GREEN.

## Blast radius (bounded)
`crates/wat-reader/src/{lexer.rs,parser.rs,ast.rs?}` (the token + arms) · `src/check.rs` (one arm) ·
`src/runtime.rs` (one arm) · `src/special_forms.rs`, `src/resolve/boundary.rs`, `src/rete/purity.rs`,
`src/macros/eval.rs`, `src/macros/expand.rs` (one mirror line each). **No new `WatAST` variant. No new type. No
change to `infer_map_literal`, `to_holon_inner`, `watast_to_holon`, or `eval_quote` — they are reused as-is.**

## STOP triggers (halt + surface; do NOT improvise)
- **STOP-1:** if landing `#holon` requires a **new `WatAST` variant** (the parser/checker won't carry it as a
  `List`-desugar) — STOP. The four-questions chose Option A *because* it needs no new variant; if that's false the
  decision must be revisited.
- **STOP-2:** if the lexer cannot recognize `#holon` in a **single clean arm** without entangling other `#`-prefixed
  lexing (e.g. `#wat-edn.*` wire tags, `#{`) — STOP and surface the collision; do not broaden into a general
  `#<tag>` reader-macro scanner (that is a bigger, separate design).
- **STOP-3:** if `to_holon_inner(eval_quote(...))` does **not** yield a measurable hologram for the fixture (i.e.
  the `Value::wat__WatAST => watast_to_holon` arm doesn't cover the heterogeneous map/set/vector/scalar leaves the
  fixture uses) — STOP and report which leaf `watast_to_holon` drops; do not hand-patch the codec.

## EXPECTATIONS (scorecard — fixed before the strike)
| # | what | command | expected |
|---|---|---|---|
| 1 | the RED probe flips GREEN | `cargo nextest run --release -p wat -E 'test(holon_tag_makes_heterogeneous_edn_measure)'` (after un-ignore) | PASS |
| 2 | the demo fixture runs + measures | `cargo wat wat-scripts/demos/holon-literal/cosine.wat` | prints `1.0` (identical holon literals → exact coincidence) |
| 3 | a 2-literal heterogeneous cosine differs honestly | (orchestrator adds a probe asserting `#holon {…a…}` vs `#holon {…b…}` ∈ (0,1)) | a cosine in (0,1), not 1.0, not error |
| 4 | **bare het map STILL rejects (monomorphic wall intact)** | `cargo wat` on a bare `{:kw [..] true #{..}}` (no `#holon`) | still type-errors (we did NOT relax core literals) |
| 5 | nothing else breaks | `cargo nextest run --release` (whole workspace) | floor 0; SET-diff ∅ vs HEAD |

**Runtime prediction:** 20–35 min. **Trap-door:** the lexer arm is the one hot-path edit — read the lexer diff
end-to-end (a greedy `#holon` match that swallows a following delimiter, or shadows `#{`, would be a silent
corruption tests might not catch). **Content-integrity:** confirm nothing outside the named rooms moved.

**You are a LEAF. Do NOT spawn subagents. If the work exceeds this brief or hits a STOP, halt and report.**

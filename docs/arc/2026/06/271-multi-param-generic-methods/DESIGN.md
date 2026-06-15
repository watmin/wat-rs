# Arc 271 — multi-type-param generic protocol methods (`method<S,R>`)

> Shipped 2026-06-15. The block-and-build dep for arc-209 stone host-parity-4a
> ([[DESIGN-STONE-host-parity-4a-start]]): the host-agnostic `Host/spawn` method must be generic over
> BOTH `S` and `R`. Grounded against HEAD `4dc4ea69`.

## The bug — one design, applied in one place only

The question that found it (builder): *"do we have N ways to handle generics? did we just forget to
use the tooling?"* — the latter. There is ONE mechanism (angle-depth-aware tokenization), implemented
in `lex_keyword` (lexer.rs:637) but NOT in `lex_symbol` (lexer.rs:811):

- `lex_keyword` tracks `<...>` depth (line 663+) and keeps commas inside it (line 768) — so generic
  **fn** names, which are KEYWORDS (`:wat::core::foldl<T,Acc>`), parse multi-param fine.
- `lex_symbol` was a naive `while !is_symbol_break` scan with zero angle-awareness — so a generic
  **method** name, a bare **Symbol** (`combine<A,B>`, arc-232 convention), split at the comma
  (`is_symbol_break`, lexer.rs:467 — comma is EDN whitespace) into `combine<A` + `B>`.

Everything downstream was already multi-param-ready: `split_name_and_type_params` (runtime.rs:2998)
splits `inside` on `,`; the call-site instantiation (check.rs:5541) loops over ALL `type_params`.
Single-param `make<T>` worked only because it has no comma to split on.

## The fix

Teach `lex_symbol` the same angle-awareness `lex_keyword` already has: track `angle_depth`; `<` opens
a type-head ONLY when preceded by alphanumeric / `_` / `'` (so operator/leading `<` — `<-`, `<`, `<=`
— never false-opens); while `angle_depth > 0` a comma is retained instead of breaking. One mechanism,
now applied in both lexer paths. (A full extraction into a shared scanner is heavier — `lex_keyword`
also juggles `()`/`[]`/`{}` keyword-specific rules — and is not warranted for this one-line-of-logic
parity; the angle/comma rule is mirrored, not duplicated wholesale.)

## Gate

- `tests/probe_arc271_multi_param_generic_method.rs` — `combine<A,B>` called with `(i64, String)` →
  A=i64 (return), B=String (the `y` arg a literal `:B` would reject) → returns `5`. RED at HEAD
  (`:t::Combiner/combine` unresolved — the split drops method registration); GREEN after the fix.
- Sibling `probe_arc232_generic_method` (single `<T>`) still green. lexer units 48/0. lib 917/36 +
  nursery 895/4 (zero-new). Workspace builds.

Unblocks host-parity-4a-ii (`Host/spawn` generic over S,R). Pairs [[feedback_use_the_tool_not_hand_fix]]
+ [[feedback_deferred_dep_becomes_necessary_block_and_build]].

# BRIEF — Arc 281: `ast-end-span` (node end-position tracking)

You are a single-hop sonnet executor in `/home/watmin/work/holon/wat-rs`. **Do NOT spawn sub-agents.
Do NOT run `git`.** Build, run the named tests, report. The orchestrator weighs independently on its
own build — this change is WIDE (it touches `Span`), so it will re-run the full floors.

## The work (one paragraph)

Add `:wat::core::ast-end-span` — the symmetric twin of `ast-span` — returning a node's END
`{:line, :col}` (one char past its last char; for `(a b c)`, col 8). This requires threading
end-position from the lexer (which already tracks char position) up through the parser: `Span` gains
`end_line`/`end_col`, the lexer stamps each token's end, the parser sets each node's end (atoms from
their token; structural nodes from their closing delimiter), and the intrinsic reads it.

## The contract — implement EXACTLY the DESIGN

Read **`docs/arc/2026/06/281-ast-end-span/DESIGN.md` § "The mechanism"** and implement its four parts
verbatim (Span additive end + `with_end`; lexer stamps end via `span_at(end_i)`; parser combines
open.start..close.end; the `ast-end-span` intrinsic mirroring `ast-span`).

## Read in order (the rooms)

1. `docs/arc/2026/06/281-ast-end-span/DESIGN.md` — THE SPEC.
2. `src/span.rs:44-83` — `Span` struct + `new`/`with_end`(add)/`unknown`/`is_unknown`. Add `end_line`,
   `end_col`; `new` defaults them to `line`/`col`; add `with_end`.
3. `src/lexer.rs:240-300` (the lex loop + `span_at` at `:248`, line/col at `:443`) — stamp each
   token's span with its end. For single-char delims end_i = i+1; for multi-char tokens (atoms,
   strings, keywords) end_i = the scan-end index. Use `span_at(end_i)` for the end line/col.
4. `src/parser.rs:200-264` (`parse_form` — atoms + `WatAST::List/Vector/Map` construction) and
   `:290-400` (`parse_list_body`/`parse_vector_body`/`parse_brace_body` — they consume the closing
   delimiter). Thread the close token's span so structural nodes get `with_end(open…, close.end…)`.
   Atoms use their token's span directly (already end-stamped by the lexer).
5. `src/edn_shim.rs:506-548` (`eval_ast_span`) — copy to `eval_ast_end_span`, reading
   `span.end_line`/`span.end_col`.
6. `src/runtime.rs:3754` — add the `:wat::core::ast-end-span` dispatch arm beside `ast-span`.
7. `src/check.rs:16911` — register `ast-end-span` with the SAME scheme as `ast-span`.
8. `src/macros/eval.rs:579` — add `:wat::core::ast-end-span` to `is_pure_total` beside `ast-span`.
9. `tests/probe_arc281_ast_end_span.rs` — remove the `#[ignore = "arc 281 …"]` attribute.

## STOP triggers (halt + report, do not improvise)

1. **`ast-span` (start) MUST stay byte-identical.** If adding end fields changes ANY node's START
   line/col (existing span-asserting tests go red), STOP — the start is sacrosanct; only the end is new.
2. If the body-parser signature change (returning the close span) ripples into call sites you can't
   cleanly thread, STOP and report — do not fabricate the end (e.g. do not set end=start for lists;
   that defeats the feature).
3. If the lexer's `end_i` for a multi-char token is ambiguous (where exactly the token ends), STOP and
   report the token kind — do not guess an off-by-one.

## Blast radius

`src/span.rs`, `src/lexer.rs`, `src/parser.rs`, `src/edn_shim.rs`, `src/runtime.rs`, `src/check.rs`,
`src/macros/eval.rs`, and `tests/probe_arc281_ast_end_span.rs` (un-ignore) + any new parser unit tests
you add. **NO wat-source changes** (this is pure substrate). No git.

## Verify (run these, paste output verbatim)
```
cargo test --release -p wat --test probe_arc281_ast_end_span             # 1/1 GREEN (end col == 8)
cargo test --release -p wat --lib -- --test-threads=1 2>&1 | grep "test result"   # 929 passed / 36 failed (UNCHANGED — start spans intact)
cargo test --release -p wat --test nursery -- --test-threads=1 2>&1 | grep "test result"  # 893 passed / 4 failed (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"               # 259 passed / 1 failed (UNCHANGED)
cargo test --release --test test_stdlib_load_order 2>&1 | grep result    # 1 passed / 0 failed
```
Report: a summary of each file's change, the five command outputs verbatim, and any delta from
expected. If the lib/nursery/deftest counts moved AT ALL, say so loudly and explain — a moved count
means a span regression (STOP-1). Do not claim green you did not see.

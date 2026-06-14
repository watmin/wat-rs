# EXPECTATIONS — Stone C.1: defservice skeleton + op enum

Independent scorecard, fixed BEFORE the strike so the result can't move the goalposts. The
Inquisitor scores against its OWN re-run (every row), reading the diff, crediting nothing the
disk doesn't show.

## Scorecard

| # | what | command | expected |
|---|---|---|---|
| 1 | C.1 gate: defservice emits the op enum; the enum constructs + matches | `cargo test --release -p wat --test probe_arc209_c1_defservice_op_enum` | `1 passed` |
| 2 | fence prereq still holds (no regression to the foundation) | `cargo test --release -p wat --test probe_arc209_c1_defmacro_ast_walk` | `2 passed` |
| 3 | lib unit baseline unchanged | `cargo test --release -p wat --lib -- --test-threads=1` | `915 passed / 36 failed` (zero new) |
| 4 | nursery baseline unchanged | `cargo test --release -p wat --test nursery -- --test-threads=1` | `895 passed / 4 failed` (zero new) |
| 5 | full surface compiles | `cargo test --release --workspace --no-run` | exit 0 |
| 6 | the new file is pure wat + one stdlib line | `git -C . diff --stat` | `wat/service.wat` (new) + `src/stdlib.rs` (+~4) ONLY |

## Runtime prediction

8–15 min. The macro is ~25 lines of wat; the algorithm is fully spelled in the DESIGN; the
foundation probe + `cond` + `keyword/of` are verbatim references. Most of the risk was retired by
the C.1-pre fence stone and the program-body-contract discovery.

## Trap-doors named

- **Top-level quasiquote** — the single most likely slip. If the macro body's top-level is
  `` `~(…)`` / `` `(…)``, the param evaluates and `ast->children` gets a value, not a node
  (STOP-2). The body's top-level MUST be a regular form (`let`); quasiquote only nested. The
  foundation probe is the antidote — mirror it.
- **Enum-name colon** — `keyword-node` requires a `:`-prefix; `keyword/from-string` takes the text
  WITHOUT the colon and adds it. Use `keyword/from-string` (as `keyword/of` does), NOT
  `keyword-node`, for the `<fqdn>::Op` build.
- **Drop count** — the self-arg is a THREE-token triple `s <- :State` in the arg-vec's children;
  drop 3 (not 1). An empty `:State`-only arg-vec → `(drop … 3)` = empty → bare variant.
- **Variant flatten** — building per-op token *lists* then needing a flatten; the `foldl`+`conj`
  shape in the DESIGN avoids it (appends keyword [+ field-vec] directly to one accumulator).
- **op-head is already a keyword** — `(first ch)` yields the `:Increment` keyword node directly;
  no symbol→keyword conversion (the surface decision pays off here).

## Out of scope (affirmatively cut — not deferred)

The dispatch loop (C.2), client wrappers + start fn (C.3), handler-body emission, the counter
proof (Stone D). C.1 ships the op enum and nothing else; the bodies/ret in `:ops` are read-and-
ignored until C.2.

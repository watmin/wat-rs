# WEIGH — STONE 255.95: a keyword holds a pair — ACCEPTED

**Executor: grok via pulsare, solo, through three amendments** (`$bare`; S1 a symbol is a value; the checker says what
runs). Commits `059f2af80`, `9854e3c66`, `f8eaaa16e`, `3bbb742c5`; SCORE `17f140ccb`. Weighed by the orchestrator on
2026-10-05.

## Re-run by the orchestrator at `17f140ccb`

| row | result |
|---|---|
| release floor | `.floor/2026-10-05T08-11-39Z`: **6424 passed / 24 skipped**, exit 0, doc-link 0 |
| test-name set | against `.floor/2026-10-05T07-34-12Z`: **MISSING 0**, ADDED 3 (the checker gate) |
| clippy | exit 0 |
| ignores | 18 |
| cost (grok, six and six) | fuzz 31.95 s against 31.99 s at `0b2dcc1e2` |

## What landed (builder rulings KW1, `$bare`, S1, 2026-10-04)

- **Measured first:** the 88,952 run-time `::` keyword values were 58 spellings, **all names, no data** (the builder
  predicted zero).
- **A keyword holds a `Name`;** an unqualified keyword is `{$bare, k}`; printed `:ns/name` or `:k`; `$bare/x` refused.
- **A symbol is a value** (`Value::Symbol(Name)`): pure data, crosses a process boundary, resolved only by `apply`
  (tested across a process). `'a.b/c` and `:a.b/c` are unequal (tested).
- **`defservice` passes names as symbols;** no stdlib caller builds a name with `keyword/from-string` (six data uses
  remain: metric keys, variant leaves).
- `<` is a name character; the sentinel is `:wat.kernel/__peer_crashed__`.
- **The checker states what runs:** `compose-variant` and `variant-parent-of` typed as the symbols they return; a gate
  binds each verb's result to a `defclause` typed as the checker says and runs it (red on the old tree, shown).

## Owed (noted)

- The two verbs' `@example` lines still show a keyword result (`:wat::cache::Lru.Hit`, `:wat::core::Option`); the
  comments at `src/runtime.rs:8065` and `wat/process.wat:54` still say "keyword". Docs that state a retired type.
- The text bridge (`get(&str)`, spelling index, `canonical-identity` keeping `::` for `fix.wat`'s enum maps) stays until
  nothing passes text: the reader stone.

## Verdict

Accepted and pushed. Next: the reader is the door: `::` keywords in name positions read as symbols.

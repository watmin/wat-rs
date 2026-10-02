# WEIGH — STONE 255.82: cutover 5a — a call head that names nothing is refused — ACCEPTED

**Executor: grok via pulsare, commits `25b7ee38f` (STOP-1, census) and `581478c9c` (the refusal).** Weighed by the
orchestrator on 2026-10-02.

## Re-run by the orchestrator at `581478c9c`

| row | result |
|---|---|
| release floor | `.floor/2026-10-02T19-08-53Z`: **6374 passed / 24 skipped**, exit 0 |
| clippy | `cargo clippy --release --all-targets -- -D warnings`, exit 0 |
| the probe | `wat --check …/probe-255.82-a-head-that-names-nothing.wat.bad`: refused, `unresolved references`, naming `my.made.up.thing`, `wat.core.Option.zzznope` (and `foozle`; grok's SCORE: "3 unresolved references") |

## What landed

- A call head that is a symbol with no `/`, not a local in scope, and not accepted by the one door
  (`is_resolvable_call_head`) is `UnresolvedReference` at check time. `UnresolvedReference` gained an optional `remedy`
  (omitted from EDN when absent, so keyword-head refusals are byte-identical): a dotted wrong join carries the slash
  spelling (`wat.core.Option.expect` → `wat.core.Option/expect`). Bare `Some`/`Ok`/`Err` stay refused (admitted only by
  the door's `is_retired` rung) and still name their replacement.
- Scope for a bare head mirrors the checker (`let`, `fn`, `defclause`, `match`); `extend-type` method names are
  declarations; quoted forms are data. Keyword-heresy ledger **147 → 146**.
- **STOP-1 ruled:** the 24 bare `(def …)` heads were never a form (written so at birth, `87311ecf3`, never ran); once
  `def` was real the three scratch files failed `UnnamespacedName` and proved nothing, so they were deleted, with the
  reason.

## Carried to 5b (the census of keyword-text matchers)

- **The quasiquote escape detector is keyword-only:** a symbol-spelled `wat.core/unquote` inside a template is read as
  data. After 5c converts the corpus, a template would stop unquoting without a word.
- `src/resolve/walk.rs:338` joins a dotted namespace into the door's key with `stem.replace('.', "::")` under a
  `namespace` rune: a string operation on names (CLAUDE.md's recurring class) beside the identity door `ns_to_wat_path`.
- 39 bare unresolved heads in embedded Rust literals are fragments (feature names, template bodies, let binders), not
  loaded programs; none is refused today.

## Verdict

Accepted and pushed. Next: 5b.

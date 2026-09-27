# WEIGH — STONE 255.55: a newtype is tagged, and ordered by its inner value — ACCEPTED

**Executor: grok via pulsare, commit `55fe36769`.** Weighed by the orchestrator on 2026-09-26. Written fresh on `main`
(builder: *"the amount of effort is near zero"*), with the names of `origin/reason/little-wat-findings` stone R
(`AggregateValue.is_newtype`, `AggregateValue::newtype`) so the eventual merge reconciles like with like.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | **6186 passed / 23 skipped** (6179 + 7 rows) |
| the diff | read: `is_newtype` is stamped only by `AggregateValue::newtype` (a newtype's construction arm); one `values_compare` arm orders two same-class newtypes by their inner value, else `None` |
| the IDE's "`values_compare` is private" | stale: it is `pub` now (`src/runtime.rs:5961`) |
| census / delta / clippy | grok's: pre/post `no STOP-8`, NEW 2 / RECOVERY 0 same files, clippy rc 0. Not re-run |

## Findings

- **F-030 reproduced on `main`, and is fixed.** A newtype's field name `"0"` reached `Keyword::new` and panicked.
  Now a newtype writes `#ns/Name <inner>` (`"#u/T 7"`) and reads back `=`.
- The first floor's three reds were this stone's own test-text lints (inlined EDN/wat, a loose `contains`). They
  were kept and fixed, not re-run away.
- 255.54's newtype STOP-2 is answered: a declared-`Orderable` newtype now has a runtime arm.

## Carried

- `values_compare` became `pub` so the test could call it, because `<` does not reach it until 255.56 switches the
  gate. After the switch, the test drives `<` in wat and `values_compare` returns to `pub(crate)`.
- Merge note for `reason/little-wat-findings`: `src/edn/render.rs`, `src/record/construct.rs` and `src/value/value.rs`
  now carry this stone's version of stone R. Reconcile at that merge.

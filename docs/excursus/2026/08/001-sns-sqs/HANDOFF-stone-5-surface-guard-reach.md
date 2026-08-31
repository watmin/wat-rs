# HANDOFF → grok — excursus 001 stone 5: the surface guard's reach

Branch `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-5-surface-guard-reach.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-5-surface-guard-reach.md`
- `docs/excursus/2026/08/001-sns-sqs/repro/README.md` — the two reproductions

**This stone fixes the thing that sent your last strike in the wrong direction.** The
`:messages` completeness guard already exists and already says exactly what stone 4 had to
derive by hand. It just never looks at the field type `wat-queue` used.

`src/types/surface.rs:927-935` — the `<-` handler matches only `WatAST::Keyword`. A field typed
with a **parametric form**, `(:wat::core::Vector :- [:p::Item])`, is a `WatAST::List` and is
skipped entirely. `collect_user_type_paths` (`:974-995`) descends into parametrics perfectly —
it is simply never called for these. **Do not change it** (STOP-1).

Measured, both directions:

```
:Ok [item  <- :p::Item]                            --check = 1   GUARD FIRES
:Ok [items <- (:wat::core::Vector :- [:p::Item])]  --check = 0   GUARD MISSES
```

⚠ **THIS STONE IS EXPECTED TO TURN THE FLOOR RED.** `wat-scripts/queue/sqs.wat` carries the
missed shape, so after the fix it must **stop freezing**. That is the guard working. Red on the
queue is expected; red anywhere else is yours. **Do not fix the queue here** — that is stone 6.

★ **Row 7 is the interesting row.** The guard has been blind to parametric field types since arc
278, so **any surface in the tree could carry the same latent defect**. Widening the reach is
what surfaces them. Each one is a real fork-failure waiting to happen: **name them, do not fix
them**, and do not let the count discourage the widening.

Assert the guard's reason **byte-identical** — `no_loose_string_assert`, and the sibling test at
`src/types/surface.rs:1200-1210` already does this and explains why a loose check would pass on
a wall that named the wrong message.

Verify in the FOREGROUND; read the Summary line. On a red outside the queue: do NOT re-run,
capture the arm whole, name the exact assertion.

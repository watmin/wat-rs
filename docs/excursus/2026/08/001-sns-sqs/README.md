# excursus 001 — SNS and SQS

**What this is:** free experimentation, not commissioned work. `docs/excursus/` is the sibling
of `docs/arc/`; see `docs/excursus/README.md` for why it exists and for the `(301)` residue in
the commit log.

**Artifacts are per stone**, four kinds, all named `<KIND>-stone-<slug>.md`:
`BRIEF` (what to do) · `EXPECTATIONS` (written before, so the result cannot move the goalposts)
· `HANDOFF-…` (the executor's entry point) · `SCORE` (written after the orchestrator's OWN
re-run, never from the report).

## The stones, in order

| stone | what | state |
|---|---|---|
| 1 | **SNS in userland** — one topic, N subscribers, both loci | ✅ `wat-scripts/topic/sns-fanout.wat` → `"3 3"` |
| 2 | `:wat::query::Store` gains **`delete`** — SQS's ack needs it | ✅ struck |
| 2b | the **delete differential**, mem vs sqlite, GSI included | ✅ backends agree |
| 2c | **mem's `put` becomes a replace** — `PutItem` is the referent | ✅ and it exposed a live bug |
| INST | **`#inst` at constant nanosecond width** — one token | ✅ struck |
| WRITE-OPTS | serialization options become a **value the caller passes** | ✅ struck |
| WO-OPT | that opts argument becomes **optional** | ✅ struck |
| JOURNAL-CENSUS | run every journal fixture against **both** backends | ✅ 13/15 agree — and why that is not reassuring |
| SORTKEY | **a telemetry event carries its own identity** | ✅ first fully green floor |
| **3** | **SQS in userland** — `wat-scripts/queue/`, **wat-queue** | ◀ **drawn, not struck** |

## The shape of the detour, because it is not obvious from the list

Stone 1 built SNS. **Drawing stone 3 is what uncovered everything between them**: `receive`
needs a *re-put* (which found `mem-store` appending where `sqlite-store` replaced), and `ack`
needs a *delete* (which the `Store` did not have). Fixing the first made `mem` stop hiding a
`journal` key collision that was silently dropping two metrics in three from every span close —
on every conforming backend. Stones INST through SORTKEY are that debt, paid.

★ **Stone 2c broke nothing. It removed a blindfold.** And the eventual fix was cheap *because*
the substrate was made honest first — `SortKey.time` is a real `Instant` rather than a
hand-padded string only because INST fixed the renderer.

## Findings that outlived their stone

- `NOTE-mem-store-put-appends-where-sqlite-replaces.md` — an oracle that admits a state the
  subject cannot represent is not an oracle. Carries a ⛔ CORRECTED section: it first called
  the divergence a tie between two defensible readings. It was not; DynamoDB rules.
- `NOTE-journal-loses-metrics-on-sqlite-because-sk-is-time-only.md` — also ⛔ CORRECTED: the
  table at its top is **half a measurement**, and the bug is not sqlite-specific.
- `NOTE-a-record-accessor-in-value-position-loses-its-receiver-type.md` — filed to its home arc
  as `docs/arc/2026/04/109-kill-std/NOTE-a-callable-keyword-in-value-position-has-four-kinds-and-three-answers.md`.

## Where the code lives

```
wat-scripts/topic/    wat-topic   — the SNS half
wat-scripts/queue/    wat-queue   — the SQS half (stone 3)
wat/topic.wat, wat/queue.wat      — the builder's to grant, once they demonstrate excellence
```

# BRIEF — clamp the request, not the handler

Move the clamp's conditional inside the `let` binding so the handler body is spliced once.
`wat/service.wat` only. Read `DESIGN-clamp-the-request-not-the-handler.md` first.

## READ IN ORDER

| room | why |
|---|---|
| `wat/service.wat:2208` | `page-clamped` — the `if` that splices `~outcome-match` into both branches |
| `wat/service.wat:2185-2207` | `page-ctor-args` — the field-by-field request rebuild, unchanged by this stone |
| `wat/service.wat:2214` | `shape-guarded` — where `page-clamped` lands inside `Validation::Valid` |
| `wat-scripts/scratch-pad/probe-does-an-unused-arm-cost-in-a-service.wat` | the measurement behind the diagnosis: a large unused arm is ~30 µs in a service impl, 0 ns in a defn |

## SKETCH

```wat
page-clamped  (:wat::core::if (:wat::core::= page-field "")
                outcome-match
                `(:wat::core::let
                   [~req-binder (:wat::core::if (:wat::i64::> (~page-limit-acc ~req-binder) ~page-cap-kw)
                                  (~page-req-ctor ~@page-ctor-args)
                                  ~req-binder)]
                   ~outcome-match))
```

`~outcome-match` appears **once**. Everything else is untouched.

## BLAST RADIUS

`wat/service.wat` only, plus scratch probes. **No `src/`, no `wat/query.wat`, no `sqs.wat`, no
`circuit.wat`.** No declaration changes.

## STOP TRIGGERS

- **STOP-1** — `~req-binder` cannot be rebound in a `let` at that position (shadowing, or the
  binder is not a plain symbol). Report the shape.
- **STOP-2** — publish does **not** recover. **That is the finding**: it would mean the +7.8 % is
  not the duplication, and the diagnosis in the DESIGN is wrong. Report it and stop; do not hunt
  for a second cause under this brief.
- **STOP-3** — the write-side `:max-entries` guard turns out to splice a handler too. Report it;
  do not fix it here.
- **STOP-4** — anything outside `wat/service.wat`.

## THE MEASUREMENT

`circuit.wat` ×3 with **both** delay sites pinned at 25 ms (parent `await-timer-ms` **and** the
Publisher child's `await-ms`), then restored.

```
before :max-page   publish 18466   receives ~4780     ← the target
with   :max-page   publish 19912   receives ~4700     ← today
```

Plus `probe-a-read-declares-its-page.wat` unchanged — the bound and the tool must still behave
identically.

## GRADE AGAINST

`SCORE-find-the-934-milliseconds.md` — the arc's prior "a shape change cost us time" stone.

Write `SCORE-clamp-the-request-not-the-handler.md`, then `pulsare_yield kind=scored`.

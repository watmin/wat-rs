# BRIEF — a call outcome cannot lie

Replace `call-by-deadline`'s `(Option O, i64)` return with a parametric three-arm enum.
**A pure refactor: every number must come back identical.**

Read `DESIGN-a-call-outcome-cannot-lie.md` first.
`wat-scripts/scratch-pad/probe-can-a-defenum-take-a-type-param.wat` proves the shape is
expressible and is the syntax to copy — it is the tree's first parametric `defenum`, so there is
no other worked example.

## READ IN ORDER

| room | why you are there |
|---|---|
| `wat/service.wat:3119-3123` | the comment block and `call-by-deadline`'s signature. The `defenum` goes **immediately before** the defn, still after the macro |
| `wat/service.wat:3126-3150` | the four `Tuple` returns — `(Some m, 0)`, `(None, 1)`, `(None, 2)` — each becomes one arm |
| `circuit.wat:397-401` | **`receive`** — the site that reads `(first recv-got)` and never the code. After this it *cannot*; the fallback arm at `:535-542` becomes `PeerGone` + `DeadlineFired` |
| `circuit.wat:440-444` | `check` — `ans`/`code`/`peer'`, the pattern the other two copy |
| `circuit.wat:484-486` | `mark` |
| `circuit.wat:513-515` | `ack` |

## SKETCH

```wat
(:wat::core::defenum :wat::service::CallOutcome :- [O] :wat::enum::Pure
  :Answered      [reply <- :O]
  :PeerGone      []
  :DeadlineFired [])
```

Each call site becomes a `match` on three arms. Where the old code read
`(if (:wat::i64::= code 0) peer (redial))`, the redial now lives in the `PeerGone` and
`DeadlineFired` arms — **identical behaviour, stated twice instead of derived from an integer.**

## BLAST RADIUS

`wat/service.wat` and `wat-scripts/fanout/circuit.wat`. No `src/`, no `sqs.wat`, no codemod, no
test files — **if a `.rs` file needs to change, that is STOP-3.**

⚠ **A `wat/` edit is frozen into the binary at build time — rebuild before any run.**

## STOP TRIGGERS

- **STOP-1** — if the parametric `defenum` cannot be declared inside `wat/service.wat`
  specifically (as opposed to the scratch-pad probe, where it works), STOP and report the exact
  checker error. Do not fall back to a non-parametric enum per Reply type.
- **STOP-2** — if any number moves. This is a refactor; a changed number means the arms are not
  equivalent to the codes. Report which row moved rather than adjusting anything.
- **STOP-3** — if the change reaches a `.rs` file, STOP and report.
- **STOP-4** — do not add a fourth arm splitting `Lost` from `Closed`. It is cut in the DESIGN
  with its reason.

## PRIOR RESULT TO COPY FOR SHAPE

`SCORE-every-client-call-has-a-deadline.md` — the stone this tightens, one round back.

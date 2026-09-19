# EXPECTATIONS — one selectable-set primitive

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null

⛔ **"Two impls ARE warranted, and here is the cell that warrants them" is a FULL delivery** — the
drift table plus the load-bearing difference, nothing unified. Unifying *through* a real difference
to satisfy the sentence is the failure mode this row exists to prevent.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | ⭐ **The drift table** | Every `SelectOutcome` variant × tier (thread, process) × both impls → which `ServiceEvent`. One row per cell, read from the code. ⛔ A table with a blank cell does not score. The decode-failure row is known (`poll`→`Malformed`, `select`→`Lost`); **the value is the cells nobody has compared.** |
| 2 | **One engine** | `select` is the peers-only call of the same code path. Both wat verbs survive with byte-identical signatures (18 call sites). |
| 3 | **The `Lost` collapse is gone** | A decode failure classifies the same way from both verbs. ⛔ Judged by MUTATION: break the shared classification, show a test redden — the bar `632335c55` set after a control that could not. |
| 4 | **Variant set follows from inputs** | A peers-only call cannot build `Admin`/`Connection` because nothing in its set can, not via `unreachable!()`. If expressing that in the TYPE needs a narrower event type, say so and stop — that is the next stone. |
| 5 | **No serve-loop regression** | `poll` is the service hot loop (`service.wat:2478`). Before/after measurement, stated as a single run unless you ran more. |
| 6 | **Scope wall** | bracket's four dead arms, the transport resend, both wat signatures, and `ServiceEvent`'s shape all untouched. Say what you did not do. |
| 7 | **Floor** | `scripts/floor.sh`, release. Summary verbatim + `.floor/<stamp>/`. A red: do not re-run, capture whole, name the arm. Clippy over the WHOLE output. ⚠ `no_loose_string_assert` / `no_inlined_wat_in_tests` police new tests; fixtures, and runes only with a reason that earns it. |

## What would make this stone wrong

- **A unification that picks one side's behaviour for an uncompared cell.** Row 1 exists for this.
- **`unreachable!()` traded for a silent default.** If a peers-only call can now reach the listener
  arm, that is a bug, not a `_ => nothing`.
- **A measured regression in the serve loop waved off as noise** — state the number.
- **The four bracket arms quietly repaired.** Out of scope; the parked stone owns them and depends
  on this one landing first.

## Deliverable

`SCORE.md`: the full drift table, what unified, the mutation evidence for row 3, the serve-loop
measurement, floor Summary + `.floor/` path.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).

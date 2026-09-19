# EXPECTATIONS — the bracket runs on the queue

Scored against `DESIGN.md`. Write `SCORE.md` beside it.

## The null, and what is NOT a null

⛔ **"`Locus/launch` cannot honestly describe a queue" is a FULL delivery** — row 1 decided against
(a), with the parameter-by-parameter reading that decided it. Report it and take (b), or report that
both are wrong and land nothing.

⚠ **But "there are panics elsewhere" is NOT a reason to stop.** The census is a count, not a
verdict; it was used once in this excursus to hesitate and that was wrong. Every codebase has
panics. This stone removes ten specific arms on one path.

## Rows

| # | what | how it is judged |
|---|---|---|
| 1 | **The satisfier decision** | (a) or (b), decided by reading `launch`'s six parameters against what a queue can supply, **named one by one**. ⛔ A choice without that reading does not score. |
| 2 | **Surface unchanged** | `:wat::bracket::map` / `::each` signatures byte-identical. Show the diff is empty for both `defmacro` forms. |
| 3 | ⭐ **The ten arms are replaced, not moved** | Each of the 5 `TimedOut` and 5 `Malformed` sites on the queue path placed in RETRY / REPORT-FINAL / REPORT-GONE. `Malformed` is **never** retried. Every report names its outcome; every bound is wall-clock and says which bound it hit. |
| 4 | ⭐ **The control, by mutation** | The induced-fault reproducer **red on the peer path, green on the queue path, same commit.** A green-only control fails this row. |
| 5 | **Scope wall** | bracket's other 20 sites, queue's 72, and stone 1b untouched. Say what you did not do. |
| 6 | **Load order** | `(:wat::deporder::verify-stdlib)` → `[]`. Any manifest change named. |
| 7 | **Floor** | `scripts/floor.sh`, release. Summary line verbatim + `.floor/<stamp>/`. A red: do not re-run, capture whole, name the arm. Clippy counted over the WHOLE output. |

⚠ **Expect the Tier B gate to fire if you add a `defservice`.** `no_stdlib_body_names_a_user_declarable_name_in_evaluated_position` pins quoted `:user::main` at 10 and `:user::spawn::service-locus` at 11. An eleventh `service-forms` is a legitimate bump **with the reason recorded at the assertion** — it is not a licence to edit the pin for any other reason.

## What would make this stone wrong

- **The queue path silently serialises what the peer path parallelised.** Brackets exist for
  parallelism. Measure throughput on both paths, not just correctness.
- **A retry loop with no wall-clock bound**, or one that counts attempts instead. That is the defect
  the governing DESIGN names as ruling #1.
- **`Malformed` retried anywhere** — a defect by contract, not a judgement call.
- **The control passes because the fault never fired.** Non-vacuity: show the induced fault actually
  occurred on both runs.

## Deliverable

`SCORE.md`: the row-1 parameter reading, the ten arms with their dispositions, the mutation evidence
for row 4, deporder output, floor Summary + `.floor/` path.

⚠ Paths in pulsare messages are repo-root-relative (`wat-rs/docs/…`).

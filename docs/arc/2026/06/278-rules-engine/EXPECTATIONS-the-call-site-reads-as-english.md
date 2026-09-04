# EXPECTATIONS — the call site reads as English

Written **before** the strike.

| # | what | expected |
|---|---|---|
| 1 | ★★ **one call site, read aloud** | `(Alarm :delay (Milliseconds 50) :op :-tick)` — *"alarm, delay 50 milliseconds, op -tick."* If it needs `service.wat:52` to parse, the stone failed |
| 2 | ★★ **the stutter is gone** | `service.wat:1618-1622` no longer carries two `after`s as different parts of speech |
| 3 | ★ the finder's census | reported **before** applying, against my hypothesis: 74 `:after` occurrences / 25 files (some may not be `Alarm`'s); 99 constructor sites, `Millisecond` 87 |
| 4 | both codemods idempotent | re-run → 0 changes |
| 5 | `Microsecond` renamed too | zero call sites; renamed for symmetry, and said so |
| 6 | purity unchanged | seven constructors stay Pure + Deterministic |
| 7 | scope | one field in `service.wat`; no seam work |
| 8 | the floor | `5214/5214` |
| 9 | the circuit | `distinct=8000; dup=0`, five runs |

## ⛔ ROW 1 IS READ, NOT MEASURED — and that is the point

Every other stone this campaign had a number. This one has a sentence. The contract decision is
whether a line of code parses as English on first reading, and **the only instrument for that is
reading it.**

So quote it, before and after. A green row here is a quotation, not a count.

## RUNTIME PREDICTION

**60–90 minutes**, most of it the BOOTSTRAP dance and two codemods. The edits are small; the ordering
is the work.

## TRAP-DOOR RISKS

1. **`:after` is a bare keyword.** Form, not token — STOP-1. Eight censuses have died on this.
2. **BOOTSTRAP.** Stdlib is frozen at build time; the tool cannot boot to fix its own stdlib
   mid-flight. `fix.wat`'s header is the supported path.
3. **`Millisecond` is 87 of 99 sites** — the bulk, and the one most likely to appear in prose the
   codemod cannot reach. Report the comment sites.
4. **Doctrine 1** protects primitive type keywords in value position; the constructors are values, not
   types. Renaming them should not touch it — if it does, that is a finding.
5. **This closes no defect**, so a red anywhere is pure cost. There is no upside trade to make.

## WHAT WOULD MAKE ME REJECT A GREEN REPORT

- Row 1 reported as done rather than **quoted**.
- Row 3 reported as my numbers rather than the finder's.
- `Microsecond` skipped for having no callers.
- Any other `service.wat` change riding along.

# EXPECTATIONS — the `:impls` completeness guard

Written BEFORE the strike. Scored against my own re-run.

| # | what | the command | expected |
|---|---|---|---|
| 1 | ★ the partial satisfier is rejected | `--check probes/red-partial-satisfier.wat` | rejected, naming the service, the surface, and **every** missing op — not just the first |
| 2 | ★ the complete satisfier compiles | same file | not named. If it is, the rule is not `features ⊆ impls` |
| 3 | ★ an extra INTERNAL arm compiles | same file, a satisfier with `-tick` | not named (STOP-1). This is what a symmetric rule breaks |
| 4 | the real corpus still compiles | the census, RUN | reported in the SCORE. Only the red probe rejected → ship. Live-code hits → a finding to report (STOP-2) |
| 5 | self-scheduling services survive | `--check wat/telemetry/span.wat` | clean — it carries five features plus two internal arms |
| 6 | parametric surfaces survive | `--check wat/cache.wat wat-tests/service-cache-lru.wat` | clean, or STOP-4 |
| 7 | no runtime change | `git diff --stat src/runtime.rs wat/service.wat` | empty |
| 8 | no rune on the criterion | `grep -nE '^\s*;;\s*rune:' probes/red-partial-satisfier.wat` | none. **Match the FORM, not the token** — prose about runes is not a rune |
| 9 | the probe is in `probes/` | its path | `docs/arc/…/probes/`, never `wat-scripts/` (STOP-3) |
| 10 | the error teaches | read the rejection | service + surface + all missing ops, in one message |
| 11 | floor | `./scripts/floor.sh` — Summary line, never a piped exit code | 5162+ run, 0 failed, FLOOR=0 |

**Runtime prediction:** 45–90 minutes, dominated by resolving `:satisfies` to a surface's feature
names at check time and by the census.

## Trap doors, named in advance

- **A symmetric rule.** `impls == features` rejects every internal op and therefore every
  self-scheduling service. Rows 3 and 5 exist only for this.
- **Firing on nothing.** The one known instance is already fixed, so a guard that never fires passes
  rows 2–11. **Only row 1 catches it**, which is why the red probe is the stone rather than colour.
- **Naming one missing op.** Row 1 says *every*. A five-op gap must not be five cycles.
- **Runing the acceptance criterion** to quiet a red floor — refused correctly on excursus 002
  stone 1, and row 8 makes it an automatic fail here.

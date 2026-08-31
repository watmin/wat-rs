# EXPECTATIONS — excursus 002 stone 1

Written BEFORE the strike, so the result cannot move the goalposts. Every row is scored against my
own re-run, never the executor's report.

| # | what | the command that checks it | expected |
|---|---|---|---|
| 1 | the real case-1b escape is rejected | `./target/release/wat --check wat-scripts/scratch-pad/probe-handle-to-surface-relation.wat` | REJECTED, naming `:hs::dial-and-drop-is-the-real-escape` — with the rune from row 8 it checks clean instead |
| 2 | the `conn` helper beside it still compiles | same file, same run | `:hs::conn-is-safe-the-caller-owns-the-handle` is NOT named. If it is, the rule was keyed on the param — the single most likely way to get this wrong |
| 3 | the 16 safe sites all still compile | `grep -rn --include=*.wat -E '\-> \(:wat::kernel::Peer' tests/ wat-scripts/ wat/ wat-tests/ examples/` then `--check` each file | zero rejections among them, **stdlib `stdio-connect-{out,err,in}` included** |
| 4 | the census still says 2 and 16 | the grep above, counted | 18 total; if this moved, the acceptance criterion moved and rows 1–3 must be re-derived, not assumed |
| 5 | case 1a (let-value escape) fires | `:sev::dial-and-drop` in `tests/services/probe_severed_reaches_the_client.wat:68` | rejected pre-rune. 1a and 1b are DIFFERENT sites; a strike that only does the fn-return half passes row 1 and fails this |
| 6 | no runtime change | `git diff --stat src/runtime.rs` | empty |
| 7 | the severed gate still proves what it proved | `cargo nextest run --release -E 'test(probe_severed_reaches_the_client)'` | 2 passed. Stress it: 30 iterations, 0 failures (the mechanism is measured racy at 6/10 in the tight shape; this shape held 90/90 before the stone, and must still) |
| 8 | the rune is a rune, not a retreat | `git diff tests/services/probe_severed_reaches_the_client.wat` | a `rune:` naming this wall with a reason. **A wildcard, a deleted assertion, or a weakened rule is a FAIL even if the floor is green** |
| 9 | the floor | `./scripts/floor.sh` — read the Summary line, never a piped exit code | 5131 run / 5131 passed / 15 skipped, FLOOR=0. A red is a red: do not re-run, name the arm |
| 10 | case 2 was not smuggled in | `grep -in 'tail' src/check.rs \| grep -v 'strip_prefix\|detail'` | still no tail-position concept. STOP-4 says case 2 is a separate stone |
| 11 | the error teaches | read the rejection text | names the escaping peer's service, the `/start` span that created the handle, AND the escape span. A message naming only one of the two spans sends the reader hunting — which is the 38-day failure this whole excursus came out of |

**Runtime prediction:** 40–90 minutes. Case 1b is a few lines at a site that already holds both
facts; case 1a is the same idea in `infer_let`; most of the cost is the service/surface relation and
the error's spans.

## Trap doors, named in advance

- **Keying on the parameter instead of on creation.** Rejects every `conn` helper. Row 2 and row 3
  exist only to catch this; it is the error the first draft of this design made.
- **A Handle is PARAMETRIC** — `(:c2::alpha::Handle :- [:wat::kernel::Shared])`. Matching a bare
  path misses every handle and the wall silently does nothing. A wall that fires on NOTHING passes
  rows 2, 3 and 9 — only row 1 catches it. Treat a green floor with no rejection as a FAILURE.
- **Doing only 1b.** Rows 1 and 5 are different sites on purpose.
- **The rune collision** (STOP-3) arriving as a surprise mid-strike and getting "solved" by
  softening the rule.

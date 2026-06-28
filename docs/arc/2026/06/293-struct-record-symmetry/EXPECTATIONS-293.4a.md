# EXPECTATIONS — 293.4a: method members in `defsurface` (parse + satisfy)

Independent scorecard, fixed BEFORE the strike so the result cannot move the goalposts. Scored by the orchestrator's
own re-run, not the executor's say-so.

| # | what | command | expected |
|---|---|---|---|
| 1 | the 293.4a probe flips GREEN (un-ignored) | `cargo nextest run --release -E 'test(method_member_surface_parses_and_is_satisfied_by_a_defn)'` | PASS (no `--run-ignored` needed once un-ignored) |
| 2 | a method member PARSES | the probe's `.wat` startup no longer throws `MalformedDecl "triple is incomplete"` at the `(area [self] -> :f64)` line | startup Ok |
| 3 | a method member is SATISFIED by a `defn` | the probe's `(:t::accept (:t::Sq …))` type-checks — Sq backs `area` with `defn :t::Sq/area`, no `:satisfies` | type-checks |
| 4 | a MISSING method = NOT satisfied (negative) | add (in the probe or a sibling) a record with `color` but NO `defn :T/area`, passed to `accept` | rejected (surface not satisfied) — the executor SHOULD add this negative arm; if absent, orchestrator adds it at weigh |
| 5 | the acceptance demo stays RED (untouched) | `grep -n '#\[ignore' tests/types/probe_arc293_acceptance_demo.rs` | still `#[ignore]`'d (293.4d's gate, not this slice) |
| 6 | whole workspace green | `cargo nextest run --release` | `4086 passed / 0 failed / N skipped` (floor 0 — the campaign lint stays GREEN; the new probe adds 1 pass; the demo stays skipped) |

## Independent prediction
- **Runtime:** 35–60 min. A new `SurfaceMember` enum is a real (but small) cascade — every `SurfaceDef.members` reader
  grows an arm; the parser member-walk + the satisfaction-resolver are the two pieces of actual design.
- **The load-bearing rows:** #1 (probe green) + #6 (no regression). #4 (negative) proves satisfaction is a real check,
  not a parse-only stub that always-accepts.

## Trap-door risks (named)
- **The method-resolver seam (STOP-1).** The real risk: `check.rs:14380` may not have the `defn :T/name` table in a
  closure-reachable place. If the executor STOPs here, the orchestrator re-plans (the resolver may need threading through
  the satisfaction call from a higher frame). A satisfaction that checks name-existence WITHOUT the sig is a false pass —
  weigh #3/#4 carefully against the disk for this.
- **Always-accept regression.** If the satisfaction extension is written so a method member is *ignored* (treated as
  trivially satisfied), #1 and #6 pass but #4 fails — and #4 is what catches it. Do not let #4 be skipped.
- **The shared-sig-parser error-type clash (STOP-3).** Acceptable resolution: copy the sig-parse shape into `surface.rs`.
  Not a defect; note it in the SCORE.

## What "done" means
#1, #3, #4, #6 all green by the orchestrator's own re-run; #5 confirms the demo is untouched; the diff is read
end-to-end; satisfaction is a REAL sig-check (not name-existence). Then commit on green; un-ignore stays.

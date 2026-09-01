# HANDOFF — the `:impls` completeness guard

You are making a partial satisfier a **check error**. Today a `defservice` can declare
`:satisfies <Surface>` and implement only part of it: it compiles, freezes, and the floor stays
green, because `serve-op-arms` folds over `:impls` and an unimplemented surface op simply gets no arm.

Start here, in order:

1. `DESIGN-STONE-impls-completeness.md` — the one-directional rule, and why this brief has no census.
2. `BRIEF-impls-completeness-guard.md` — the rooms as exact `file:line`, four STOP triggers.
3. `NOTE-impls-completeness-is-unenforced.md` — the finding, with its verified instance.

Three things carry the risk:

**`features ⊆ impls`, never equality.** `:impls` legitimately carries arms the surface does not
declare — the reactor-internal ops, leading dash: `-flush-logs`, `-flush-metrics`, `-tick`. A
symmetric rule rejects every self-scheduling service in the tree. `wat/telemetry/span.wat` is the
case to check yourself against: five declared features, seven arms.

**The red probe is the stone, not colour.** The one known partial satisfier has already been fixed,
so a guard that fires on nothing passes almost every row. Write the probe first — a partial
satisfier that must be rejected, a complete one that must compile, and one with an extra internal arm
that must also compile — in `docs/arc/2026/06/278-rules-engine/probes/`, never `wat-scripts/`, and
with no rune on it.

**The census is yours to produce, and it is a finding.** This brief deliberately contains no number:
I attempted a census and it was noise (it reported a complete service as missing five ops that belong
to a different surface). Build the guard, run `--check` across the corpus, and report what it names.
Only the red probe → ship. Live code → report before fixing. Broad rejection → the rule is wrong;
STOP.

The floor is `./scripts/floor.sh`. **Read the Summary line, never a piped exit code.** A red is a
red — do not re-run, name the arm, surface it.

Write `SCORE-impls-completeness-guard.md` when done. It will be graded by re-running.

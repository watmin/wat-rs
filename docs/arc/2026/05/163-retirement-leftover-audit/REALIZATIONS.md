# Realizations — Arc 163

A continuation of the substrate-as-teacher lineage (arc 111 named the
pattern, arcs 112/113 layered the progress-meter and substrate-author
audience observations, arcs 114/115/117 applied it to verb retirement +
walker variants). Arc 163 carries the pattern across two more wide
substrate sweeps (slice 3e + slice 3f) and codifies how the
**orchestrator** must consume the doctrine — because this session
showed how easily that step gets skipped.

## What slices 3e + 3f added to the lineage

Both slices were Pattern 3 symbol migrations applied at substrate-
internal scale: bare → FQDN for container heads (3e) and primitive
paths (3f). The mechanism was textbook — sweep substrate writes,
flip canonicalize step, iterate from `cargo test` failures until the
diagnostic stream goes silent.

Slice 3e waterfall: 848 → 129 → 127 → 121 → 28 → 7 → 0. Seven sweep
iterations. Each round dropped a category. Each category was named by
the diagnostic from the previous round.

Slice 3f waterfall: 2041 → 2 → 0. Two iterations because the discipline
was settled and the spawn carried a properly-shaped BRIEF (no tool-
availability preamble; clean substrate-as-teacher framing).

The *delta* between slices is measurable: slice 3e took ~60 minutes of
iteration plus ~1.5 hours of orchestrator-side dodging before the
discipline clicked. Slice 3f took ~11 minutes total because the
discipline carried over.

## The orchestrator failure mode this arc surfaced

Arcs 111–113 named substrate-as-teacher for **agent delegation**: the
agent reads diagnostics, applies fixes, iterates. The mechanism worked
by design.

This arc surfaced an **orchestrator-side** failure mode: when the
substrate flips state and `cargo test` reports a large failure count,
the orchestrator's reflex is to TREAT THE COUNT AS A CRISIS rather than
as the substrate teaching. The reflexive responses — "stash + revert,"
"step back and re-plan as a multi-day arc," "want me to enumerate all
categories first?" — *break* the discipline at the orchestrator layer
even when sonnet would have executed it cleanly.

The user direction that broke through:

> *"i expected a fuck ton of errors - we need to do the hard work to
> clean it up... go study the arcs after 109..."*

The cost of NOT trusting the doctrine: ~1.5 hours of probing past
reflexive bridges. Same shape as arc 144 / arc 146's "missing union
types" framing × 3 drafts (recovery doc FM 10), but applied to
substrate-wide migration sweeps.

The fix codified: recovery doc FM 15 — *"failures from substrate-wide
migrations are work items, not crisis."* Memory `feedback_substrate_
teacher_failures_are_data.md` carries the discipline across compaction.

## A second orchestrator failure mode

Sonnet's false-Bash-denial hallucination (FM 7 in the recovery doc)
is well-documented. This arc surfaced the *prevention* angle: ANY
mention of tool-availability in a sonnet BRIEF — even preemptive
("Bash works", "do NOT claim denied") — TRIGGERS the hallucination.

Two re-spawns with progressively-stronger "Bash works" framing both
hallucinated denial. Slice 3f's spawn omitted tool-availability
mention entirely → no hallucination, clean execution.

The fix codified: recovery doc FM 16 — *"don't preempt; just give
the work."* Memory `feedback_no_tool_preamble_in_briefs.md` carries
the discipline.

## What this arc confirmed about the doctrine

The substrate-as-teacher mechanism scales to **270+ substrate sites
across 8+ files + 5 crates** without writing arc-specific migration
hint helpers. Slice 3e + slice 3f together swept ~290 substrate writes
across the workspace. The diagnostic stream named every site that
needed updating; the orchestrator (and sonnet) read errors and applied
the FQDN rule per category.

No `arc_163_migration_hint` was needed because the substrate already
emits the right diagnostics:
- TypeMismatch errors with `expected: ...; got: Parametric { head: ... }`
  — the head string IS the migration brief
- BareLegacyContainerHead / BareLegacyPrimitive walker errors —
  Pattern 3 from the doctrine, names the FQDN form in the message
- Dispatch arm-not-matched errors with `(args)` showing actual types
  — points at registration-side bare-form misalignment

The doctrine extends naturally:

> **The substrate's existing diagnostic vocabulary IS the migration
> brief for substrate-internal sweeps too.** Arc 111 said "the brief
> to sonnet collapses to: run cargo test; read errors; iterate until
> green." Slice 3e/3f confirmed: that same brief works for sweeping
> SUBSTRATE-side bare-form storage, not just user-side fixtures. The
> orchestrator just has to TRUST the loop and stop reflexively
> proposing escape hatches.

## The four-question cost ledger

Per `feedback_four_questions.md`, every architectural decision has a
trade-off. This arc paid for two specific costs:

| Decision | Cost paid | Cost saved |
|---|---|---|
| Slice 3e canonicalize-upgrade arms (temporary scaffolding) | bridge code lives in substrate until slice 3h retires it | ~4040 wat-fixture sweeps deferred to slice 3g; slice 3e ships green without breaking every test |
| Slice 3e revert of `Value::type_name` primitive arms (bare during slice 3e, FQDN at slice 3f) | two-phase migration; mixed convention briefly | atomic slice boundaries; each slice independently verifiable |
| Substrate-as-teacher iteration vs upfront category enumeration | `cargo test` runs ~5 times per sweep | no premature enumeration; the substrate reveals real categories, not predicted ones |

The third row is the load-bearing one. Trying to pre-enumerate all
categories before sweeping (recovery doc FM 15 anti-pattern) costs
*more* time than running the loop, because the substrate finds
categories the orchestrator's prediction misses. Slice 3e found
**wat-macros codegen** as a category I never grep'd for; slice 3f
found **type_to_affinity in wat-telemetry-sqlite** the same way.

## The third audience for the diagnostic stream

The doctrine names three audiences for the substrate's diagnostic
stream: humans, agents, orchestrators (per arc 111 REALIZATIONS).
This arc adds clarity to the third role:

> **The orchestrator is the one most likely to misread the stream.**
> Humans see one error, fix one site. Agents iterate per the brief.
> Orchestrators see the count and have to decide: trust the loop or
> propose escape. The escape is the failure mode.

The orchestrator's reading discipline post-arc-163:

1. **Count drops** = work happening. Don't stop the loop.
2. **Count plateaus** = next category surfacing. Read the next failure
   message. Don't stop the loop.
3. **Build broke** = sweep regressed. Stop. But don't propose stash;
   read the error and reverse the offending edit.
4. **Substrate change requires rust-side companion change** (e.g.
   slice 3e's wat-macros codegen) = name the category, sweep it,
   continue the loop.

## What slice 3f added on top of slice 3e

Slice 3f shipped **without orchestrator-side dodging**. The user did
not need to break through. The BRIEF + EXPECTATIONS + spawn went
clean from the start.

The pre-flight checklist worked:

- ✅ DESIGN reflected the slice + closure dependency (slice 3h gate)
- ✅ BRIEF named categories AND the iterate-from-diagnostic discipline
- ✅ BRIEF omitted tool-availability mention (FM 16 prevention)
- ✅ EXPECTATIONS scored hard rows + named honest deltas to flag
- ✅ Spawn explicit `model: "sonnet"` + `run_in_background: true`
- ✅ Substrate-as-teacher pattern doc cited in BRIEF + spawn prompt

Sonnet's report mode-A-confirmed at 2041/0 with two clean iterations
and four explicit honest deltas. Total: ~11 minutes wall-clock + zero
orchestrator-side rescue cycles.

## Cross-references

- `docs/SUBSTRATE-AS-TEACHER.md` — the doctrine
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 15 (orchestrator-side
  failure mode this arc surfaced)
- `docs/COMPACTION-AMNESIA-RECOVERY.md` § FM 16 (sonnet-prevention
  failure mode this arc surfaced)
- `docs/arc/2026/04/111-result-option-recv/REALIZATIONS.md` — the
  pattern's first naming (where this lineage begins)
- `docs/arc/2026/04/113-cascading-runtime-errors/INSCRIPTION.md` —
  third application + integ-test verification
- Memory `feedback_substrate_teacher_failures_are_data.md`
- Memory `feedback_no_tool_preamble_in_briefs.md`

## Coda

Arc 111's REALIZATIONS closed with:

> *"the user supplied the will. The substrate supplied the loop. The
> agent supplied the patience. Don't tell Gödel."*

Arc 163's continuation: **the orchestrator supplies the trust.** The
loop only works when the orchestrator reads the count as data, not
crisis. Trusting the doctrine is the discipline; proposing stash is
the failure.

The lineage continues. The migration hint pattern retires per arc when
its window closes (`arc_111_migration_hint` was retired by arc 111
slice 5; arc 163's canonicalize-upgrade arms retire at slice 3h). The
*meta-discipline* — that the substrate's diagnostics are load-bearing
across humans, agents, AND orchestrators — survives forward because
the recovery doc carries it across compactions and the REALIZATIONS
sequence carries it across arcs.

(Three audiences, one stream. Now four, including the orchestrator
who learns to trust it.)

# DESIGN — the malformed census

Builder's ruling **D5-c**: *"CENSUS THEN SEQUENCE — a report-only rule finds every raising `Malformed` arm;
the builder sequences the real ones. The rule must NEVER auto-fix."* Plus the scope constraint that governs
everything downstream:

> *"the placeholders... we are only modifying **service items** right now.... we'll chase getting rid of
> `raise` at large later."*

**Drawn 2026-09-13. NOT STRUCK.** Fourth of the five.

## ⛔ FIRST — THE POPULATION IS 487, NOT 61, AND THAT DECIDES THE WHOLE SHAPE

```
RecvOutcome::Malformed   487 occurrences   over wat/ wat-scripts/ tests/
```

The record's *"61 live placeholder arms"* is a **subset** someone once separated by hand. So:

★★ **The census's value is its CLASSIFICATION, not its list.** A 487-row report is useless, and even a
59-row one is useless if the builder must open 59 files to apply *"service items only."* **The census must
carry the axis the constraint cuts on**, or it has not done the job.

## ⭑⭑ AND IT IS NOT A `wat/lint.wat` RULE — the builder's own constraint rules that out

I proposed a lint rule in the four-questions. Two facts on disk say otherwise:

1. **`lint-source [files]`'s rules are HARDCODED** (`wat/lint.wat:579`) — there is no `Rule` type and no way to
   pass one in. A new rule is a **stdlib edit**.
2. ⛔ **A permanent gate that fires ~59 times is noise, not a gate.** The builder deferred *"getting rid of
   `raise` at large"* — so a stdlib rule added now would report a population they have explicitly chosen not
   to work through yet, on every lint run, indefinitely.

> **So: a census PROGRAM under `wat-scripts/`, not a stdlib rule.** The permanent lint rule belongs to the
> later *kill-raise* campaign, when the population is near zero and a gate would hold a line instead of
> describing a backlog.

★ The precedent is this session's own tooling: `wat/fix.wat`'s codemods carry a `--grep` **finder** mode that
reports without applying (`:user::grep`). A census is that, with no apply half at all.

## The classification axis — derivable, not guessed

`lint-file` runs over **every top-level form**, so a walker knows the enclosing form's head. Two axes, both
mechanical:

| axis | values | why it is the right cut |
|---|---|---|
| **enclosing top-level form** | `defservice` · `defsurface` · `defn` · `deftest` · other | *"service items"* is exactly `defservice`/`defsurface` — the builder's filter becomes a column, not a judgement |
| **home** | `wat/` (stdlib) · `wat-scripts/{queue,topic,fanout}` (userland services) · `wat-scripts/scratch-pad` (probes) · `tests/` (fixtures) · `docs/` | probes and fixtures are where **dying loudly is correct** — they must be separable at a glance |

Plus, per arm: **does the body raise, or return a value?** Only the raising ones are findings.

## ⛔ THE RAISE IS THREE FORMS, NOT ONE

```
(:wat::kernel::assertion-failed!   3654
(:wat::kernel::raise!               10
(:wat::kernel::panic!                1
```

A census keyed on `assertion-failed!` alone **misses 11 sites.** ★ *Match the FORM — "this arm's body
raises" — not the token.* This is the same defect that made five of my own greps wrong this session, written
down before the instrument is built rather than after.

## The one contract decision

> **The census REPORTS and never writes.** No `fix`, no apply mode, no auto-migration — not even for the
> arms whose cure looks identical to the two already done.

★ The two migrated arms were **tears mislabelled** (an oversized frame severs the sender; the service lives),
which is a fact about *that* call site derived from *that* probe. **No rule can know that**, and a template
applied across a class is precisely `[[feedback_a_fallback_that_collapses_failures_reports_nothing]]`.

## ⭑ The instrument must pass a TWO-SIDED control before its output is quoted

- ⛔ **Must NOT flag** `circuit.wat`'s and `sns-fanout.wat`'s poisoned-call arms — migrated at `15ddc5e35`,
  they now return `"malformed"`. If they appear, the "does it raise" test is wrong.
- ⛔ **Must flag** a known raiser — e.g. a `wat/` arm carrying the `UNMIGRATED PLACEHOLDER` text.
- ⚠ **And the arms are on ONE LONG LINE** carrying several arms at once: my own grep for the migrated form
  returned lines that *also* contain `assertion-failed!` from a neighbouring `Stopped` arm. **A line-oriented
  instrument cannot do this job** — the census must walk **forms**.

## The four questions

**Obvious?** YES — a table with two columns the builder's filter reads directly. **Simple?** YES — one
walker, one classifier, no writes, no stdlib change. **Honest?** YES: it reports a population without
implying a work list, and it says which rows are probes where raising is *correct*. **Good UX?** YES — the
builder applies *"service items only"* by reading a column instead of opening 59 files.

## Scope

**IN:** a `wat-scripts/` census program over all `.wat` homes · per-arm classification on both axes · the
raise/return distinction across **all three** raise forms · the two-sided control, stated in the output ·
the report committed as the artifact.

**OUT = REJECTED:**
- ⛔ **Migrating ANY arm.** Census only. The builder sequences.
- ⛔ **A `wat/lint.wat` rule.** Reasoned above; belongs to the later kill-raise campaign.
- ⛔ **Deciding what "service item" means for borderline rows.** Report the two axes; the builder's filter is
  theirs to apply. ⚠ If a row is genuinely ambiguous, say so in its row rather than resolving it.
- **The other outcome variants' placeholder arms** (`Lost`, `Closed`, `TimedOut`, `Stopped`). Same shape, and
  a much larger population — this census is `Malformed`, as ruled.

## Trap-doors

1. ⛔ **A line-oriented instrument cannot do this** — multiple arms share one line. Walk forms.
2. ⛔ **Three raise forms**, not one.
3. ⛔ **Do not quote a count before the two-sided control passes.** Five of my greps were wrong this session
   and every one was caught by a control.
4. **`487` is occurrences of the token, not findings.** The finding count will be smaller and the difference
   must be explained, not glossed.
5. **Report-only means no `FixEdit`** — `Finding.fix` stays `None` if the `Finding` shape is reused at all.
6. **`wat-scripts/**/*.wat` is type-checked by the floor**, so the census program itself must load cleanly.

# DESIGN — boot names where its time goes

**Drawn 2026-09-17**, builder-directed: *"you're asking if i want a 99.5% speed up - what do you think my
answer will be ... this feels like another [deep flaw]"*. **NOT STRUCK.**

⛔ **THIS STONE MEASURES. IT DOES NOT OPTIMISE.** Not one microsecond may be made faster by it. The
reason is in this campaign's own record: `3f159b0c5` (2026-08-26) is a commit whose entire body is a
retraction for *"describ[ing] a mechanism and prescrib[ing] a remedy in the same breath, having measured
neither"*, on **this exact subject**. Its closing line — *"a prescription is a claim"* — is the rule this
stone obeys.

## What is already measured, and what is not

Established (`FINDING-boot-is-99-percent-stdlib.md`):

```
binary start, stdlib NOT loaded (--help)   0.002 s
binary start, do-nothing program           0.425 / 0.432 / 0.438 s     ⇒ 99.5 % is the stdlib
vs 2026-08-26                              147 ms → 430 ms  (+193 %) on +17 % source ⇒ ~2.5× superlinear
marginal cost of one service-bearing file  +1,910 lines ⇒ +126 ms ≈ 66 µs/line vs a 19 µs/line average
```

**NOT established: which PHASE owns the 430 ms.** Two independent readings point at *macro expansion*,
and **both are inferences from marginal cost, not a profile.** That gap is this stone.

## The work

**Env-gated boot instrumentation**, default off, that prints a breakdown of one boot to stderr:

1. **per PHASE** — the real phases as the loader actually has them. ⚠ Do **not** assume my names
   (parse / expand / check / freeze); read `src/load/` and report the phases that exist. If the code has
   no seam where a phase boundary should be, **that is a finding** — say so rather than inventing one.
2. **per MANIFEST ENTRY** — all 55, so the curve is visible and the expensive files are named.
3. **a total that reconciles** with the externally measured wall clock.

Precedent for the shape, all from this session: `WAT_STARTUP_HANDSHAKE_DEADLINE_MS` (env-gated, read
once, default preserved), the ring census (`new_ring` funnelling every site so the count cannot drift),
and the CI runner-facts step. Follow them: **off by default, zero cost when off, and the instrument
itself under test.**

## Expectations of the numbers — state them before measuring, then compare

Written down first so the measurement can contradict them:

- **expansion dominates.** Predicted from two marginal readings. If the profile says parsing or
  type-checking instead, **say so loudly** — that would redirect the whole attack.
- **the per-file curve is lumpy, not flat**, with `defsurface`/`defservice`-bearing files far above the
  average. 17 such forms exist across the stdlib.
- **the phases sum to ~430 ms ± noise.** A large unexplained remainder is itself a finding: it means the
  cost is somewhere nobody has a name for.

## Trap-doors

1. ⛔ **Do not optimise anything.** Not a `clone`, not an allocation, not a lookup. A stone that measures
   and fixes cannot report honestly about either.
2. ⛔ **Do not raise a timeout, and do not touch `.config/nextest.toml`.** Raising them is the habit this
   finding exists to end.
3. ⛔ **Measuring the measurer.** Per-file timing around 55 entries is cheap; per-form timing may not be.
   Report the instrument's own overhead, and if it distorts the total, say which numbers are affected —
   `[[feedback_the_measurement_contains_the_measurer]]`.
4. **Off by default, and prove it**: with the env var unset, boot must be indistinguishable from today's
   0.425–0.438 s band.
5. **The floor must stay green.** This adds a diagnostic, not a behaviour.
6. **Batching is already refuted** (`3f159b0c5`: a batched gate reported one type-check error and
   stopped, leaving 97 of 98 rows unexamined). Do not re-propose it as a finding.

## Out of scope

- **Any fix**, including precomputing the freeze. That is the builder's ruling, and this stone exists to
  inform it.
- **The queue promotion**, parked on `queue-promotion-blocked-on-startup-cost` (`6136d144f`). It is
  blocked *by* this cost and must not be un-blocked by touching timeouts.
- **`wat-scripts/` load-gate redesign.** Its 687 startups are a consequence, not the cause.

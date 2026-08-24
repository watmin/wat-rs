# NOTE — the anti-graveyard gate is scoped by EXTENSION, and 9 files have rotted through the gap

**Filed 2026-08-23**, from R5's report during the prose sweep, confirmed by my own measurement.

## The gate and its promise

`wat-rs/CLAUDE.md`, on why scratch `.wat` belongs in `wat-scripts/`:

> *"the `every_wat_scripts_file_loads` gate parses + type-checks **every** `.wat` under `wat-scripts/`
> (recursively, incl. `scratch-pad/`) on the current runtime — so a scratch program that rots goes RED
> and **cannot become a graveyard that reads like live code**. All wat stays correct, always."*

## The gap

The gate selects by **filename extension**. Renaming a file exempts it:

```
wat-scripts/**/*.wat.disabled     11 files
wat-scripts/intueri/*.wat.intueri 11 files
```

Measured — **9 of them carry retired syntax in CODE, not comments**:

```
wat-scripts/ping-pong-fork.wat.disabled          wat-scripts/count-logs.wat.disabled
wat-scripts/ping-pong.wat.disabled               wat-scripts/metrics-summary.wat.disabled
wat-scripts/seed-fixture.wat.disabled            wat-scripts/demos/aggregates/showcase.wat.disabled
wat-scripts/intueri/peer-pid-accessor-naming.wat.intueri
wat-scripts/intueri/bracket-loci-vocabulary.wat.intueri
wat-scripts/intueri/s3-client-vocabulary.wat.intueri
```

R5 confirmed these fail `--check` **identically before and after** its edits — the rot is pre-existing
and unrelated to the sweep. It correctly reported them and did not fix them: its remit was comments.

★ **The gate asks "does the name end in `.wat`" when the property it means is "is this a wat program".**
A check scoped to a spelling rather than to the property it protects — the same shape this arc has paid
for repeatedly, here in the very mechanism written to prevent graveyards. And the exemption is trivial
to take: it is one rename, it requires no justification, and it leaves the file looking exactly like
live code.

⚠ **`.disabled` may be deliberate.** A file parked on purpose is not the same as one that rotted, and
the two are indistinguishable from the outside — which is itself the defect. Whatever is decided, the
distinction should be visible: a parked file should say it is parked and why, and a rotted one should
be red or gone.

## What is owed

- Decide what `.wat.disabled` and `.wat.intueri` MEAN. If parked-on-purpose: the gate should read a
  declared reason, not an extension. If dead: delete them — `[[feedback_read_the_epitaph_before_you_build_on_prior_art]]`,
  a graveyard that reads like live code is exactly what the gate exists to prevent.
- Either way the selector moves off the extension, or the exemption is recorded per file rather than
  taken by rename.

## Scope

Out of the prose sweep's scope (comments only, and these are code defects). Not tracked elsewhere; this
NOTE is the record.

Kin: `NOTE-the-guides-are-not-executable.md` — the same shape again: `.md` code blocks are also wat
programs nothing gates.

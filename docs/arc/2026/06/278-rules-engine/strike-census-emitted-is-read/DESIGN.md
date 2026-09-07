# DESIGN — census G's deferred gate: a counter the engine emits must be read, or say why not

## Why

`tests/lint/census_name_read_by_a_cost_test_is_emitted.rs` enforces **READ ⇒ EMITTED**: a name a
cost test reads must be a name the engine emits, because `unwrap_or(0)` answers *"this mark does not
exist"* and *"this mark measured zero"* with the same `0`.

**The reverse is enforced by nothing.** That is how three `prod:` counters reached HEAD unobserved;
census G removed two by hand. An emitted counter nobody reads is where a name rots: the whole A–M
census was *"the counter's name says X, the quantity is Y"*, thirteen sections of it, and the only
thing that would have caught any of them at construction time is a reader that asserts on the
number.

## THE SCOPE DECISION, MEASURED — 7, not 21

Sweeping every `phase_end` / `census_count` / `census_count_n` literal in non-test `src/` against
every string literal under `src/rete/kernel/tests/`:

| emitter | emitted | never mentioned in any cost test |
|---|---|---|
| `phase_end` | — | **14** |
| `census_count` | — | **7** |
| total | 78 | **21** |

**This gate covers `census_count` / `census_count_n` ONLY, and the split is the reason.** A
`phase_end` mark names a region of the fire path and its quantity is always "time spent there" —
the name cannot drift from the measurement, and its consumer is the census TABLE, which is a
display, not an assertion. A `census_count` name asserts *what is being counted*, and that is
exactly the claim A–M found false thirteen times.

⚠ The asymmetry with the existing gate is deliberate and must be stated in the new gate's header:
READ ⇒ EMITTED covers both emitters; EMITTED ⇒ READ covers counters only.

**⛔ And 21 was my second number.** The first sweep said 54, because it implemented only the
existing gate's READ rule 1 (glyph literals) and missed rule 2 (row-reader closures). It was caught
by anchoring: that gate's own prose names `"compiled:exec"` and `"accum:index-builds"` as real read
names, and both were in my "unread" list. Re-run with those as the anchor, the number is 21.
`[[a-throwaway-sweep-is-an-instrument]]` — third correction of the day.

## The seven

```
filter:test-env-builds     filter:test-key-alloc      match:bind-insert
match:clause               match:head-miss            prod:class-alloc
rematch:compiled
```

`prod:class-alloc` is the survivor of the exact family census G removed two of.

## The one contract decision

**Each of the seven takes one of three dispositions, and "leave it" is not among them:**

1. **Wire a reader** — a cost test asserts a NONZERO value on it. This is the only disposition that
   makes the counter's name falsifiable.
2. **Delete it** — census G's own precedent, and the right answer for a counter reporting something
   nobody has ever needed. A counter costs a branch on the hot path.
3. **Declare it** — a per-name `rune:lint(census-emitted-unread) <name> — <reason>` stating what it
   is for and why no test asserts on it.

⛔ **Disposition 3 must not become the default.** A gate whose whole population is exempted is a
ratchet, and this arc has removed several. `no_stale_path_in_doc`'s 34-row `DEFERRED` fence is the
precedent to follow: fenced, worked down, **and the fence deleted rather than left standing empty**.
Whatever the split, the SCORE must state it per name with the reason, so the next hand can argue.

## Files

- new `tests/lint/census_emitted_name_is_read_or_declared.rs`
- whichever of the seven take dispositions 1 or 2 (`src/rete/kernel/`)

## Out of scope = REJECTED

- **`phase_end`.** Measured above; different failure surface, different consumer.
- Widening the READ set beyond what the existing gate defines. Its two scope cuts were each made
  against a measured counter-example and are documented; re-litigating them is a different strike.
- The A–M census rows themselves — closed.

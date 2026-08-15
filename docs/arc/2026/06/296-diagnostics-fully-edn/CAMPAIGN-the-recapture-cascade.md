# 296 · CAMPAIGN — the recapture cascade: 224 tests come out of the dark

> Builder, 2026-08-15: *"i've been wanting to kill those ignores for months.... we attack as many as
> we can here... if we've already built the tooling for this... we use it everywhere it must be used."*

## THE PRIZE, MEASURED

**250 `#[ignore]` attributes** live in the tree. **224 of them carry one identical reason:**

```
296-recapture-pending: golden asserts pre-stone-B rust-debug face;
unlock: 296 recapture (.edn data-equality flip)
```

224 tests — roughly **5% of the 4421-test floor** — have been dark since stone B replaced the Rust
`{:?}` debug face with EDN. They are not flaky, not slow, not quarantined for cause. They are waiting
on a conversion.

> **A number correction, on the record.** This was reported as *"70 ignored tests"* several times
> today, including inside a committed brief. **70 is the FILE count.** The test count is 224. Stated
> as measured; it was not.

The other **26** ignores are heterogeneous and individually dispositioned — `RED-at-HEAD` (arc 255
`metadata-of` ×6), `UNWRITTEN` (`unimplemented!()` bodies ×3), perf harnesses run on demand ×4,
`ARC-170 WIP` ×2, diagnostics meant to be read with `--ignored` ×3. **They are NOT this campaign.**
Each needs its own ruling and several are honest.

## ⛔ THE LAW OF THIS CAMPAIGN — read before you capture

`UPDATE_EDN=1` writes whatever the code currently emits. On a test that is failing **for a real
reason**, that freezes the bug into the golden and paints it green forever. Capture-don't-guess
becomes capture-the-bug.

**224 tests have been dark.** Some of them are dark over a genuine regression that nobody has seen
since stone B. That is the whole reason this cohort is worth attacking and the whole reason it is
dangerous.

So, at every wave, in this order and no other:

1. **Un-ignore.**
2. **Run WITHOUT `UPDATE_EDN`.** Read every failure.
3. **Triage each one.** Is it the expected staleness — a golden pinning the pre-stone-B rust-debug
   face where an EDN face now arrives? Or is it something else?
4. **Only the expected-staleness class gets recaptured.** Anything else is a **finding**: capture it
   verbatim, name it, report it. Do not recapture it. Do not re-ignore it to move on.

A wave that reports "N tests recaptured, all green" without a triage list has skipped the only step
that distinguishes this from mass-blessing 224 assertions.

## THE THREE TIERS — all 224 classified

| tier | count | shape | work |
|---|---|---|---|
| **T1** | **105** | already on `assert_edn_matches_file!` | un-ignore → triage → recapture the stale |
| **T2** | **101** | `assert_eq!(msg, "<inline literal>")` — an inline string pinning the `{:?}` face | convert to an `.edn` data-equality golden, then capture |
| **T3** | **16** | `assert!(… contains …)` and kin | **rebuilt, not recaptured** |

**T3 is not a recapture.** A `contains` assertion does not pin a face — it pins a fragment, and this
arc has a `no_loose_string_assert` lint against exactly that shape (it has fired on this arc's own
probes twice). Converting a loose assert to a golden silently *strengthens* it, which is right, but
each needs a reading of what it actually measures first. Smallest tier, most judgment.

## WAVE STRUCTURE — one build at a time

FM 18: N riders each running cargo against one `target/` is not parallelism, it is N-way lock
contention plus N unwinnable gates. So the waves are **serial**, and each wave's screams stay
unambiguous because only one thing changed.

- **Wave A — T1 (105).** The highest value and the cheapest: these are already converted and merely
  parked. It is also the honest non-vacuity proof that the recapture mechanism works at scale rather
  than on the one file its landing commit named.
- **Wave B — T2 (101).** The bulk. The *conversion* is text-only and could fan out safely (riders
  edit, no cargo); the *capture* is one central serial pass afterward.
- **Wave C — T3 (16).** One rider, per-test judgment, no blind conversion.

**Prerequisite for all three: H-2a must land first.** It finishes the `assert_edn_eq!` →
`assert_edn_matches_file!` migration, which is what makes `UPDATE_EDN=1` reach the corpus at all.

## WHAT SUCCESS LOOKS LIKE

Not "224 green." **224 adjudicated**: recaptured where the golden was stale, fixed where the code was
wrong, and reported where neither — with the count of each stated plainly. A regression found here is
worth more than a hundred tests turning green, because it has been invisible for the entire life of
the cohort.

## THE CLASS THIS BELONGS TO

Fifth unadopted capability found today, after `insert-all` (unused by 9/9 grid axes), `Record::of`
(surviving its own retirement), `wat_field_names_from!` (zero consumers until G reached for it), and
`assert_edn_matches_file!` itself (piloted on one file, stalled at seven).

Task **#48** exists to inventory this class and is still pending. Every instance so far has been
stumbled into rather than listed. That inventory keeps paying for itself in findings we did not go
looking for.

# SCORE — perf 3: the indexed vector update

**STRUCK.** Executor: grok, 2026-09-01. Every row re-run by me. **Two weighs — the first was RED and
handled correctly.**

```
Summary [ 345.360s] 5169 tests run: 5169 passed (3 slow), 15 skipped
FLOOR=0
```

| # | what | my re-run |
|---|---|---|
| 1 | ★ write cost stops growing | ✅ puts **138 / 267 / 554 ms** (1.93×, 2.07× — was 400/1333/4887 at 3.33×/3.67×); deletes **140 / 284 ms** (2.03× — was 751/2801 at 3.73×) |
| 2 | ★ the differentials hold | ✅ all five, `git diff tests/` **empty** — swap-remove did not trip them |
| 3 | the circuit, measured not predicted | ✅ **88.6 s** (was 257 s), `total=8000; distinct=8000; dup=0` |
| 4 | order-independence verified | ✅ by the strike, then by me |
| 5 | out-of-range is loud | ✅ *"index 9 out of range (length 3)"*, *"drop-last on empty vector (length 0)"* |
| 6 | the primitive is narrow | ✅ `set` + `drop-last` joined an existing `:wat::vector::` namespace |
| 7 | durable shape unchanged | ✅ still `(PersistentVector :- [StoredRow])` |
| 8 | sqlite untouched | ✅ empty diff |
| 9 | hibernate/resume | ✅ |
| 10 | header updated | ✅ |
| 11 | reads did not regress | ✅ scan **107 / 121 / 150 ms**, scan-index **108 / 101 / 106 ms** |
| 12 | floor | ✅ 5169/5169, my own run |

**Writes went from quadratic-trending to linear.** The contract decision held: the defect was in core,
and `mem-store`'s foldl was the only shape a language without indexed update left available.

## The red first weigh was handled exactly right

The first weigh went red on two arms — `checker_skip_debt_is_named_and_frozen` and
`every_dispatched_verb_is_classified_or_disposed` — because the new verbs were registered as
intrinsics but not in `register_builtins` / `intrinsic_meta`.

**The ARM was kept, it was not re-run, and both arms were named.** The fix was to *classify* the
verbs (TypeSchemes `(PV T, i64, T) -> PV T` and `(PV T) -> PV T`; pure and Partial, beside `conj`) —
not to weaken a gate. Both arms pass now on my run.

★ Those two gates are doing exactly what they exist for: a new verb that nothing classified is a verb
the purity analysis cannot reason about. **They caught it in the same session it was introduced**,
which is the whole point of a completeness gate.

## The arc of the three perf stones

```
                       circuit    reads (250/500/1000)      writes (puts 250/500/1000)
before perf-2           287 s     1691 / 3489 / 9204 ms     400 / 1333 / 4887 ms
after  perf-2           257 s      119 /  116 /  123 ms     (unchanged)
after  perf-3            88.6 s    107 /  121 /  150 ms     138 /  267 /  554 ms
```

**3.2× end to end**, with `total=8000; distinct=8000; dup=0` unchanged throughout. Reads went flat;
writes went linear.

## A measurement error of mine, recorded

My first pass at row 11 read **111 / 185 / 341 ms** for `scan` and an erratic 264 / 318 / 139 for
`scan-index` — apparently a regression against the report's 112/126/159.

It was not. **I had launched the circuit in the background before running the probes**, so both were
measured under a competing load. Re-run on an idle machine they are 107/121/150 and 108/101/106,
matching. The instrument was wrong, not the code — the same lesson as
`feedback_state_what_the_instrument_can_see_before_quoting_it`, in a new dress: state the *conditions*
beside the number too.

## Named, not chased

Row 3's remaining cost: the GSI partition filter on `receive`'s re-put is still a walk. This stone did
not promise to erase it, and the SCORE says so rather than presenting 88.6 s as the floor of what is
achievable.

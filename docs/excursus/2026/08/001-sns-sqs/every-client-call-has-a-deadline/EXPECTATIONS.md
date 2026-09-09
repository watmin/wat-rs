# EXPECTATIONS — every client call has a deadline

Written **before** the strike. Every "before" is a run of mine on `d960085eb`.

| # | what | command | expected |
|---|---|---|---|
| 1 | **★★ a dropped `mark` reply no longer hangs** | drop chaos aimed at `mark`, tiny ×6 | every run **terminates**; before: `drained-never` at ~160 s |
| 2 | **★★ `check` is behaviour-identical** | circuit ×5 + tiny ×6 | numbers match the before-state row for row 3/4 exactly |
| 3 | rate-0 unchanged | circuit ×5 | `total=8000; distinct=8000; dup=0`; `seen-recorded=8000` |
| 4 | tiny unchanged | tiny ×6 | completing runs `total=100; distinct=100; dup=0; seen-recorded=100` |
| 5 | **the floor** | `./scripts/floor.sh` — **the Summary line** | `5214 passed`, 19 skipped |
| 6 | the helper is the only new stdlib form | `git diff wat/service.wat` | one added `defn`, **after** the macro; nothing above `:896` moves |
| 7 | the goldens are undisturbed | floor | no `peers_bijection` failure. If one appears, row 6 was violated |
| 8 | timings, reported not gated | circuit ×5 | record `publish/drain/stop`. Before: publish `45790–46126` |

### Before-state, recorded verbatim

```
row 3   total=8000; distinct=8000; dup=0 ×5; seen-recorded=8000; seen-skipped 6–11
row 4   5/6 complete, total=100; distinct=100; dup=0; seen-recorded=100; 1/6 claim deadline exhausted
row 5   Summary [ 366.422s] 5214 passed, 19 skipped   .floor/2026-09-05T07-10-04Z/
row 8   publish 45790 45971 46126 46104 46104
```

## ⛔ ROW 1 IS THE REFUTATION ROW, AND IT NEEDS THE HARNESS POINTED SOMEWHERE NEW

The existing drop chaos hides **`check`** replies. Row 1 requires aiming it at **`mark`** —
which is precisely what the previous strike backed away from after a worker hung ~160 s.

**If a dropped `mark` still hangs, this stone has not landed**, however green rows 2–8 are. Say
so and hand it back; do not re-point the harness at `check` to get a green.

## ⛔ ROW 2 IS THE GUARD ON THE WHOLE STONE

`check` already works. Refactoring it onto the helper is the only way to know the helper is
right before trusting it at three sites that were never deadlined. **Identical numbers, or the
helper is not equivalent and the other three sites are unverified.**

## RUNTIME PREDICTION

50–80 min. The parametric signature and the `select` laundering are the hard part; the four call
sites are mechanical once the helper type-checks. Budget a rebuild after every `wat/` edit.

## TRAP DOORS, NAMED

1. **The stdlib is frozen at build time.** A `wat/service.wat` edit with no rebuild produces a
   binary that still runs the old helper, and every row here would report the *old* behaviour.
2. **`:wait` is not a deadline.** Row 3's `Queue/receive` keeps both. Removing `:wait` because
   "the deadline covers it" changes the server's long-poll and is STOP-3.
3. **The inert payload.** The timer needs a value of type `O` it never reads. If the executor
   makes that payload meaningful — a "default answer" — a timeout silently becomes a reply.
4. **A green floor proves nothing about row 1.** The floor runs rate 0 with the drop cells
   ignored; row 1 is outside it entirely.

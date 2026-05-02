# Arc 139 — Generic-T tuple return propagation

**Status:** opened 2026-05-03 as PLACEHOLDER.
**Blocked by:** arc 138 (CheckError spans). Cannot diagnose without
file:line:col on type errors.

## TL;DR

Arc 135 slice 3 surfaced an apparent substrate limitation: a function
defined as `(define :helper<T> body -> :(Thread, T, Receiver))`
appears to return `T` at runtime instead of the 3-tuple. Workaround
in slice 3 was three concrete non-generic helpers with nested
2-tuple returns — clunky. This arc fixes the substrate so users can
write the generic version.

The fix can't be designed until we can see WHICH form the type
checker rejects WHERE. Arc 138 lands first; then we re-run the
reproduction with file:line:col errors; then this arc fills in.

## Reference reproduction

`/home/watmin/work/holon/wat-rs/wat-tests/tmp-3tuple-probe.wat`:

```scheme
(:wat::core::define
  (:test::make-3tuple<T> (mid :T) -> :(wat::core::i64,T,wat::core::String))
  (:wat::core::Tuple 42 mid "hello"))

(:wat::test::deftest :wat-tests::tmp::generic-3tuple-roundtrip
  ()
  (:wat::core::let*
    (((triple :(wat::core::i64,wat::core::bool,wat::core::String))
      (:test::make-3tuple<wat::core::bool> true))
     ((a :wat::core::i64) (:wat::core::first triple))
     ((b :wat::core::bool) (:wat::core::second triple))
     ((c :wat::core::String) (:wat::core::third triple))
     ((_ :wat::core::unit) (:wat::test::assert-eq a 42))
     ((_ :wat::core::unit) (:wat::test::assert-eq b true)))
    (:wat::test::assert-eq c "hello")))
```

## What we know so far (UNVERIFIED — pending arc 138)

- Sonnet's slice 3 report claimed runtime returned T instead of 3-tuple.
- Errors lacked spans, so neither sonnet nor I could confirm where
  the failure originated.
- Possibilities, in priority order to investigate:
  1. **Type inference** — generic T inside a multi-element tuple
     return type doesn't unify properly during call-site
     monomorphization.
  2. **Tuple constructor** — `(:wat::core::Tuple ...)` arity/type
     resolution is fine for concrete T but degrades for parametric T.
  3. **Tuple destructure** — `:wat::core::first/second/third` on a
     value with parametric T type breaks accessor inference.
  4. **Display lie** — code is correct but error message
     misrepresents the failure (less likely now that we're adding
     spans + the arc-130 hint pattern).

User's stated preference: "i think i want the tuple form... that
ret val of many thing as a tuple to be decomposed is an amazing UX."
Generic-T 3-tuples must work end-to-end. The clunky workaround
(3 non-generic helpers + nested 2-tuples) is acceptable in the
short term, NOT long term.

## Slice plan (placeholder — refined when arc 138 lands)

1. **Slice 1 — Diagnose.** Re-run `tmp-3tuple-probe.wat` post-arc-138.
   Read the file:line:col error. Identify which inference rule
   misfires.
2. **Slice 2 — Fix.** Substrate fix — likely in
   `src/types.rs` or `src/check.rs` parametric tuple unification.
   Detail TBD.
3. **Slice 3 — Sweep.** Find any other places generic-T-in-tuple
   workarounds were used. Replace with the proper form.
4. **Slice 4 — INSCRIPTION + USER-GUIDE row + 058.**

## Cross-references

- `docs/arc/2026/05/138-checkerror-spans/DESIGN.md` — the prerequisite.
- `docs/arc/2026/05/135-complectens-cleanup-sweep/SCORE-SLICE-3.md` —
  the slice that surfaced the issue.
- `wat-tests/tmp-3tuple-probe.wat` — minimal reproduction (kept
  in place until arc closes).

## Why placeholder, not full spec

The user explicitly said "make tuple support proper" — confirms
intent — but the surface area can't be specified until we can
actually navigate to the failing form. We open the arc to claim
the number + park the reproduction; we fill in slice 1 when arc
138 lands.

## Sequence

```
arc 138 (spans) → re-run tmp-3tuple-probe.wat with coords
                → fill in arc 139 slice 1 (diagnosis)
                → arc 139 slices 2-4 (fix + ship)
                → unwind back to arc 135 slice 4
                → sonnet runs the suspect-tier sweep on a clean substrate
```

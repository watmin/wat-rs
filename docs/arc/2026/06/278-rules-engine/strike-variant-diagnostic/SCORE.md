# SCORE — D1's residual, weighed against the orchestrator's own re-run

> Re-run here at `f22704f1f`.

## ⚠ THE COMMIT MESSAGE ON `f22704f1f` IS MANGLED, AND THE HISTORY WAS NOT REWRITTEN

I wrote the message with backticks inside a `-m "..."` argument; the shell command-substituted
them, so three lines lost their identifiers before the commit was made and pushed. The amend that
fixed it would have rewritten already-published history on a shared branch, so **I reset back to the
pushed commit instead** — local and origin agree, nothing was force-pushed. The three damaged lines
should read:

```
  #wat.rete/UnknownEnumVariant -- defrule evt::good (:evt::Req):
  :evt::G has no variant Hii; available variants: [Hi, Lo]
```

The mechanical lesson: **a commit message with backticks belongs in a file passed to `-F`, never in
`-m`.** Every other strike this session used `-m` safely only because none of them quoted an
identifier.

## The scorecard, re-run

| # | pre-value at HEAD | after |
|---|---|---|
| 1 | `UnknownField` — *"has no field `:evt::G::Hii`; available fields: [k, grade]"* | ✅ **`#wat.rete/UnknownEnumVariant` — "defrule `evt::good` (`:evt::Req`): `:evt::G` has no variant `Hii`; available variants: [Hi, Lo]"** |
| 2 | offers `[k, grade]` | ✅ offers `[Hi, Lo]`, and the **absence is independently gated** — re-driven below |
| 3 | control green | ✅ |
| 4 | tagged arm **UNKNOWN** | ✅ **answered by driving**: it does NOT reach the new arm, and must not — see below |
| 5 | — | ✅ three routes, not the two I named |
| 6 | `_ => "keyword"` held two facts | ✅ a named three-state `KeywordConstant` |
| 7 | — | ✅ `typing.rs` + `error.rs` + probes |
| 8 | lint 116/116 | ✅ 116/116 |
| 9 | floor 5216/5216 | ✅ `Summary [ 421.095s] 5219 tests run: 5219 passed (5 slow), 21 skipped`, zero FAIL rows — **5,216 + 3, reconciling the rider's finding 6** |
| 10 | clippy rc=0 | ✅ rc=0 |

**Row 2's mutation, re-driven here.** Appending field names to the remedy — so every *presence*
substring survives — reddens **exactly** the naming probe:

```
FAIL  the_misspelled_variant_refusal_names_the_enum_and_its_real_variants
PASS  a_misspelled_enum_variant_in_a_rete_constraint_is_refused          ← D1's own
```

The absence is gated independently of the presence, and the naming contract rides on the new probe
rather than on D1's.

**Row 4, answered.** `:tg::P::Hi` (arity 1) *resolves* through `enum_variant_ctor`, so it lands on
`Keyword` and keeps D1's route: still *"`:tg::Req` has no field `:tg::P::Hi`; available fields:
[k, grade]"*. **That is correct**, and the reason matters: the variant EXISTS, so "has no variant"
would be false. Its real mistake is a *bare tagged variant used as a value* — a third thing, whose
remedy is neither the fields nor the variants. DESIGN cut it; the rider **pinned it with a golden**
so the day someone takes that strike, the pin reddens and names what moved. ★'s literal *"never a
keyword"* is half-satisfied, deliberately and on the record.

## ⛔ Where MY brief was thin — and one would have shipped a false diagnostic

- **A. ★★ THE SKETCH CONTRADICTS THE STONE'S OWN CUT.** I wrote the new arm as a guard —
  `_ if prefix_of(k).is_some_and(…enum…)` — placed **after** the arity-0 arm. A guard inherits
  everything the arms above it did not consume, so it catches `Some((_, _, n>0))` — a tagged variant
  that **exists** — as well as `None`. Followed literally it emits *"`:tg::P` has no variant `Hi`;
  available variants: [Hi]"*: a message listing the variant it claims is missing. **That is the exact
  class this strike exists to delete, drawn into the cure.** The correct split keys on the
  resolver's own `None`, not on a predicate about the prefix that merely correlates with it.
  **Arc doctrine, fifth catch-all split and the first drawn wrong: split on the DISCRIMINATOR, not
  on a symptom.**
- **B. Row 5 names two routes; there are three.** A `::` name whose prefix is registered but as an
  **aggregate** is its own route — and it is precisely the one a `types.get(prefix).is_some()`
  widening (the most natural over-wide slip) swallows while both routes I named stay green. Driven
  by the rider at M3b, isolated to a single error.
- **C. My mutation 2 was the weaker mutation.** *Substituting* field names for variants trips the
  presence check too, so it cannot distinguish presence-only from presence+absence. Only *appending*
  leaves every presence substring intact. The rider ran the appending form; so did I.
- **D. Trap 5's prescription is not what the lint asks.** `no_loose_string_assert`'s rubric routes a
  deterministic **structured** value to a co-located `.edn` golden via `wat::assert_edn_eq!`, not to
  `assert_eq!`. And D1's `run()` helper passed an **absolute** path, putting a machine-dependent path
  into the refusal's `Span` and making any golden uncheckable — fixed to `current_dir(manifest)` +
  relative. Unanticipated by me, and without it rows 1–2 could only have been asserted loosely.
- **E. My breadcrumb named a kind that does not exist.** `UnknownEnumVariant` appeared in three
  stamps before this strike and `grep -rn 'UnknownEnumVariant' src/` returned nothing — I invented
  the name and then cited it. **Second time in four strikes** (D3's `callee_program`). Promoted to
  memory. It now exists, because the rider built it — which is luck, not vindication.
- **F. STOP-3 fired and the rider correctly did not stop.** The tagged path does not reach the new
  arm; halting there would have left the primary work undone while row 4 asked for exactly that
  answer. A STOP trigger is a rejection criterion for *proceeding blind*, not for *finding out*.

## Arms not driven, named

`segment()`'s `UnknownVariant` half — **not reachable**: both callers intercept `UnknownVariant`
before `segment()` is consulted. Kept rather than `unreachable!()`, so no panic path is introduced;
named in its doc.

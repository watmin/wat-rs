# SCORE — the nested-constructor wall, weighed against the orchestrator's own re-run

> Re-run here at `c0c883082`.

## The scorecard, re-run

| # | pre-value | after |
|---|---|---|
| 1 | **`"ACCEPTED-UNVALIDATED"`** | ✅ refused — *"`:fsn::Inner` has no field `:nope`; available fields: [x]"*, driven by me |
| 2 | `grep -c` → **0** | ✅ **6** |
| 3 | `RhsMissingFields` never fired | ✅ driven |
| 4 | `RhsArityMismatch` never fired here | ✅ driven, and separated from the enum-variant producer of the same kind by mutation D |
| 5 | `RhsPositionalConstructionRetired` never fired | ✅ driven — **and see the finding below; this is new enforcement** |
| 6 | un-lowered branch unknown | ✅ driven via `unreachable!` over 567 tests: **silent**. Kept, not deleted |
| 7 | `aggregate-new` | ✅ **arm omitted — and my premise for omitting it was wrong**; see thin spot A |
| 8 | the pin | ✅ re-pointed, guard replaced honestly (thin spot D) |
| 9 | radius | ✅ **one hunk in one function** + probes |
| 10 | lint 116/116 | ✅ 116/116 |
| 11 | floor 5225/5225 | ✅ `Summary [ 422.470s] 5230 tests run: 5230 passed (4 slow), 21 skipped`, zero FAIL rows |
| 12 | clippy rc=0 | ✅ rc=0 |

**Mutation A, re-driven here** — revert the head recognition:

```
FAIL  a_nested_constructor_naming_an_undeclared_field_is_refused
FAIL  a_nested_constructor_under_supplying_a_declared_field_is_refused
FAIL  a_nested_multi_arg_positional_construction_is_refused
FAIL  a_nested_single_positional_arg_against_a_wider_record_is_refused
FAIL  a_nested_constructor_names_the_field_keyword_not_the_whole_form   ← the re-pointed pin
PASS  a_correctly_spelled_nested_constructor_still_compiles_and_fires   ← the control
PASS  every_shape_spelled_correctly_compiles_and_fires
```

Exactly five, the orphaning reproduced, controls green — so it is not a blanket refusal.

## ⛔⛔ THE STRIKE SHIPS NEW ENFORCEMENT, AND ONLY A DRIVE FOUND THAT

`RhsPositionalConstructionRetired`'s doc claimed the runtime dispatch *"unconditionally retires
multi-arg RAW POSITIONAL construction"*. **Driven at HEAD: a nested `(:T ?k 99)` compiled, fired,
and derived `y = 99`.** Both citations verified by me:

- `arm.rs`'s `rhs_must_compile` — *"Refuse — **do not walk `build_insert_fact` on native fire**."*
- `eval_insert.rs:45` — the non-kwargs arm returns the args verbatim: *"Positional is already
  declaration order BY DEFINITION."*

So the retirement was **never enforced on the rete path**; the doc was written from the
**interpreter's** behaviour and never checked against this one. Wiring the kind is therefore **not**
"making an unreachable refusal reachable" — it is this wall becoming the only enforcement of that
doctrine on this path. **Accepted deliberately**, on a corpus sweep of 1650 `.wat` / 460 `:then`
clauses showing zero uses, and **the false doc is corrected at the site** with the drive and the
decision. Promoted to memory: *reviving a dead guard is a behaviour change — drive the live path
first.*

## ⛔ Where MY brief was thin

- **A. ★ My "out of scope" premise was FALSE.** DESIGN said the driven evidence shows all spellings
  arrive as `kwargs-construct`. A hand-written `(:wat::core::aggregate-new :T ?k)` in a `:then`
  **does** arrive, head intact, type resolving. I generalised from four *source* spellings to all
  spellings. **The rider omitted the arm anyway, for a better reason than mine:** `aggregate-new`
  **is** the positional route, so firing `RhsPositionalConstructionRetired` under it would be an
  actively **wrong refusal**, not merely a dead arm. Right answer, wrong reason in my stone.
- **B. My sketch's `_ => return` would have been a regression.** Where `items[0]` is not a keyword,
  the current code still recurses into every item; returning would drop nested constructors under a
  call form. The rider computed a `type_idx` and left the fall-through intact.
- **C. My sketch indexes `items[1]` unguarded.** `(:wat::core::kwargs-construct x 1)` over a
  non-keyword is expressible — `purity.rs:829` has a gate for exactly that. STOP-3's shape, and
  reachable; the rider falls through to generic recursion rather than widening blind.
- **D. Row 8's "keep the anti-vacuity guard" was unwritable as stated.** The guard asserted
  `stdout == "ACCEPTED-UNVALIDATED"` to prove the program reached `main`; after the fix `main`
  cannot run. Replaced with an exact-`Span` golden plus the control, and the sentinel **kept in the
  fixture as a tripwire** — if a future lowering orphans the wall again, the failure message shows
  the exact string that named the original hole. Better than what I asked for.

## Reported, not fixed — per the cut, and worth rows

- **`RhsMissingFields` and `RhsArityMismatch` render the NESTED operand as the inserted fact**:
  ``:then` insert of `:nwm::Inner`` when the `:then` inserts `:nwm::Outer` and `Inner` is nested
  inside it. Their messages were written for the top-level producer and are reused verbatim.
- **A third head shape nobody anticipated:** the positional prime `:T'` reaches the wall
  **un-lowered**, and `types.get` fails on the suffix — still silently unvalidated. Zero corpus
  uses; not widened for.

## Arms not driven, named

None among the four kinds. The un-lowered `type_idx == 0` aggregate branch is **not reached** across
`wat::rete` + `wat::lint` (567 tests) — driven by `unreachable!`, kept rather than deleted, since
that is two binaries and not the full floor.

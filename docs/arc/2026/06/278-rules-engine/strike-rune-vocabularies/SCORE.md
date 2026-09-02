# SCORE — F1 row 3, weighed against the orchestrator's own re-run

> Re-run here at `9d4b68088`. **This is the strike where the rider refuted both of my premises, and
> was right on both.**

| # | pre-value | after |
|---|---|---|
| 1 | `perspicere`/`purgare` 0 mentions in CONVENTIONS.md | ✅ both defined, in `sequi`'s shape — **copied from the ward spells, not invented** |
| 2 | ★ definition precedes the gate | ✅ — and from a **better** source than I specified; see A |
| 3 | the census.rs six | ⛔ **REFUTED — they are correctly labelled**; see B |
| 4 | the gate | ✅ `no_unknown_ward_rune.rs`, 7 arms all mutation-proven |
| 5 | STOP-3 | ✅ **CONFIRMED, and sharper than the row** |
| 6 | non-vacuity | ✅ measured floors (2,589 files / 46 / 11), driven |
| 7 | radius | ✅ `CONVENTIONS.md` + one gate. **Zero `src/` edits** — correctly, given B |
| 8 | lint 173/173 | ✅ **182/182** |
| 9 | floor 5287/5287 | ✅ `Summary [ 428.168s] 5296 tests run: 5296 passed (6 slow), 21 skipped`, zero FAIL |
| 10 | clippy rc=0 | ✅ rc=0 |

## ⛔⛔ A — MY PREMISE WAS FALSE, AND I VERIFIED THE REFUTATION MYSELF

DESIGN: *"Nothing says what any of them mean."* I fetched `perspicere` from the signed channel and
its § **The rune** defines all three categories verbatim — `read-once` is *"deep type appears exactly
once and a name would be read-once-then-forgotten"*; `mumble-alias` is *"the typealias would itself
be a **Level 2 mumble**"*; `intentional-structure` likewise.

**I grepped `CONVENTIONS.md`, got 0, and concluded "undefined."** That proved the *copy* was missing
— nothing about the thing. The authority is the ward spell, from the same signed MCP I had used at
the top of this very session.

It also changes the method for the better, and the rider saw that: my ⚠ demanded the set not be
derived from use, and the spells are **an independent referee predating every site**. A definition
I invented locally would have been derived from the sites it was meant to judge. Promoted to memory.

## ⛔⛔ B — MY LIVE FINDING DOES NOT SURVIVE CONTACT, AND ACTING ON IT WOULD HAVE MADE THINGS WORSE

I reported six `census.rs` runes whose reasons *"all end 'alias would be a mumble'"* and called them
mislabelled. Measured here: **that clause appears verbatim at 18 sites across four files** —
`binding_repr_bench.rs` ×9, `census.rs` ×6, `purity.rs` ×2, `accum_alpha_cost.rs` ×1. **I saw a
quarter of the population and read shared boilerplate as a per-site argument.**

Against the authored definition the six are **correctly labelled**: `mumble-alias` requires the
alias to be a *Level 2 mumble*, and `CensusLog` is a specific domain noun that reads **better** than
`RefCell<Option<Vec<RoundCensus>>>`. Each reason's *first* clause is the locality argument;
`read-once` fits.

**Following my instruction literally would have moved 6 of 18 identical sites** — manufacturing
exactly the `ARM_TABLE`/`EXEC_ARENA` divergence `sequi` exists to prevent. The rider re-categorised
nothing and made **zero `src/` edits**, which trap 2 required and which I would have overridden.

The real defect is what the ward's own reporting format already names: a reason whose trailing clause
is *"vague, **copy-paste**, or reads like 'I didn't want to alias this'"*. Recorded in the table and
the gate header rather than acted on.

## C — my site counts were off by more than 2×

I said 18 `perspicere` / 9 `purgare` = 27. Measured: **46 `perspicere` tree-wide**, 11 `purgare`,
**57 sites**. The `purgare` figure matches only a `src/`-only scan; the `perspicere` one matches
nothing reproducible.

## STOP-3 — confirmed, and it found more than the row claimed

No trait exists at any `trait-contract` site (`clause.rs` has **zero** `impl`/`trait` lines — I
checked). Sharper: the two sites that genuinely **are** trait impls (`impl Debug for Receiver<T>`
and its `Select` twin) are labelled **`public-api`**. The two categories look **swapped**. And no
category covers *"retained for structural completeness"* — a genuine gap in the **ward's** vocabulary,
now named in the table rather than silently patched. The rider stopped and re-categorised nothing,
correctly.

## What the gate does NOT do — driven by me, not asserted

Swapping `census.rs:86` from `read-once` to `mumble-alias` leaves the gate **green** (9/9).
**Spelling is machine-checkable; fit is not.** Only the CONVENTIONS tables judge fit, each carrying a
discriminating question. That limit is in the gate's header, where a passing floor cannot imply more
than it proved.

## One deviation, and it is an improvement

EXPECTATIONS predicted the empty-vocabulary mutation would red the *non-vacuity floor*. It reds the
**violations** assert instead, naming all 46 sites — because `seen` counts markers independently of
the set's contents. That is the stronger failure (it says which sites lost their vocabulary), and the
floor arm is separately proven by blinding the extractor.

## Arms not driven, named

The `read_to_string` `Err` branch — a `continue`, not a verdict; **not reachable** without an
unreadable file, and it renders no judgement either way.

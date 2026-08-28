# Proposal — `experiri`, the ward the vigilia does not have

> *experīrī* — Latin, deponent: to put to the proof, to try, to learn **by trial** rather than by
> report. Root of *experiment* and *experience*. Not "to inspect a thing" — to **undergo** it.

**Status:** proposal, written 2026-08-28 from arc 278. Not published; the grimoire is a signed
channel and this is the builder's to mint. The evidence below is what earned it.

## The gap, stated as a fact about the set

Twenty-two inward wards, and **every one of them READS.** They hold a source against a spec, a
name against its referent, a comment against its code, a document against itself, and report where
the two disagree. `circumspicere` is the outward one — it looks around at what the inward lenses
turned their backs on — and it reads too.

**Nothing in the vigilia RUNS the thing it audits.**

## What that blindness cost, measured

The rete vigilia **converged**: recasts 12 and 13 both `0 L1 + 0 L2`, inward 17/17 plus
`circumspicere`, at HEAD `8839bb16`; `REALIZATIONS.md` R68 records the empty recasts as the event.
PILE 2, the ward tail, was later audited row-by-row and came back **4-for-4 stale** — every
finding already closed. By every instrument the watch has, the surface was clean.

Then a ledger that DRIVES each row found, in one day:

| what | how many |
|---|---:|
| rows that pass admission, totality, arity and type and **cannot execute at all** | **6** |
| rows accepted as an inline constraint that **compile, fire, and match nothing** | **39** |
| a first-class type (`Tuple`) that could be **constructed and never read**, since genesis | 1 |
| a second implementation of `first` disagreeing with core about which containers exist | 1 |

## Why no reading ward could have caught it

This is the load-bearing argument, and it is not "they were not thorough enough."

A reading ward finds a **disagreement**. Here there was none. `:wat::rete::core::Tuple` was
declared in `RETE_OPS`; the fence admitted it; the checker typed it; `purity.rs` approved it; the
naming rule derived it; `every_rete_row_is_total` passed it. Source and spec were **consistent** —
and both were wrong together, because the spec advertised a surface the executor had no arm for.

> **A reading cannot see an execution defect.** It is FM 28 and FM 30 one level up: a count cannot
> see a value defect, a list is only a claim, and a declaration is only a promise.

And a corpus cannot stand in for the run. `vocabulary.rs` had already written the sentence that
proves it: *"a corpus records what COMPILED, so it is structurally blind to what cannot be
written."* Three of the six broken rows appear nowhere in the 1569-file corpus — which read as
neglect and was actually the symptom.

## What KIND of spell it is — observation vs experiment, not work vs reasoning

It is tempting to say `experiri` is "the one that does the work". That is wrong twice over: the
reading wards do plenty of work, and this one does plenty of reasoning — the position set, the
calibration, the finding-vs-driver adjudication are all judgment before a single program runs.

The true line is **what counts as evidence at the moment of the finding.**

> Every other ward's evidence is the **SOURCE**. `experiri`'s evidence is an **EVENT**.
>
> The others ask *what does this text say?* — and are right to, because most of the book's
> questions have no runtime at all. There is no program you can run to learn whether a name lies
> (`intueri`), whether a document coheres (`cohaerere`), whether prose rings true (`consonare`), or
> whether a suppression still earns its standing (`excusare`). Those are observational questions
> and reading is the correct instrument.
>
> `experiri` asks *what does this system DO?* — the one question no amount of reading answers.

Two consequences follow, and both are arguments for keeping it rare rather than for promoting it:

- **It is expensive.** The rete precedent is 77 rows × 2 positions = 154 full program loads, ~30s
  serially — past the runner's deliberate 30s kill, and sharded six ways to fit. Reading is cheap
  and runs on any file; that is why the book is mostly readers, and that is the right ratio.
- **It can lie in a way a reading ward cannot.** A broken reader usually finds NOTHING — a
  filter that matches no pattern reports an empty list, and empty reads as clean. A broken DRIVER
  finds a JACKPOT: one mis-rendered position reports a whole column of refusals that look exactly
  like a discovery, and its findings are meant to be believed. That asymmetry is why the
  calibration, the finding-vs-defect split, and the mutation proof are not ceremony here — they
  are the price of admission for an instrument whose failure mode is a false triumph.

**A catalog line, for whoever mints it:**

> **`experiri`** — Put the declared surface to the proof. The datamancer experītur — every form the
> system ADVERTISES is synthesized, driven, and made to answer, in every position it claims to be
> usable; a surface that cannot be reached, or is reached and does nothing, is a promise the system
> does not keep. The only ward whose evidence is an event rather than a source — cast it when the
> spec and the code AGREE and you still do not know whether either is true.

## What the ward does

> The datamancer **experītur** the declared surface — every form the system ADVERTISES is
> synthesized, driven, and made to yield a verdict, in **every position it claims to be usable**.
> A surface that cannot be reached, or is reached and does nothing, is a promise the system does
> not keep. Presence in a table is not aliveness; only the run is.

Three properties distinguish it from everything already in the book:

1. **It EXECUTES.** The only ward that must. Its evidence is a program that ran, never a file that
   parsed.
2. **It SYNTHESIZES its own callers.** It may not sample the corpus, because the corpus is the
   blind spot — it holds only what already worked.
3. **Its unit is (declaration × position),** not the declaration. Reachability is not a property of
   the thing declared: `keyword::=` is reachable inside a `where` fence and refused as an inline
   constraint — same op, same field, same comparison, two answers. A ward that asked once per row
   would have to pick one, and either choice is a lie about half the surface.

## Its own failure modes, named up front

An executing ward can lie in ways a reading one cannot, and the harness has to answer for them:

- **A broken driver reports a column of false findings** that read exactly like a discovery. Cure:
  a CALIBRATION of known-answer cells, with **two of each verdict** — a driver that renders nothing
  passes an all-refusal control, one that never applies its constraint passes an all-fire control.
- **"It refused" is two different facts.** A refusal that names the thing under test is an answer;
  one that does not is a bug in the driver. They must be separate outcomes, and the second must be
  loud rather than counted.
- **"It ran" is not "it worked."** A form reached that then discriminates nothing is not
  reachability. Every cell must be made to CHANGE ITS ANSWER, or it proves only that nothing threw.

## What it is NOT

- **Not `cernere`** — its exact mirror. `cernere` catches a CALLER using a form the spec never
  defined (phantom form). `experiri` catches a SPEC declaring a form no caller can reach (phantom
  surface). Opposite directions, and only one of them can be answered by reading.
- **Not `conferre`** — that finds spec and implementation DISAGREEING. `experiri` exists for when
  they agree and are jointly wrong.
- **Not `probare` or `complectens`** — those judge whether tests are substantive and well-layered.
  `experiri` audits the SHIPPED surface, not the test suite.
- **Not a test suite.** It is a census with a verdict per cell, and its output is a matrix a reader
  can disagree with — not a pass/fail.

## ⛔ Would it have caught everything the others missed? NO — and the honest tally is the point

A proposal that claims total coverage is the decoration this arc keeps removing. Every finding of
2026-08-27/28 tested against the ward as specified above:

| finding | would `experiri` catch it? |
|---|---|
| 6 rows passing every static gate that cannot execute | **YES** — this is precisely its class |
| 39 rows accepted inline that compile, fire, and match nothing | **YES** — its unit is (declaration × POSITION), and a cell must change its answer |
| `:wat::core::Tuple/length` is an unknown function (a surface I inferred from error-message string literals) | **YES** — drive it, and the phantom names itself |
| the `Tuple` row unobservable | **YES, but only the SYMPTOM.** It reports "cannot be driven to a verdict". Whether the cure is *delete the row* or *add the accessors* is a judgment it does not make — and I drew the wrong one |
| `filterv` shipped with no `infer_rete_form` route | **ONLY WITH THE RIGHT POSITION SET.** It fires in a fence and would refuse in ordinary wat. Caught if and only if "written in ordinary wat" is one of the positions driven |
| `first_of` a second implementation of `first`, disagreeing with core about which containers exist | **NO.** No cell would exercise it, because the row that needed it did not exist yet. This is `solvere`'s class — duplicated encoding — and `solvere` ran and closed **all 7 L2** without it |
| `reduce`'s 2-arity raise against its `total: true` row | **NO.** Needs edge-input driving, not a discriminating pair. This is `conferre`'s class — spec against implementation — and `conferre` ran and closed its L2 without it |

**Three lessons, and they matter more than the yes-column.**

1. **The POSITION SET is the ward's coverage, and choosing it is the whole difficulty.** The ledger
   modelled two positions and missed a third the same row was reachable from. An `experiri` that
   drives one surface and calls the row audited is the same false-completeness it exists to attack.
2. **It finds symptoms; it does not diagnose.** "This cannot be driven" is a fact. "Therefore the
   row should not exist" was my inference from it, and it was WRONG — the builder refused it, and
   measuring proved core had served tuples correctly all along.
3. **Two of the day's findings came from neither the wards nor the ledger — they came from the
   BUILDER.** The corpus fallacy on `Tuple` ("never tell me we don't need something because a user
   hasn't used it") and "foldl is reduce" were both human corrections of a confident instrument.
   No spell in the book substitutes for that, and one claiming to would be lying.

## Precedent, already built

`src/rete/reachability.rs` is this ward performed by hand against one table: 77 rows × 2 positions,
calibrated, adversarially separated into finding-vs-driver-defect, mutation-proven in both
directions by disjoint tests. It found all six unrunnable rows, and every one is now fixed.

The generalisation is not rete-shaped. Any advertised surface has this failure available to it: a
config key nothing reads, a CLI flag no path honours, an error variant nothing can raise, a trait
impl no caller can select, a feature flag with no branch. Each is declared, consistent, and inert —
and each is invisible to twenty-two wards that read.

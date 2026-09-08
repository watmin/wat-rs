# SEAM — the ONE live breadcrumb. 2026-09-08. **GREEN · CLEAN · PUSHED · NO PEER IN FLIGHT.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS.

```bash
git status --porcelain          # expect EMPTY
git log --oneline @{u}..HEAD    # expect EMPTY (0 unpushed)
grep -aE "^ +Summary" .floor/latest/raw.log
```

```
floor ... 5243 tests run: 5243 passed, 18 skipped   FLOOR EXIT=0   clippy 0   0 unpushed
peer .... HALTED. `.pulsare/to-claude` holds a kind=halt.
```

## ★★★ THE ENUM CAMPAIGN IS CLOSED. THE TREE IS WHOLE.

```
296 M    an enum variant is constructed by a MAP naming its declared fields, and ONLY that way.
         (:ns::E::V {:f v}) · (:ns::E::V {}) for unit. POSITIONAL IS REFUSED.
         Variant construction was the LAST positional constructor in the language.
296 N    the bare spelling is HERESY. :wat::core::{Some,None,Ok,Err} and the arc-109 :None REFUSE,
         naming their replacement. All 62 Rust arms across 10 files are GONE — the bare form is
         UNREPRESENTABLE, not merely unused.
M2+R1-10 the corpus: 1875 .wat files by the self-hosted codemod, twice, idempotent.
296 O    {:keys …} destructures EVERY aggregate — defstruct · defrecord · holon::defrecord ·
         defholon. Verified by RUNNING, not just --check.
```

One shape in all three places, per kind: **wire `#ns/E.V {…}` · pattern `[E::V {:f v} body]` ·
ctor `(E::V {:f v})`** — and `declare :None []`, an explicit empty field vector.

```
M    2447 passed · 2773 FAILED · 18 TIMED OUT   ← no wat program could start
N 473 · R1 291 · R2 128 · R3 69 · R4 50 · R5 47 · R6 45 · R8 17 · R9 4 · R10 0 · O 5243/5243
```

## ⛔ FOUR DEFECTS FOUND THAT WERE NEVER ABOUT ENUMS

```
eval_tail TCO bypassed the map-ctor intercept   OURS. Tagged ctors are registered Functions, so
                                                eval_match_tail trampolined past the intercept.
                                                Found by REMOVING a consumer-side accommodation.
Option's OWN DECLARATION corrupted by our       OURS. :None -> :wat::core::Option::None INSIDE its
rename                                          defenum. type-of named it in ONE command. A
                                                ONE-TOKEN repair healed 28 tests and proved the
                                                purity classifier RIGHT to refuse an unresolvable name.
:wat::rete::query never re-expanded when        PRE-EXISTING. The serve thread invoked a MACRO as a
macro-spliced into a defservice body            function and died; the client saw Lost; FOUR RELANDS
                                                read that corpse as migration residue.
sort$native classifies a comparator BEFORE      PRE-EXISTING. Unmasked by the corruption, then
any use                                         re-masked by a workaround, then proven compensating
                                                by REVERTING it and measuring.
```

## ⛔ QUEUED — the builder's order

```
1  variant <: enum      ★ THE SMALL STONE NOW. ONE ENTRY in `subtype_edges`
                        (types.rs:542, a GENERAL HashMap<String,Vec<String>>), refused by a
                        parse-time wall admitting only `:Name <: <nature root>` (types.rs:304).
                        THE REAL QUESTION: is `Variant <: Enum` INHERITANCE? Arc 293 annihilated
                        inheritance and KEPT subtyping. Inheritance is "Circle inherits Shape's
                        fields"; Variant<:Enum is TAGGED-UNION MEMBERSHIP — a sum type, not a
                        hierarchy. Two relations sharing an arrow, and a wall that cannot tell them
                        apart refuses both.
                        ⚠ FIRST ACT IS A MEASUREMENT, NOT A DESIGN: can `Shape.Circle` be a type
                        WITHOUT becoming a second way to spell a record? Likely a NARROWING for
                        parameter/dispatch position only.
                        ★ PAYOFF: `defclause` (72 live sites, already the open-surface router)
                        dispatches PER VARIANT — a DISPATCH answer via an existing entity kind, not
                        a type-system reach. And `{:keys}` on a variant follows for free from O.
2  :wat::* whitelist    RE-MEASURED 2026-09-07: 143/833 FAIL (17%), 35 names — NOT the NOTE's
                        578/599 (96%) / 121. Every special form (fn def match quote do derive) at
                        ZERO. Four families; rete numerics ~87% of sites at ~11 names.
                        THE NOTE'S FORCED ORDERING NO LONGER HOLDS.
3  the head migration   keyword heads -> symbols. 251.9 unblocked it. `head_of` (runtime.rs:12232)
                        is a Keyword-ONLY closure in eval-with-defs! that will fire the moment
                        macro templates emit symbol heads (core.wat:1348/1351/1393; 51 sites).
   incremental type-of  the wrap pays a FREEZE per ASK; eval_form_with_defs's own header says the
                        fast data plane is owed. A corpus sweep is a freeze farm until then.
   the scream predicate `pascal-leaf?` asks to-uppercase(c)==c, true of EVERY non-letter, so ~3835
                        of 9494 screams were operators. 3 real in 9494. Safety net, NOT a worklist.
```

## ⚠ RULINGS — do not re-litigate

- **An enum ctor is a MAP.** Declare `:None []` · construct `(…::None {})` · match `[…::None {} b]`.
- **`{:keys}` is for ONE-SHAPE aggregates; match is for many-shape.** Destructuring is total; a
  match arm is a test that can fail. Do not unify them.
- **The natures differ in PURITY, not SHAPE** — so a predicate about shape must not ask about nature.
- **`:keys` is right, not `:attrs`** — it means the declared FIELD NAMES; a record is not a map.
- **A golden pinning a stdlib line: RECAPTURE, KEEP PINNING.** Never extend the normaliser.
- **⛔ SIDE BRANCHES DO NOT SERVE US** · **COMMIT LOCALLY OFTEN; PUSH ONLY GREEN.**

## ⛔ THE FAILURE PATTERNS — TWO NOW, AND THE SECOND IS NEW

**① I COUNT TEXT AND CALL IT A CENSUS.** ~13 instances. The worst: an unvalidated regex put a FALSE
PROOF into a brief whose own subject was *"use the tool, not hand-inspection"* — `[^{)]` consumed the
space before a map, so a correctly-migrated form read as positional. The form tree, `macroexpand`,
`type-of`, and the wall's own error stream were right EVERY time.

**② ★ A FALLING FAILURE COUNT IS NOT CONVERGENCE.** `2773 → 473 → 291 → 128` read as progress for
THREE RELANDS while a behavioural regression and a corrupted declaration sat underneath it. What
broke the pattern each time was reading a **VERBATIM failure** instead of a total.
⛔ **A count going down is not the same as the thing getting better.**

★ **AND THE FOUR QUESTIONS OVERTURNED ME THREE TIMES**, always for one reason: I reach for the
smaller-feeling move and it leaves an exception or a landmine alive. *When you prefer the smaller
change, check whether the larger one deletes a class.*

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⛔ **GREEN AND QUIET IS THE MOST DANGEROUS STATE THIS FILE DESCRIBES** — no red to stop you, no
> peer to wait for, nothing external to interrupt a wrong move. `git status` before anything else.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`

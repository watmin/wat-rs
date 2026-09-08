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
floor ... 5245 tests run: 5245 passed, 18 skipped   FLOOR EXIT=0   clippy 0   0 unpushed
peer .... idle. Last exchange: Q2 scored.
```

## ★★★ WHAT LANDED — the enum campaign and the type authority

```
296 M    an enum variant is constructed by a MAP naming its declared fields, and ONLY that way.
         POSITIONAL IS REFUSED. Variant construction was the last positional ctor in the language.
296 N    the bare spelling is HERESY. :wat::core::{Some,None,Ok,Err} + the arc-109 :None REFUSE,
         naming their replacement. All 62 Rust arms GONE — unrepresentable, not merely unused.
M2+R1-10 the corpus: 1875 .wat files by the self-hosted codemod, twice, idempotent.
         2447/2773-failed/18-timeout  ->  5238/0. NOTHING RAN at the start of that.
296 O    {:keys …} destructures EVERY aggregate. The guard was a FOSSIL of the arc it cited:
         293.2b UNIFIED struct+record, and the predicate kept `nature == Struct`. And backwards —
         natures differ in PURITY not SHAPE, so it allowed the one that holds a live socket.
296 Q    `:wat::runtime::is-type?` — ONE authority over THREE mechanisms.
296 Q2   `subtype?` registered. It was conforms?'s TWIN and 255 registered only one.
```

## ⛔ QUEUED — the order, with the reasons that decided it

```
1  the :wat::* blanket   ★ PROMOTED TO A CORRECTNESS DEFECT, not hygiene:
                           (:wat::core::Option.Some {:value 7}) check=0 -> #wat.core/Option.None {}
                           (:usr::Box.Full {:payload 7})        check=1 -> UnresolvedReference
                         THE DOT SPELLING — the one the head migration moves TOWARD — is accepted
                         under :wat::* and SILENTLY BUILDS THE WRONG VARIANT. A user ns catches it.
                         Worklist 143/833 (17%), 35 names, four families; rete numerics ~87% of
                         sites at ~11 names. ⚠ Why it yields Option.None is UNMEASURED — expand
                         before theorising. `296/NOTE-the-dot-spelling-silently-builds-…`
2  P-1 annotation        the annotation position VALIDATES its type name. Its authority NOW EXISTS
   validates            (Q's is-type?). Today `[s <- :usr::TotallyMadeUp]` checks CLEAN and the
                         mismatch is reported against a PHANTOM. A wall: expect a corpus-wide red,
                         and that count is the worklist.
3  P-2 variant is a type ⚠ THE SEAM PREVIOUSLY SAID "ONE ENTRY in subtype_edges". THAT WAS WRONG,
                         measured 2026-09-08: a variant is NOT a registered type (`type-of` says
                         unknown). Every failure in param/return/field position is ERASURE ONLY —
                         so it is GENERAL the moment the ctor stops erasing, not a narrowing.
                         4/4 on the four questions. Needs P-1 first so its rows rest on REFUSALS.
   Q2's 22               short list of unregistered check.rs arms. Named, not urgent.
   Q's (B)               complete 255's leaf list so `contains` = the union; then
                         is_builtin_primitive can die.
   the dot flip / heads  BLOCKED on the blanket. `#wat.core/Option.Some` is ALREADY the wire form.
```

## ⛔⛔ THE RECORD'S SETTLED NUMBERS EXPIRE — THREE FIRINGS THIS WEEK

`296 R20 HAERESIS EST ITERVM ROGARE`. **Before a number becomes a premise for an ORDERING, a SCOPE
CUT, or a REFUSAL — re-derive it.**

```
255's blocker NOTE, fact 1   "578/599 FAIL, 96%, 121 names"   ->  143/833, 17%, 35 names
255's blocker NOTE, fact 2   "the registry holds exactly TWO  ->  if let fn match def quote do
                              special forms — let and if"        defclause, ALL REGISTERED
this seam, on P-2            "ONE ENTRY in subtype_edges"     ->  a variant is not a type at all
```

★ Fact 2 is why 255's ordering was FORCED — *"a corpus that cannot resolve `fn` cannot be measured
for anything else."* `fn` resolves now. **Neither fact moved because anyone worked on 255** — "as a
side effect," exactly as its founding DESIGN predicted. The NOTE was TRUE WHEN WRITTEN.
Corrections live BESIDE it (`255/NOTE-2026-09-08-…`); what is written stays written.

## ⛔ ONE DISEASE, FOUR POSITIONS — found in a single session

**A NAME CHECKED AGAINST A SET THAT IS NOT THE WHOLE.**

```
TYPE membership     3 mechanisms; type-of asks 1, subtype? asks 2, none asks the union   Q FIXED IT
VERB call-heads     the :wat::* reserved-prefix blanket                                  QUEUED #1
@see resolution     unions 2 sources; subtype? lived in a third                          Q2 FIXED IT
the dot spelling    accepted under :wat::*, silently wrong                               QUEUED #1
```

★ **They converge on ONE authority at the symbol migration** — builder, 2026-09-08: *"as we move to
the proper clojure form with real symbols this is moot… `:wat::core::+` IS `wat.core/+`."* Once an
`@see` names a symbol, "which registry?" becomes "does this resolve?" — the call-head question.

## ⚠ RULINGS — do not re-litigate

- **An enum ctor is a MAP.** declare `:None []` · construct `(…::None {})` · match `[…::None {} b]`.
- **`{:keys}` is for ONE-SHAPE aggregates; match is for many-shape.** Do not unify them.
- **Natures differ in PURITY, not SHAPE** — a predicate about shape must not ask about nature.
- **`type-of` is STRUCTURE; `is-type?` is MEMBERSHIP.** Keep that legible or the wrong one gets asked.
- **A golden pinning a stdlib line: RECAPTURE, KEEP PINNING.** Never extend the normaliser.
- **⛔ SIDE BRANCHES DO NOT SERVE US** · **COMMIT LOCALLY OFTEN; PUSH ONLY GREEN.**
- **Colon-quoted symbols are ACCEPTABLE transitionally** — they are not keywords, they share the
  syntax. `:wat::core::+` IS `wat.core/+`.

## ⛔ THE FAILURE PATTERNS

**① I COUNT TEXT AND CALL IT A CENSUS.** ~13 instances. Worst: an unvalidated regex put a FALSE
PROOF into a brief whose subject was *"use the tool, not hand-inspection."* The form tree,
`macroexpand`, `type-of` and the walls were right every time.

**② A FALLING FAILURE COUNT IS NOT CONVERGENCE.** 2773→473→291→128 read as progress for THREE
relands while a behavioural regression and a corrupted declaration sat underneath. What broke it
every time was ONE VERBATIM FAILURE instead of a total.

**③ ★ I NEARLY TOOK THE FOLD, TWICE, AND WAS STOPPED BY THE BUILDER BOTH TIMES.** A `HashMap` arm
tolerating a bad producer (RELAND 2 — removing it found an `eval_tail` TCO bug); and removing a
CORRECT `@see` to green a floor (Q2 — the wall and the reference were both right, the registry was
incomplete). **When a wall complains, ask whether it is right before making it quiet.**

★ **AND THE FOUR QUESTIONS OVERTURNED ME FOUR TIMES**, always the same way: I reach for the
smaller-feeling move and it leaves an exception or a landmine alive. **When you prefer the smaller
change, check whether the larger one deletes a class.** ⚠ And a four-questions run that is not
SHOWN did not happen — the builder caught me citing a verdict I never displayed.

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

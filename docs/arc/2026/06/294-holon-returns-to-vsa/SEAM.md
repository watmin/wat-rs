# SEAM — the ONE live breadcrumb. As of 2026-09-02. **The campaign is arc 255: THE REGISTRY BECOMES THE SOLE AUTHORITY.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `251/SEAM.md`, `278/SEAM.md` are PARKED. Arc 109's megafile campaign reached its floor.
> ⛔ **PARKED IS NOT DEAD.**

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor ............ 5127/5127, 0 FAIL, 17 skipped, ~118s   (scripts/floor.sh, exit read UNPIPED)
clippy ........... 0 under `-D warnings --all-targets`
registry rows .... 515   ⛔ the 490 this line used to carry was WRONG, and so was every
                         earlier reading of it: the census grepped for the SUBSTRING
                         `wat_intrinsic("` and counted three PROSE PLACEHOLDERS as names —
                         `<fqdn>`, `…`, and `:wat::holon::…` (an ellipsis that survives any
                         "starts with `:`" filter). Anchor to the attribute SITE:
                         `grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/`
runtime.rs ....... 19,261   check.rs ....... 22,604   special_forms.rs ....... 379
host ............. JohnDesktop · john · ~/work/holon/wat-rs
```

## ⬜ THE CAMPAIGN — read these three, in this order, before proposing anything

```
RULING-the-registry-is-the-sole-authority.md                  the doctrine + the census
DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority.md    4 shapes, A picked, + the SEQUENCING RULING
WORKLIST-the-121-the-registry-cannot-vouch-for.md             re-derived 2026-09-02 → 107, with the road map
```
(all in `docs/arc/2026/06/255-builtin-registry/`)

**The builder's words, which are the standard:** *"the registry must be the thing who knows all
names.. who delegates to the code who performs for those names... **we must eliminate every source of
duplication or inconsistency**."* And the sequencing: *"we continue to add names to the registry....
then we attack the hand lists."*

## ★★★ THE METER — and the finish line

```
GAP_A 60 · GAP_B 78 · DEBT 95 · TYPES_UNCHECKED 10 · KNOWN_UNREVIEWED 20
the whitelist experiment: 107 at last measurement — ⚠ NOT re-derived since 1b-i.
28 of the 107 were registered by that stone, so the next run should read ~79 —
that is a PREDICTION. Re-run the WORKLIST's four steps before quoting a number.
```

⚠ **DEBT GOING UP IS NOT A REGRESSION.** Registering a row with no `CheckEnv` scheme converts an
*invisible* absence into a *named* one. That is an absence ledger's whole job. DEBT falls at Phase 2c.

## ✅ WHAT THIS SESSION SHIPPED — 96 commits, every one green at push

```
17 → 29 of special_forms.rs's 35 rows registered.  SIX REMAIN:
     ann-form · do · stream::lazy        ← ordinary, 1a-ζ
     defstruct                           ← a stdlib MACRO; unregisterable (see the fourth registry)
     unquote · unquote-splicing          ← punctuation; a CONTAINMENT fact, not a row (open)

⛔ freeze::is_liftable_declaration_head — KILLED, with its meter. THE FIRST HAND-LIST TO DIE.
   Four remain in that family: is_mutation_head · is_mutation_form ·
   DECLARATION_HEADS · RUNTIME_DECLARATION_HEADS

NEW MACHINERY, each sabotage-proven:
   SpecialFormRole::Declare    the third regime — freeze-time processing
   Purity::Unevaluated         a form that never evaluates; the gate keys on it, not @Category
   Category::Splice            a load is not a declaration; it replaces itself with N forms
   @alias                      the alias field — and it IS the dispatch, not documentation
   an alias INHERITS its axes  declaring one is a compile error
   the named refusal           8 forms stopped being told they do not exist
```

## ⛔ WHAT COST THE MOST — read all six

**1. A GATE OUTRANKED MY OWN STOP, AND THE GATE WAS RIGHT.** My brief said *"don't touch the eval
arms"*; registering `role = eval` gave the rows handlers, and the registry-first-door gate demanded
those arms be deleted. **A STOP is a claim about the world and the floor outranks it.**

**2. I NAMED THE WRONG WITNESS, THREE TIMES IN ONE DESIGN.** `:wat::rete::i64::+` recorded as `Alias`
class; it is `Fallback`, 4-arg, with `:undefined` machinery. Eight live tests broke. **The rider
implemented as briefed, measured the collision, and reported it for a decision.**

**3. WE MINTED THE CAMPAIGN'S OWN DEFECT, IN UNDER AN HOUR.** An alias and its target declared
contradicting `@Totality`/`@Category`. A per-row judgement (correct for the row it was made about)
outlived the row when I re-pointed the witness. **The cure was structural, never "author carefully".**

**4. THREE MIS-AIMED SABOTAGES, TWO OF THEM NEARLY VERDICTS.** A tenth arm inserted into the wrong
identically-spelled fn 40 lines away. A negative-lift fixture whose first form was a literal, so the
predicate was never consulted. **A green from a mis-aimed probe is indistinguishable from a working
gate.** Run the sabotage; never trust the pass.

**5. A `true`-FOR-EVERYTHING ACCESSOR PASSED THE ENTIRE FLOOR.** Killing a hand-list deleted the only
test asserting its predicate ever answers `false`. **Retiring a subject disarms its negative tests.**

**6. I ARGUED AGAINST A CURE THIS ARC HAD ALREADY DESIGNED.** `NOTE-declaration-position-class-guard`
(2026-06-24) named the position-class property and deferred it *until the registry could answer*. I
recommended the opposite, in a NOTE in the same directory, and found it only because the builder
pushed on a diagnostic.

## ★ WHAT ACTUALLY WORKS

- **Cast `intueri` on a name BEFORE minting it.** It killed a proposed sixth axis by finding a live
  witness (`use!` is `@Category Declaration` *and* evaluates), and named `:Splice` from the word three
  files already used with no naming pressure on anyone.
- **Write the required RELATIONSHIP between two sabotages, not two expectations.** *"`@Category Io`
  must go GREEN and `@Purity Pure` must go RED"* cannot be satisfied by accident; either alone can.
- **Make "I cannot tell" an explicitly correct rider outcome.** It is the only way one under
  completion pressure will choose it over a plausible guess.
- **Riders refuse well and refused seven times.** Every refusal was right.
- **The ratchets drive the next stone.** Four reds this session were gates demanding work I had not
  planned, by name.

## ⛔ THE TWO NUMBER-SHAPED FAILURES, both caught by a rider on the same stone

★★★ **A BAR YOU DERIVE LANDS; A BAR YOU ESTIMATE MISSES.** 1b-i's acceptance table had five
rows. The three I derived from the rule — GAP_A 88→60, GAP_B 106→78, DEBT 95→95 — all landed
EXACTLY. The two I estimated were both wrong:

```
floor "5127 → 5155"   I added 28. Registering a registry row mints no `#[test]` fn — the
                      membership gates are single tests that iterate the registry internally.
rows  "490 → 518"     My census grepped the SUBSTRING `wat_intrinsic("` and counted three PROSE
                      PLACEHOLDERS as registered names: `<fqdn>`, `…`, and `:wat::holon::…` —
                      an ellipsis in a doc comment that DEFEATS a "starts with `:`" filter.
                      Anchored to the attribute SITE the answer is 515; the baseline was 487.
```

⛔ **ANCHOR A CENSUS TO THE SITE, NEVER THE SUBSTRING.** A doc comment that quotes the very
construct you are counting is not a rare hazard — it is what good prose in this repo looks like,
so the false positives scale with how well a module is documented. And the corrective came from
the rider's live measurement, never from re-reading my own claim.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
- ⛔ **THE LSP HAS LIED TEN CONSECUTIVE STONES.** Stale `E0560`/`E0004`/dead-code every time. Run
  clippy; believe nothing else.
- ⛔ **`./scripts/floor.sh > /dev/null 2>&1; echo $?`** then read the Summary from `.floor/latest/raw.log`.
- ⛔ **`git commit -F`, NEVER `-m`** — backticks in a message are shell-interpreted and ate three
  identifiers this session. **`git commit <paths>`, never pathless.**
- ⛔ **REVERTING IS A LOSS.** Get it green instead.
- ⛔ **WAT IS FQDN, ALWAYS.** Anything not a binder is illegal. **Parsing is not legality.**
- ⛔ **Riders: no worktrees, no stash, no sub-agents, everything FOREGROUND, `model: "sonnet"` explicit.**

## ⬜ NEXT

```
✅ 1b-i    DONE (4e1d8e81d) — 29 OpClass::Alias rows. GAP_A −28, GAP_B −28, DEBT UNCHANGED.

Phase 1b-ii  the 6 Form + 2 Redispatch rows whose target IS registered:
             rete::core::{and or if let match fn} · rete::core::List · rete::holon::coincident?
             ⚠ These have NO CheckEnv scheme by construction, so they TRADE GAP_B for DEBT
               (GAP_B −7, DEBT +8). A different deliverable from 1b-i — say so in its own
               acceptance row rather than reporting one number for two mechanisms.
Phase 1b-iii the 17 BLOCKED rows. Gated on 11 core targets that are themselves GAP_B
             population: = · not= · cond · first · PersistentVector · Vector · PersistentMap ·
             Tuple · foldl · map · filter · reduce. Register those FIRST — it takes 11 off the
             107 and unblocks 17 rete rows in the same motion.
Phase 2b     the :undefined fallback machinery — Fallback's 20
Phase 1a-ζ   ann-form · do · stream::lazy
Phase 3a     resolve asks the registry — kills is_reserved_prefix, THE FOUNDING TARGET.
             ★ LOAD-BEARING, not tidying: it is the ONLY thing keeping the macro namespace
               disjoint from the registry's (41 stdlib macros, 0 visible to the registry).
```

⛔ **1b WAS NEVER ONE STONE, AND THE LINE THAT SAID SO WAS MINE.** This SEAM used to read
*"the 54 ALIAS rows — a name and a target each … 66 of the 107 live here."* Measured: 37 of the
54 have a registered target; `no_dangling_or_chained_aliases` reds on the other 17. **You cannot
alias to a name the registry cannot vouch for** — the RULING's forced order, third instance. And
the WORKLIST had already written the warning I walked past: those core verbs *"need their own
stones"*, and *"counting them as one number is how a plan gets written that cannot be executed."*

⚠ **Re-derive the 107 after each phase** — the procedure is in the WORKLIST. That number reaching 0
licenses 3a.

★ **On rider capacity, measured:** 6 rows + 11 edits ≈ 375K tokens / 234 calls — near the ceiling for
*authored* rows. 1b's aliases are transcription, so 20–40 in one rider is plausible. **Parallel riders
need the brief to forbid the shared files** (`intrinsic/special/mod.rs`, the ledgers) with the
orchestrator doing that wiring centrally.

## ⬜ OPEN FORKS — measured, not decided

```
the FOURTH registry     41 stdlib macros, 0 visible. `:wat::core::defn` answers None — same as a
                        name that does not exist. is_reserved_prefix is all that keeps the two
                        namespaces disjoint, and it is the thing 3a deletes.
unquote's containment   "legal only inside X" wants a FIELD naming the enclosing form, not a variant.
                        intueri killed the @Position axis: 2 of its 3 variants ARE @Purity Unevaluated.
role = eval can't stack the shim is keyed on the fn identifier, not the FQDN. Compile error, but the
                        message names a mangled symbol and never says role/eval/stacking.
defclause               lost its named refusal this session; it has no registry row. Register it.
meta_has_doc_axis_key   a COMPLIANT wat-side alias has zero axis keys → misclassified. Not live.
```

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today I named the wrong witness three times in one design
> and broke eight tests. I minted the campaign's own contradiction inside an hour. I mis-aimed three
> sabotages and nearly published two greens as verdicts. I argued against a cure this arc had already
> written down. **Every one of those was caught by a rider, a gate, a cast, or the builder — not once
> by re-reading my own claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** 96 commits, every one green at push. The first
> hand-list is dead, with its meter. Eight forms stopped being told they do not exist. The registry
> learned three regimes, two poles, a category and an alias — and the alias is the dispatch.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`

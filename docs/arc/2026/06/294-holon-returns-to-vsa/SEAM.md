# SEAM — the ONE live breadcrumb. As of 2026-09-01. **The campaign is arc 255: THE REGISTRY BECOMES THE SOLE AUTHORITY.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `251/SEAM.md`, `278/SEAM.md` are PARKED. Arc 109's megafile campaign reached its floor and handed
> off. ⛔ **PARKED IS NOT DEAD.**

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor ............ 5123/5123, 0 FAIL, 17 skipped, ~115s   (scripts/floor.sh, exit read UNPIPED)
clippy ........... 0 under `-D warnings --all-targets`
runtime.rs ....... 19,045   (was 34,152 — the megafile campaign, -15,107)
check.rs ......... 22,613   (its partire map still stands, still uncast by name)
host ............. JohnDesktop · john · ~/work/holon/wat-rs
```

## ⬜ THE CAMPAIGN — read these three, in this order, before proposing anything

```
RULING-the-registry-is-the-sole-authority.md          the builder's doctrine + the census
DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority.md   4 shapes, A picked, 4 phases
NOTE-there-are-two-registries.md                      the finding that opened it
```
(all in `docs/arc/2026/06/255-builtin-registry/`)

**The builder's words, which are the standard:** *"the registry must be the thing who knows all
names.. who delegates to the code who performs for those names... what you query to know what
exists... what they take.. what it returns.... the properties these names have.... **we must
eliminate every source of duplication or inconsistency**."*

## ★★★ THE PROGRESS METER IS ALSO THE FINISH LINE

```
GAP_A 89 · GAP_B 112 · DEBT 83 · TYPES_UNCHECKED 10 · KNOWN_UNREVIEWED 28
```

These five ledgers **exist only because the split does.** When they are empty and their files
deleted, the RULING is satisfied. ⚠ **A stone that claims to eliminate duplication and moves none of
them has eliminated none.** Check them before and after every stone.

## THE AUTHORITIES STILL COMPETING (measured, not recalled)

```
RETE_OPS          src/rete/vocabulary.rs   74 rows  — 55 of GAP_A's 89 ARE these
SPECIAL_FORMS     src/special_forms.rs     35 rows  — 23 unregistered; calls ITSELF a registry
register_builtins src/check.rs            350 env.register schemes
literal arms      src/check.rs            118 type-grammar arms
RETIREMENT_TABLE  src/remedy/retirement.rs 144 rows
residues          intrinsic_meta 37 · is_expand_time_legal 54 (16 already DEAD) · effectful_by_prefix 8
MacroRegistry     src/macros/registry.rs   41 stdlib macros — 0 visible to the registry.
                  ★ `:wat::core::defn` answers None, same as a nonexistent name.
⛔ NINE MORE       NOTE-the-sloppy-registries-a-measured-census.md — incl. FIVE hand-lists of
                  "what kind of form is this head", INCONSISTENT on disk (`def` is a mutation to
                  freeze.rs and is not one to runtime.rs). @Category is the vehicle; 1a unblocks it.
is_reserved_prefix src/resolve/reserved.rs  THE ARC'S FOUNDING TARGET, still on disk
```

⚠ **NOT duplicates — do not delete these:** `constructor_meta`/`accessor_meta` DERIVE from the frozen
`TypeEnv`; `step_list`'s 19 names declare a capability with `NoStepRule` as its honest refusal.
**A campaign that cannot tell a derivation from a duplicate deletes correct code.**

## ⛔ WHAT COST THE MOST TODAY — read all five

**1. NINE MISCOUNTS, EVERY ONE A PATTERN THAT MATCHED A SUBSET OR A SUPERSET.**
`grep -c "registry()"` matched `macro_registry()`. `grep "Binding::"` swept in `LetBinding::` (a
different enum) — I reported 46 sites; the truth was 54 raw, 43 real. `special_forms.rs` "19 rows"
was 30. A size regex omitting `mod`/`impl` reported 10,473 lines for 30 fns in a 24,103-line file.
A caller census blind to `mod tests`. A ledger regex that read quoted names out of COMMENTS and had
me alleging a defect in correct work. ★ **Every single correction came from a rider, a cast, the
compiler, or the floor. Not once from re-reading my own claim.**

**2. A BRIEF'S TABLE SHIPPED A REGRESSION.** I wrote that `and`'s eval arm called `eval_and_tail`.
It called `eval_and` — I grepped and took the FIRST match, which was the *tail* arm. The rider
followed the table, `eval_and`/`eval_or` were orphaned and deleted, and 14 lint tests went red.
`eval_and_tail`'s own doc names what I broke: *"this arc's law that nothing weakens quietly."*

**3. TWO DESIGNS REFUTED BY THEIR OWN PROBES — both times before briefing, both times good.**
`step_list` is not a door (a closed 19-name competence table; a guard would promise step rules for
~445 rows that have none). The tail door fixes no live bug (`eval_tail`'s fallthrough already reaches
the registry) — it grants a *capability*: `impls` carries `(role, SOURCE TEXT)`, so a form can declare
a tail impl the registry can never call.

**4. A VACUOUS PROBE RETURNED A PERFECT SCORE.** My first doc→TypeScheme probe compared through the
very projection whose lossiness it existed to test: 386/386. Comparing `TypeExpr` structurally found
2. **A perfect result is when to suspect the instrument.**

**5. INSERTING A TEST SILENTLY DISARMED ANOTHER.** I anchored on the `fn` line, not the `#[test]`
line, and stole an existing test's attribute. **The floor read 5114 on both sides — my new test
replaced the disarmed one one-for-one.** Only clippy's `dead_code` saw it.

## ★ WHAT ACTUALLY WORKS

- **The ratchets do the enforcing.** Registering `fn`/`match` made a gate *someone else wrote* demand
  `KNOWN_UNREVIEWED` shrink. The dead-arm gate forced the eval door's own sweep. **Build the gate,
  then let it drive the next stone.**
- **Freeze NAMES, never counts** — and it applies to the floor's own total (see lesson 5).
- **Sabotage every gate, both directions, before believing it.** Five gates this session; the tail
  door's probe SIGSEGVs when the guard sits one block too high, and passes at depth 10 either way.
- **Riders refuse well when the brief gives them an escape clause.** One refused to delete
  `special_forms.rs` rows and was right (they were the only path to `and`/`or`). One refused to
  reshape a handler for the macro. One reported its sabotage as *unverified* rather than claiming it.
- **Cast a ward when the question is "is this one thing or two."** `solvere` found a THIRD registry
  my census had missed entirely.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
- ⛔ **THE LSP LIED EIGHT CONSECUTIVE STONES.** Stale `E0603`/`E0004`/`E0560` every time. ⚠ And twice
  there WERE real problems it never mentioned. Run clippy; believe nothing else.
- ⛔ **`./scripts/floor.sh > /dev/null 2>&1; echo $?`** then read the Summary from `.floor/latest/raw.log`.
- ⛔ **`git commit <paths>`. NEVER pathless.**
- ⛔ **Riders: no worktrees, no stash, no sub-agents, everything FOREGROUND, `model: "sonnet"` explicit.**
- ⛔ **A brief's every pairing must be verified against the ARM, not a grep's first hit.** Lesson 2.
- ⛔ **REVERTING IS A LOSS.** The builder stopped me mid-revert. Get it green instead — the fix was
  one honest narrowing away.
- ⛔ **WAT IS FQDN, ALWAYS.** Anything that is not a binder is illegal; bound symbols are shadow-FQDN
  in `$bound`. ★ A short head is not a rendering style, it is not-wat — the whole clojure-ination
  migration exists to force this. I shipped short heads in 1a-α citing a probe that proved the reader
  PARSES them. It does. **Parsing is not legality.**

## ⬜ NEXT — Phase 1a, and one named gap

✅ **CLOSED by Stone 1a-α (`b9546b097`).** `signature_of_defn` renders a row's declared `@syntax`
through wat's own reader, `render-doc`'s precedence, so the two renderers agree. `match` now signs
`(:wat::core::match <scrutinee> (<pattern> <body>) ...)` — the dead `-> <T>` ASCRIPTION SLOT is gone.
⚠ `match` is FULLY SUPPORTED and always was; what died (2026-06-22) is match ASSERTING A RETURN TYPE.
Only `fn` declares types. ★ `@syntax` was
the right vehicle, not `@arg`: `@arg` carries a TYPE and those slots are syntactic positions.
A sabotage-proven gate parses every declared `@syntax` at floor time, with a non-vacuity floor.

⛔ **BUILDER'S SEQUENCING RULING, 2026-09-02:** *"we continue to add names to the registry.... then
we attack the hand lists."* **Register the population first; flip the consumer second.** The campaign
tried to run ahead twice and was stopped both times by a red or a refusal, never by the plan.

```
✅ DONE     17 of special_forms.rs's 35 rows registered — incl. all 8 declaration forms
✅ KILLED   freeze::is_liftable_declaration_head (9 names) + its meter — the FIRST hand-list
Phase 1a-γ  the homoiconic 8   quote · quasiquote · unquote · unquote-splicing ·
                               macroexpand · macroexpand-1 · forms · struct->form
Phase 1a-δ  the loaders 4      use! · load-file! · digest-load! · signed-load!   ⬅ unblocks the
Phase 1a-ε  the config 2       set-redef! · set-eval-redef!                      ⬅ mutation pair
Phase 1a-ζ  the remainder 3    ann-form · do · stream::lazy
⛔ 1 UNREGISTERABLE  defstruct — a stdlib MACRO. See the fourth-registry NOTE.
Phase 1b  RETE_OPS' 74 — BLOCKED on 1a
Phase 2a  core_name — the alias field, the one genuinely homeless one
Phase 2b  the :undefined fallback machinery — does NOT decompose
Phase 3a  resolve asks the registry — kills is_reserved_prefix, THE FOUNDING TARGET
          ★ and it is LOAD-BEARING, not tidying: is_reserved_prefix is the ONLY thing
            keeping the macro namespace disjoint from the registry's.
```

⚠ **Flipping the blanket-accept today fails 578 of 599 corpus files.** Measured. The order is forced:
registry answers → consumer asks → duplicate dies.

★ Re-derive the 121 worklist after each phase — the procedure is in
`WORKLIST-the-121-the-registry-cannot-vouch-for.md`. That number reaching 0 licenses Phase 3a.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today I miscounted nine times, shipped a brief whose table
> caused a live regression, wrote a probe that scored 386/386 by measuring in the space it was trying
> to escape, and disarmed a test while the floor stayed green. **Every correction came from outside
> me.** The riders refused three of my instructions and were right all three times.
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** 49 commits, every one green at push. `runtime.rs`
> 34,152 → 19,045. Two dispatch doors opened that never existed. Five ratchets built and
> sabotage-proven. A third registry found. A two-month-dead ascription slot surfaced. And the campaign that
> ends all of it is drawn, phased, and instrumented with a falsifiable finish line.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`

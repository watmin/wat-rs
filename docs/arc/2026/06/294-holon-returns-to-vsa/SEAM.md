# SEAM — the ONE live breadcrumb. As of 2026-09-03. **Arc 255: THE REGISTRY BECOMES THE SOLE AUTHORITY.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), ground HEAD against the disk, and read this whole file before you touch
> anything.

> `251/SEAM.md`, `278/SEAM.md` are PARKED. ⛔ **PARKED IS NOT DEAD.**

> ## ⚠⚠ A RIDER WAS IN FLIGHT WHEN THIS WAS WRITTEN — CHECK BEFORE YOU TOUCH ANYTHING
>
> **Stone:** `[[DESIGN-STONE-stepvalue-is-watast-and-the-round-trip-is-lossy]]` (255/).
> Released at HEAD `f2d1f0b1e`. **The working tree may hold its uncommitted work.**
>
> ```bash
> git status --short          # its edits land in src/holon/ast.rs + src/runtime.rs
> pgrep -af 'cargo|nextest'   # it may still be building
> ```
>
> If work is present and no report reached you, the agent ended without reporting — **do NOT
> re-run cargo to check on it** (a second build against the same `target/` lock is FM 18, and any
> number taken while its job is live is an instrument artifact). Confirm via `git status`, then
> resume it by name; a resumed rider keeps its full context. If the tree is clean, it never
> started or its work is already in a commit above `f2d1f0b1e`.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor ............ 5129/5129, 0 FAIL, 17 skipped, ~117s   (scripts/floor.sh, exit read UNPIPED)
clippy ........... 0 under `-D warnings --all-targets`
registry rows .... 553    ⛔ COUNT IT ANCHORED TO THE ATTRIBUTE SITE, never a substring:
                          grep -rhoP '^\s*#\[wat_(special_form|intrinsic)\("\K[^"]+' src/ \
                            --include=*.rs | sort -u | wc -l
                          A loose search counts PROSE PLACEHOLDERS — `<fqdn>`, `…`, and
                          `:wat::holon::…`, which defeats a "starts with `:`" filter.
host ............. JohnDesktop · john · ~/work/holon/wat-rs
```

## ⬜ THE CAMPAIGN — read these before proposing anything

```
RULING-the-registry-is-the-sole-authority.md                the doctrine + the census
RULING-rete-forged-the-paths-the-registry-claims-the-tools.md  properties must be QUERYABLE
DESIGN-CAMPAIGN-the-registry-becomes-the-sole-authority.md   4 shapes, A picked, the SEQUENCING
WORKLIST-the-121-the-registry-cannot-vouch-for.md            re-derived 5× → 37
```
(all in `docs/arc/2026/06/255-builtin-registry/`)

## ★★★ THE METER

```
GAP_A 49 · GAP_B 42 · DEBT 121 · TYPES_UNCHECKED 10 · KNOWN_UNREVIEWED 13
the corpus: 37 names — 31 VERB POPULATION · 6 NON-VERB ARTIFACTS   (121 → 107 → 71 → 39 → 37)
failing corpus files: 130 of 615        total exposure: 638 sites   (was 1343 — a 53% cut)

⭐ THE REGISTRY CAN NOW BE ASKED. `(:wat::intrinsic::rows)` returns one typed Row per entry.
   ./target/release/wat wat-scripts/scratch-pad/255-registry-census.wat
   553 rows · 70 SpecialForm · 37 alias · 87 Variadic · 517 no @syntax
   totality: Partial 50 (THE WORK LIST) · Unreviewed 378 · expand Unreviewed 295 · BOTH 270
   ⚠ Only Totality and ExpandTime HAVE an Unreviewed pole. Purity/Determinism/Category do NOT —
   they are complete BY CONSTRUCTION. The grading endgame is TWO axes, not five.
   ⛔ This is a MEASUREMENT, never a ratchet. It must NOT derive the four ledgers: a gate freezes
   NAMES so it can DISAGREE with the present; one computing both sides always agrees with itself.

⭐ DEBT 121 IS THREE POPULATIONS, MEASURED — 41 SpecialForm (a rank-1 scheme is the WRONG SHAPE)
   + 60 with a custom `infer_*` arm (a STRONGER authority; a scheme would be DUPLICATION the
   RULING forbids) + **20 GENUINELY OWED**. The ledger is 6× the real debt.
```

⚠ **DEBT RISING IS NOT A REGRESSION** — a row with no `CheckEnv` scheme converts an *invisible*
absence into a *named* one. `=`/`not=` joined it this session **on purpose**: they are dispatched
by `infer_equality`, a keyword-head arm, and minting them a rank-1 `TypeScheme` would be a second
authority for a signature that function already owns.
⛔ **DEBT IS STILL TWO POPULATIONS AND ONE OF THEM IS NOT DEBT** — `Kind::SpecialForm, no scheme`
("a rank-1 scheme is the WRONG SHAPE" — a census of the un-schemeable, which should never reach 0)
vs `Kind::Intrinsic, no scheme` (genuinely owed). The finish line is **unreachable while one number
means both**, and the `Kind` split MISFILES every alias because `Kind` is stamped by the
registration VEHICLE, not the verb.

## ✅ WHAT THIS SESSION SHIPPED — 27 commits

```
REGISTRY (arc 255)
  = and not= REGISTERED @Totality Partial — five compactions of a hold ENDED. The by-name
    totality placeholder is DELETED: `Some(Unreviewed) | None => false`, zero names.
  reduce is a defalias for foldl · Vector · HashMap · HashSet registered (all MEASURED Partial)
  TWO special-form tables die — 314 lines; a third was a `const` inside eval_apply
  holon::literal reclassified SpecialForm — a hand-list came back because a ROW LIED
  the round trip CLOSES 432/432 and FROZEN_SPELLING_MISMATCHES is EMPTY — zero tolerance
  ⭐ THE REGISTRY CAN BE ASKED — `(:wat::intrinsic::rows)` returns one typed Row per entry

HOLON-AST (arc 109/294's ruling, finally TRUE)
  eval::walk · WalkStep::Skip · StepResult::StepTerminal/AlreadyTerminal · core.wat's ->/->>
  all face :wat::WatAST now. Re-measured: EVERY remaining HolonAST site is VSA.
  ⭐ 294's ruling — "HolonAST only for VSA/HDC" — is MEASURED TRUE, not asserted.
```

★★★ **THE SESSION'S THESIS, the builder's own words:** *"the registry is forcing the discovery of
bad practices."* Not a metaphor. A crutch survives while nothing asks it a question; the registry's
demand — every name answerable, every property declared and gated — is what made each surface state
what it is. Reflection storing sketches as `HolonAST` survived six months because no gate asked.

★ And the category error under all of it: *"holon-ast is hypervector of data … edn is a wire format
of data."* **Neither is a syntax tree.** `HolonAST` was a crutch taken while `WatAST` was immature,
and it has been lowering literals into whatever the hypervector could hold ever since.

## ⛔ THE SURFACES ARE NOT ONE FENCE

```
wat/rete/compile.wat   where · accumulator · then-item-fence
                       pure ∧ det ∧ total ∧ RETE (Law A) — ALL FOUR, ARMED, CORRECT.
                       A generic core verb was ALREADY refused in rete. It was never allowed there.
wat/telemetry/journal.wat  sift-logs · sift-arena — THREE axes, no Law A, deliberately.
src/freeze.rs:790      the SIGMA-FN gate — a third fence entirely.
```

★ **Sift now refuses a predicate comparing a foreign `:wat::core::Value`**, and that is the fence
being correct. `ForeignRecord/get` returns `(Option Value)`; there is no `Value`→`String` coercion
in the 13-row `:wat::edn::` surface; `Value`'s declared domain admits `Fn`. The comparison is
genuinely `Partial` and **`properties_of(name, arg_types)` would answer `Partial` too** — which is
precisely why waiting for it never would have unblocked these rows. `probe_arc278_sift_arena` now
carries the `Fault` out through a `:Refused` variant and demonstrates a typed comparison sifting
fine *beside* the refused `Value` one.

## ⬜ OPEN FORKS — measured, not decided

```
★ THE NEXT PICK IS A REAL CHOICE, and the worklist's SHAPE changed this session:
    :wat::eval-ast!    1 name,  331 sites  ← 52% OF ALL REMAINING EXPOSURE, 3× the next entry
    :wat::rete::*     ~26 names, ~250 sites ← more NAMES, the old Phase-1b block
  More sites vs more names. Different jobs. Pick deliberately; do not default to the bigger list.

restore the Value-comparison capability   needs a comparable-subset type or a coercion verb.
                      No consumer waits but sift. Named, not deferred-in-prose.
alias vs RESTRICTION  the 8 blocked rete equality rows point at the GENERIC core_name. An alias
                      means IS; these are RESTRICTED TO. The registry cannot say that.
19 rows lie about arity  #[wat_intrinsic] derives Arity from the RUST SIGNATURE SHAPE; a
                      &[WatAST] param ⇒ Variadic with no shim check.
                      [[NOTE-nineteen-rows-declare-Variadic-and-enforce-a-fixed-arity]]
derive's declare ptr  role = declare names parse_derive_form; the real mutation is an inline arm.
the FOURTH registry   41 stdlib macros, 0 visible — AND every wat-defined verb. A wat-side
                      `{:totality …}` map lands in `sym.binding_metadata` (declare/register.rs),
                      NOT in `registry()`. Proven: `:wat::core::count` is a wat defalias with no
                      registry row. Only THREE wat verbs carry axis decorations at all.
DEBT is two populations  see the meter's warning. Splitting it is a prerequisite to the finish line.
```

## ⛔ WHAT COST THE MOST THIS SESSION — every one caught by a gate or the builder, none by re-reading

**1. I HANDED A RIDER A LIST INSTEAD OF AN INSTRUMENT, THREE TIMES.** Worst form: I wrote STOP-2
to catch a wrong census and **derived it from the same wrong census**, so it could not fire. I
swept 3 of the repo's **7** `.wat` roots (`wat-tests/` holds 81 files). A guard built from the
claim it guards always agrees. `[[feedback_a_stop_trigger_inherits_the_census_blind_spot]]`

**2. AND THE OPPOSITE ERROR, SAME DAY.** STOP-1 on the next stone was drawn so tight that normal
line-number drift in a preserved doc block stopped the whole stone — nearly a sixth hold on
`=`/`not=`. A guard wrong in either direction costs the same.
`[[feedback_a_guard_drawn_too_tight_makes_the_honest_path_noncompliant]]`

**3. A MIS-AIMED PROBE GAVE ME SEVEN FALSE GREENS.** The stdlib is `include_str!`ed
(`src/load/stdlib.rs`), so editing `wat/seq.wat` and running the stale binary tested **nothing**.
Caught only because an undefined verb appended to the stdlib *also* read `exit=0`.
⛔ **Every `.wat` stdlib probe needs a rebuild AND a sabotage canary.**

**4. A PATCH FIXES ONE COPY OF A CLAIM.** Three siblings in `seq.wat` still asserted the retired
two-arity shape after the adjacent block was corrected; `USER-GUIDE.md` presented two aliases as
"live" that exist nowhere on disk.

**5. MY OWN CENSUS INSTRUMENTS WERE WRONG FOUR TIMES** — a tokenizer returning impossible
"7-arity" rows; an `awk` ledger count returning 0 for all four; a grep that could not tell an
entry from a comment. **Every count was corrected by validating the instrument, never by
re-reading the number.**

## ★ WHAT ACTUALLY WORKS

- **The ledger ratchets name the exact edit.** Let them drive; never pre-compute their lists.
- **Derive every acceptance row from the rule.** Every derived row this session landed EXACTLY —
  `39 → 37`, `GAP_B 44 → 42`, `DEBT 119 → 121`, `registry 550 → 552`, all predicted before the strike.
- **Show a gate FIRING before shipping it.** The new 2-arity witness was sabotaged to 3-arity and
  went red before it was trusted.
- **A prose citation names a SYMBOL, not a LINE** — this arc's own stone, and the permanent cure
  for drifting doc blocks. 8 of 12 line citations were already false when it was measured.
- **Run the substrate as the census.** Registering honestly and reading the floor found the real
  blast radius in one run; no amount of grepping would have.

## ⛔ RULES THAT STILL COST TIME

- ⛔ **THE ORCHESTRATOR RUNS THE FULL FLOOR. A RIDER'S TARGETED GREEN IS NOT A VERDICT.**
  Give riders `binary_id(wat)` — where every ledger and registry gate lives — not a list of names.
- ⛔ **THE LSP LIES.** Run clippy; believe nothing else.
- ⛔ **`./scripts/floor.sh > /dev/null 2>&1; echo $?`** then read the Summary from `.floor/latest/raw.log`.
- ⛔ **`git commit -F`, NEVER `-m`** — backticks are shell-interpreted. **`git commit <paths>`.**
- ⛔ **REVERTING IS A LOSS.** Narrow the stone instead; preserve held work in a NOTE.
- ⛔ **DELETIONS MUST CLEAR A HIGH BAR** — *"we augment as they need."* A test of a retired
  behaviour becomes a NEGATIVE WITNESS of the retirement; it is not deleted.
- ⛔ **Riders: no worktrees, no stash, no sub-agents, everything FOREGROUND, `model: "sonnet"`.**

## ⬜ NEXT — read `[[SEQUENCING-the-only-chain-that-gates-the-founding-target]]` FIRST

```
0  ⚠ FINISH THE IN-FLIGHT STONE (banner at the top). StepValue::Terminal/AlreadyTerminal still
     carry HolonAST internally, and eval-step! CORRUPTS rationals and bigints because of it:
       (quote 1/2) -> 1/2        (eval-step! (quote 1/2)) terminal -> "1/2"   a StringLit
     MEASURED this session. i64 is the control and survives.

REGISTRY — the chain to Phase 3a (resolve asks; is_reserved_prefix dies — THE FOUNDING TARGET)
  1  the 2 remaining orphan core_name targets: cond · reduce. Both the FOURTH REGISTRY —
       a stdlib defmacro and a wat-side defalias, neither holdable by registry() today.
       They gate ~29 RETE_OPS rows (no_dangling_or_chained_aliases PANICS on an unregistered
       target, intrinsic/mod.rs:2119).
  2  the SIX NON-VERB ARTIFACTS need a RULING, not a registration. ⛔ The corpus can NEVER reach
       0 by registering. Nothing else in the campaign produces this work.
  3  eval-ast! + eval-with-defs! (334 sites) · then Phase 3a is decidable.

READY TO RELEASE, no design left
  ⭐ wat-scripts/fixes/bare-none-keyword-to-fqdn.wat — :None -> :wat::core::None, 94 sites/20 files.
     Dry-run PROVEN with two negative controls (:app::Result's :Err untouched; wat/cache.wat's
     FQDN sites untouched), idempotent, migrated file --checks and RUNS identically.
     ⛔ :Ok and :Err are NOT in scope — 194 :Ok sites are OTHER enums' variant declarations, and
     :app::Result declares :Err. Renaming either would CORRUPT unrelated enums.

PARALLEL, pick by value
  Phase 3b — check asks the registry. UNBLOCKED (432/432). Kills register_builtins' 302 of 325.
  the DEBT split 121 = 41 wrong-shape + 60 stronger-authority + 20 GENUINELY OWED.
  the 270 both-axes grading batch — holon 91 · kernel 49 · time 41 · io 29.
  reflect/verbs.rs's 2 holon conversions — REGISTRY WORK, deferred for SIZE only (builder's ruling).
  109/NOTE-eval-walk…: 9 golden <HolonAST> literals · a follow-up :wat::core:: WatAST leaf ctor.
```

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Today I got ONE census wrong FOUR TIMES — counting verb
> SCHEMES when the question was the DECLARED SURFACE; grepping `--include=*.wat` when wat also lives
> inside Rust string literals; missing `wat/core.wat` entirely; and mis-filing the SIMILARITY cache
> as a lapse **by reading a file's header and never reaching Stone 4 two hundred lines down**.
> A HEADER IS NOT THE FILE. I wrote a STOP trigger from the same wrong census it was meant to catch,
> and four hours later one so tight the honest path was non-compliant. **SEVEN comment-caused errors
> in one session** — "poisoned at type-check time", "cannot express any Seqable", "stays unhomed",
> "no NativeHandler" — every one FALSE, every one read as authority. **Not one was caught by
> re-reading my own claim.** The builder caught three; riders caught three; a gate caught one.
>
> ⚠ **AND THE INSTRUMENT THAT FINALLY WORKED WAS THE COMPILER**, on the builder's instruction —
> *"strike the heresy where they stand; the compiler identifies the heretics immediately."* Retyping
> three fields produced ZERO rustc errors (wat types are DATA in `types.rs`) and 17 located wat-checker
> failures. **Reach for the compiler before the fourth grep.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** `:wat::core::` is DONE with nothing held. The
> placeholder that lied about three verbs is deleted. The registry can now be ASKED about itself.
> 294's holon ruling is measured TRUE. Corpus 1343 sites → 638.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`

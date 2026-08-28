# SEAM — the ONE live breadcrumb. As of 2026-08-27. Arc 255: the homes campaign — and the registry became apply's authority.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own voice —
> which is why it will feel like *continuing* rather than *waking*, and **that feeling is the failure.**
> Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a disk copy),
> ground HEAD against the disk, and read this whole file before you touch anything.

> `255/SEAM.md`, `251/SEAM.md`, `278/SEAM.md` are PARKED and point here.
>
> ⛔ **PARKED IS NOT DEAD — IT MEANS THE SCOPE NARROWED.** A parked seam still holds **its own arc's
> state**. `255/SEAM.md`'s banner says so itself: *"come back here only for arc 255's own state."*
> **2026-08-25: I read PARKED as "skip it", worked arc 255 all day out of this file alone, and had to
> be told by the builder what 255 is FOR.** If you are working an arc, read ITS seam.

## GROUND FIRST

> **THE FRESHNESS PROBE — DERIVE IT, NEVER TYPE IT.**
> ```bash
> S=docs/arc/2026/06/294-holon-returns-to-vsa/SEAM.md
> git log --oneline "$(git log -1 --format=%H -- $S)..HEAD"
> ```
> **Empty → nothing moved.** Non-empty → every commit listed outranks every line below.

⚠ `git status` FIRST. `pgrep -af 'cargo|nextest'`.

```
floor .......... 5065/5065, 0 FAIL, 19 skipped, ~94s   (scripts/floor.sh, exit read UNPIPED)
                ⚠ ACCOUNTED BY NAME, NEVER BY ARITHMETIC.
clippy ......... 0 under `-D warnings`
host ........... JohnDesktop · john · ~/work/holon/wat-rs
stash .......... NO LONGER PROTECTED. Builder, 2026-08-26: *"the stash is backed up on another host
                 as well.... i don't think we need to worry about protecting stash."* The lifecycle
                 strike's implementation is PRESERVED IN THE TREE at
                 `docs/arc/2026/06/278-rules-engine/SKETCH-connection-lifecycle-ops-*.diff`.
                 ⚠ If you ever protect a stash again, name it by its MESSAGE, never `stash@{0}` —
                 an index renumbers on any push, and this seam pointed at the wrong object for a day.
```

## ⛔ THE THESIS — read this before you pick any next step

**ARC 255 EXISTS TO KILL ONE LINE.** `src/resolve/walk.rs:268`:

```rust
if is_reserved_prefix(head) { return true }     // :wat:: — the WHOLE language namespace
```

Every `:wat::`-prefixed name is accepted **unchecked** — real, retired, or invented. Proven with a
positive AND a negative control: a name that MOVED and a name that NEVER EXISTED both `--check`
clean. Its comment defers to the type checker; **the type checker does not do it.**

**THE REGISTRY IS THE ANSWER, AND THE HOMES CAMPAIGN IS HOW THE REGISTRY GETS POPULATED.** A home is
not carved to make a file smaller. It is carved so a namespace's names become *addressable*, so the
resolver can consult `sym`-or-registry instead of waving the prefix through.

⚠ **THE ENDGAME IS SIZED: 2,539 of 5,059 tests fail** if the blanket-accept is default-denied today
(measured by imposing it as a throwaway probe, then reverting). That is not a stone; it is where the
campaign is going. Its gate is already written and disarmed —
`tests/wat_lang/probe_undefined_builtin_resolves.rs`, *"unlock when we circle back to arc 255"* —
and **that gate cannot be un-ignored by anything smaller.**

## WHERE WE ARE

```
registered NAMES ........ 380        registered namespaces .. 27        retirement rows ... 144
                ⛔ THIS ONE POPULATION WAS COUNTED WRONG THREE TIMES IN ONE DAY — 385, 434, 381 —
                  and every time because the instrument could see PROSE. 385 counted attribute
                  SITES (4 are `<fqdn>` doc placeholders); 434 counted 48 bare `#[wat_intrinsic]`
                  mentions inside doc comments; 381 matched attribute text ANYWHERE in a file, so
                  `src/intrinsic/holon/mod.rs:9`'s doc comment — which writes `:wat::holon::…`, a
                  placeholder spelled differently from the `<fqdn>` the filter knew — became a
                  "registered name". And "30 homes" was `ls src/intrinsic | wc -l`, a FILE count.
                  ANCHOR TO ATTRIBUTE POSITION. Verified name-for-name against the awk census:
                    grep -rhoP '^\s*#\[wat_intrinsic\(\s*"\K[^"]+' src/ --include=*.rs \
                      | sort -u | wc -l          # 380
                  `[[feedback_a_file_count_is_not_an_item_count]]`
                  `[[feedback_validate_a_search_pattern_before_trusting_its_count]]`
src/ root .rs files ..... 22         runtime.rs 34,252  (was 40,441 — DOWN 6,189 in one day)
dispatch arms left ...... 168        ⚠ 191 was the honest figure before HOME-13 deleted 44
```

**SHIPPED 2026-08-26/27 — the homes campaign, and then the thing under it:**

```
A-i … E-iv   the scalars + collections   numerics · bigint/rational · maps · vectors · sets · keyword
F            the String verbs            the last per-type family
HOME-8       holon        95 verbs       src/holon/ (algebra) + src/intrinsic/holon/ (interface)
HOME-9       std DIES     14 verbs       math · stat · seq — and seq became Seqable-generic
HOME-10      math/stat/seq homes         the row HOME-9 omitted: does the home EXIST?
HOME-11      edn          13 verbs       17 producers kept their provenance
HOME-12      ast          10 verbs       all ten producers; found a blind spot in the purity gate
STONE G      provenance                  NativeHandler -> TrackedValue; E-iv's loss reversed
STONE N      apply's authority           dispatch_substrate_impl now consults the registry
HOME-13      44 dead arms deleted        REFUSED once, retracted, reinstated on new evidence
```

★ **`env`/`sym` IN A SIGNATURE IS THE SEAM `runtime.rs` HAS BEEN MISSING.** A fn taking them is
BINDING (an interface); one taking neither is ALGEBRA. The compiler already enforces that line on all
941 fns. It decided every classification in HOME-8, twice, with zero ambiguous cases.

**`wat.core/+` IS ALREADY A DEFCLAUSE** (`wat/core.wat:58+`, arc 300 stone C1) and **every one of its
four numeric arms now points at a `wat.<type>/` home.** The builder's `wat.core/+ => [wat.i64/+ …]`
was not a future shape; it was on disk with two arms in the junk drawer, and D finished the set.

★ **RULED + PARKED 2026-08-26 — `:wat::core::String/*` AND `:wat::string::*` COEXIST ON PURPOSE.**
They are not an inconsistency and **this is not an open naming question.** `String/<method>` is the
namespace `extend-type` GENERATES — instance methods on the String type — and `:wat::string::<verb>`
is the function home. Stone F evicted five plain functions from the former; it did **not** kill it,
and `extend-type :wat::core::String` still mints there (proved: `DuplicateDefine` on
`:wat::core::String/tag`). Builder: *"that would put 'instance method' in :wat::core::String/* which
is logical … keep them there for now …. we'll figure out the naming problems later."*
⚠ Production `wat/` extends **Vector · PersistentVector · List · Stream** (Seqable) and
**ThreadOpts · ProcessOpts** (Locus) — it does **not** extend String, so String's slice is currently
VACANT. A vacant namespace reads like a mistake; it is a ruled, deliberate reservation.
`[[feedback_a_rejected_option_returns_in_new_clothes]]`

⚠ **`:wat::set::` and `:wat::list::` ARE RESERVED AND UNCLAIMED.** Persistent set and list do not
exist yet; the builder has ruled they are coming. The unmarked name belongs to the flavor that will
become the DEFAULT. `HashSet`→`hashset` and `List`→`linkedlist` are marked because both are
`Arc<std::…>` copy-on-write — the same side of the axis as `HashMap`/`Vector`, not the `rpds` side.
**Squatting an unmarked name guarantees a second migration of the family that ends up unmarked.**

## ⛔ THE LESSONS THAT COST THE MOST

**1. A PROBE ANSWERS THE QUESTION YOU ASKED, NOT THE ONE YOU MEANT.** I sabotaged a registered
handler, saw the direct call change, and wrote *"proven by experiment, not by reading"* into a brief
concluding 44 arms were dead. The experiment was correct and showed *which path serves the direct
call*. It never showed *nothing else calls the arm* — `apply` did. A rider fired STOP-1, deleted
nothing, and was right. **To prove a thing dead, sabotage THE THING, not its replacement.**
`[[feedback_a_probe_answers_the_question_you_asked_not_the_one_you_meant]]`

**2. A CENSUS WITHOUT ATTRIBUTION IS NOT A CENSUS.** That same stone grepped arms whole-file and
intersected names with registrations. Deadness is a property of a name **in a function**; there are
three dispatch fns and I had measured one. The count was wrong by 23% and I quoted it across three
consecutive stones while choosing what to carve.
`[[feedback_a_census_without_attribution_is_not_a_census]]`

**3. "HOME" MEANS TWO THINGS, AND I CONFLATED THEM THREE TIMES IN A DAY.** A FILE-DOMAIN carve
(loose root files → `src/<domain>/`) is not a REGISTRY home (dispatch arms → `src/intrinsic/<ns>/`).
HOME-5/6/7 all shipped the first kind on 2026-08-25; I called them "drawn but unbuilt" three times
because I was checking `src/intrinsic/`. **Only the registry kind takes a name away from
`walk.rs:268`.** Measure `src/intrinsic/<ns>` before calling a home built.

**4. AN ACCEPTANCE ROW THAT DOESN'T MEASURE THE DELIVERABLE SHIPS A HALF-STONE GREEN.** HOME-9's
seven rows all measured *naming* — new spelling runs, old refused, `:wat::std::` gone. Not one asked
whether a home EXISTED. The rider satisfied all seven and the homes were never built; HOME-10 had to
finish it. **Row 0 of every home brief is now: does the home exist?**

**5. THE TREE KEEPS ALREADY SAYING IT.** Three stones in a row turned on prose sitting in the file:
`runtime.rs:11652` explained the `apply` split-brain that would have prevented HOME-13's error;
`collection/transform.rs` confessed the `list::` verbs reject a `List`, four times verbatim;
`intrinsic/mod.rs` documented that `mod string` was `pub(crate)` solely for arms I was about to
delete. **Read the neighbourhood before measuring it.**

**6. A RIDER WITH THE AUTHORITY TO REFUSE IS CHEAPER THAN A REVERT.** HOME-13's STOP-1 saved a
deletion that would have broken `:wat::core::apply` across nine namespaces. Two other stones were
corrected by riders catching defects in my own briefs (a count of 7 that was 6; a stale finding I
carried forward without re-checking). **Write the STOP that names the most likely way the stone goes
wrong — then believe it when it fires.**


## ★ WHAT ACTUALLY WORKS

- **The three-phase shape**: register (both spellings live) → codemod the corpus → retire. Seven
  stones, no exceptions. When a bootstrap file (`wat/core.wat`) uses a migrating verb, the order is
  not negotiable — retiring first takes down EVERY program.
- **wat-fix RULES codemods.** `rename-keyword-prefix` is a **silent no-op** on `::`-terminated
  prefixes; the rules form is the one that works. KEYWORD-ONLY guard mandatory. Dry-run on `/tmp`,
  diff, apply, prove idempotence. Recorded fixes in `wat-scripts/fixes/` — copy the nearest.
- **wat-grep for populations.** Text said 1613 where structure said 1495; the 76 were comments and
  string literals a codemod must never touch. `wat-scripts/grep/` is the corpus of encoded questions.
- **DERIVATION over hand-lists.** `check.rs`'s `register_builtins` walks the registry and aliases
  rather than restating 36 names — and it **caught a real defect on its first run** (the variadic
  `max-of` divergence, at check time, loudly) where a hand-list would have shipped wrong arity.
- **Breaking the door.** Every gate this campaign shipped was proven by removing what it guards and
  watching it go red. `NISI FRANGAS, NIHIL PROBAS.`

## ⛔ THE ROAD — builder, 2026-08-27. THE ORDER IS THE RULING.

> *"the first mass migration is homing everything.... then breaking everything up into crates...
> then killing `::` in keywords ... then making every call head a symbol ... then we'll have
> edn/clojure compliant syntax ... then we chase totality."*

```
1  HOME EVERYTHING          <- WE ARE HERE (arc 255)
2  break into crates
3  kill `::` in keywords
4  every call head a symbol
5  = EDN/Clojure-compliant syntax
6  chase totality
```

**Totality is LAST, and that is a ruling, not a backlog position.** A defect whose honest fix is
"make this total" gets RECORDED and left — see
`296/NOTE-the-doctest-runner-masks-every-failure-behind-one-raise.md`, where `:wat::core::=` raises
on 18 of 43 `Value` variants and the orchestrator offered to fix it mid-stone. The builder refused.
**Do not open a totality front out of step order**, however cheap the instance looks.

⚠ **AND "HOME" MEANS TWO THINGS** — conflated three times in one day before it was caught:
```
FILE-DOMAIN carve   loose root files -> src/<domain>/        HOME-5 edn · 6 load · 7 host  (all SHIPPED)
REGISTRY home       dispatch arms    -> src/intrinsic/<ns>/  HOME-8 holon · 9/10 math,stat,seq · 11 edn · 12 ast
```
**Only the REGISTRY kind advances step 1.** A file carve tidies the tree and takes NOTHING away from
`walk.rs:268`. Measure `src/intrinsic/<ns>` before calling a home built.

## ⬜ NEXT — measured, not guessed

- ⛔ **`apply`'s FOURTH DOOR IS STILL SHUT — 331 of 380 report "unknown function".** Stone O closed
  three of four; DOOR 2 is what remains and it is a SWEEP, not a design question — the machine is
  built and proven (O-iii), so each namespace is a migration commit.
  ```
  DOOR 1  defclause head              ✓ O-ii    22 production verbs — + - * / reduce sort into …
  DOOR 2  intrinsic, no value door    ⛔ 331     "unknown function" for verbs that plainly exist
  DOOR 3  intrinsic, value door       ✓ O-i     43 explicit + 6 generated = 49, all arity-guarded
  DOOR 4  plain fn / defn             ✓         was always correct
  ```
  **O-iv is drawn in the design and is two things:** migrate the remaining ~130 SHELL verbs (105 new
  doors + 25 two-fn collapses), one commit per namespace; and give `eval_apply` the honest word —
  consult `lookup_entry` before raising, so a registered-but-unreachable verb hears *"registered, but
  not reachable through apply"* instead of a lie. The honest-word half is independent of the sweep
  and true no matter how far the sweep gets.
  ⚠ **The two CALLING CONVENTIONS are forced by the language, not by us** — `apply`'s arguments have
  no syntax. Proven: `(apply :wat::i64::+ (:mk::pair))` → 42, and the form's AST children are
  `[apply, the verb, (:mk::pair)]` — no node for `20` or `22` exists anywhere. It is splat, and the
  arity is decided at runtime. Two conventions is right; **two REGISTRATIONS is not** — and after
  O-iii a verb that declares its algebra gets both doors generated, so it cannot be born with one.
- **`@Total` STILL HAS NO HOME**, and 255.3 cannot close without it. Unchanged; totality is **step 6**
  of the road and must not be opened early.
- **The doctest runner masks every failure behind ONE raise** — `wat/doctest.wat:67` guards both
  `eval-ast!` calls and NOT the `=` between them. THREE fixes attempted and each refuted by
  measurement; see `296/NOTE-the-doctest-runner-masks-every-failure-behind-one-raise.md`. Do not
  re-attempt without reading the three refutations.
- **`list_map_is_not_vector` is frozen by name** in `no_bare_is_err.rs`, blocked on a RULING not on
  work: arc 118.2a made `map` lazy so it preserves NO container, and the test claims container
  preservation. Retire it, or re-point it at `reverse`/`concat` — which changes what a test of that
  name tests.
- **The megafile is still the boss.** `runtime.rs` 34,252 · `check.rs` ~22,400. 168 arms remain, of
  which 36 are SPECIAL FORMS (a different contract — `#[wat_special_form]`, not a home) and ~21 are
  `rete::` (a coherent namespace with no home yet).


## ⛔ RULES THAT STILL COST TIME

- ⛔ **`git commit <paths>`. NEVER a pathless commit.** A scoped `add` does not save you.
- ⛔ **After committing, `git status` must be EMPTY** — that is what proves the commit IS the tree
  the floor measured. A floor proves the TREE; only this proves the COMMIT.
- ⛔ **`docs/arc/**` NEVER MOVES in a rename.** Half of every prose census is history.
- ⛔ **Riders: no worktrees, no `git stash` in ANY form, no sub-agents, everything FOREGROUND.**
  Ending a rider's turn ENDS it.
- ⛔ **The rete naming invariant**: `rete_name == core_name.replacen(":wat::", ":wat::rete::", 1)`,
  tested. It drags `RETE_MODULES` and `datamancer.rete.edn`'s baked ABI hash. **Add no
  `NAMING_RULE_EXCEPTIONS` entry to pass a gate**, and add no `RETE_MODULES` entry nothing forces.
- ⚠ **`.wat` scratch → `wat-scripts/scratch-pad/`** — the loader gate walks the DIRECTORY.

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** In one day: five censuses whose population was wrong while
> the pattern was right, two acceptance bars that could not measure what they claimed, a narrative
> about the project's testing posture built on the wrong runner, and a lesson I discovered myself and
> then dropped from the next two briefs. **Re-run the instrument that made the claim; do not read the
> claim.**
>
> ⚠ **AND THE COUNTERWEIGHT, or you will freeze:** seven stones landed in a day, every one green,
> every gate proven by breaking its door. The numerics, the collections and keyword are home. The
> defclause the builder described is on disk with all four arms pointing at homes. 250 names are
> registered where there were 6 in June.
>
> Read `294/REALIZATIONS.md` **R6 → R9 IN FULL** before writing R10 — not their middles.
>
> `DOLOR INDEX EST.` · `INCENDIMVS VT VIDEAMVS.` · `SCRIBIMVS VT EXVLET.` · `DERIVAMVS NE MENTIAMVR.`

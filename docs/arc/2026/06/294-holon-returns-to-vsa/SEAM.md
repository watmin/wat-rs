# SEAM — the ONE live breadcrumb. As of 2026-08-26. Arc 255: the homes campaign, seven stones in a day.

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
floor .......... 5059/5059, 0 FAIL, 19 skipped, ~90s   (scripts/floor.sh, exit read UNPIPED)
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
registered intrinsics ... 250        src/intrinsic/ homes ... 21        retirement rows ... 131
src/ root .rs files ..... 24         runtime.rs 40,492 + check.rs 22,469 = 75% of the root
```

**SHIPPED 2026-08-25/26 — seven stones, one method:**

```
A-i · A-ii   the numerics      :wat::core::{i64,f64}::*  ->  :wat::{i64,f64}::*
B-i · B-ii   the corpus        2,054 core sites + 408 rete sites, by codemod
C            the retirement    the old numeric spellings became check-time errors
D            bigint+rational   the numeric tower finished
E-i…E-iv     the collections   map · hashmap · vector · vec · hashset · linkedlist · keyword
F            the String verbs  the last per-type family — and the lint it fed was firing on a corpse
```

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

**1. A CENSUS OF A NAME MUST ASK EVERY RENDERING AND EVERY EXTENSION.** The pattern was right five
times while the POPULATION was wrong. `:wat::core::Vector/length` and `:wat.core/Vector/length` are
the same name; a census in one is blind to the other, and a golden holding the dotted form took the
floor down. Four extension misses too: `.wat.bad` (invisible to `git ls-files '*.wat'` BY EXTENSION),
`.jsonl`, `.edn`, `.wat.expr`. **And a migration census is not a registration census** — ask the
DISPATCH TABLE what exists, the CORPUS what moves.
`[[feedback_a_census_of_a_name_must_ask_every_rendering]]`

**2. RETIRING A NAME DISARMED ELEVEN NEGATIVE TESTS AND THE FLOOR STAYED GREEN.** Their fixtures used
the retired spelling in executable position; their tests asserted a bare `is_err()`. Each began
passing on the RETIREMENT error instead of the defect it existed to prove. **It does not rot loudly;
it rots green.** The pre-check now costs one command and has fired once more since.
`[[feedback_retiring_a_name_disarms_every_bare_is_err_test]]`

**3. A COMPLAINT FROM THE WRONG INSTRUMENT IS NOT A FINDING.** `cargo test` red, `cargo nextest`
green, same 247 tests. I escalated it into *"our floor cannot see a class of shared-state defect"* —
in a commit message — and was one message from a NOTE enshrining it. The contract was written in
prose in a file I had already read. **The floor is nextest. A red from `cargo test` is information
about `cargo test`.** This does NOT loosen red-is-a-red.
`[[feedback_a_pass_answers_only_the_question_the_instrument_asks]]`

**4. A LESSON LEARNED AND THEN DROPPED costs the same as one never learned.** I found the rete
naming invariant in B-ii, wrote a brief around it, and omitted it from E-i AND E-ii. **Read the
previous stone's COMMIT MESSAGE before writing the next brief** — that is where the surprises live,
and briefs are otherwise written from the design and a fresh census.
`[[feedback_a_lesson_learned_and_then_dropped]]`

**5. A GATE PINNED TO A MAGNITUDE IS AT WAR WITH A CAMPAIGN THAT MOVES IT.** Both directions in one
stone: an arm-count floor that trips as the carve SHRINKS the match (reshaped to a positive control),
and a per-row gate that timed out as the carve GREW its table 34→126 rows. `[[feedback_a_gate_freezes_names_never_a_count]]`

**6. TWO OF MY ACCEPTANCE BARS COULD NOT MEASURE WHAT THEY CLAIMED.** One counted a STRING where the
question was a MECHANISM. One was **UNSATISFIABLE AT BASELINE** — `wat --check wat/core.wat` fails
by construction (`include_str!` + Stdlib privilege; as an entry file it parses twice and collides
with itself). **A gate that cannot pass is the mirror of one that cannot fail.** I never ran it.

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

## ⬜ NEXT — measured, not guessed

- **`@Total` HAS NO HOME, AND 255.3 CANNOT CLOSE WITHOUT IT.** `@Purity` (290) and `@Determinism`
  (282) are declared at the registration site; **`@Total` does not exist**, so totality lives only in
  a hand-curated list. 255's LOCKED baseline is `name·arity·kind·pure·deterministic·
  expand_time_legal·defined_in·layer` — no `total` — while slice **255.3** already commits to
  deleting `rete/purity` and `is_pure_total`. The two designs never met (255 locked 2026-06-21; the
  four-axis fence is 278's, invented after). **Adding the field to a LOCKED model is the builder's
  ruling.** `NOTE-the-registry-asserts-properties-nothing-verifies.md`
- **⛔ AN OPEN RULING: REGISTRY ROUTING DROPS PRODUCER PROVENANCE.** `#[wat_intrinsic]`'s handler
  signature has no provenance slot, so routing a PRODUCER through the registry downgrades
  `RuntimeBuilt{producer, call-span}` to `SymbolBound`. Four keyword verbs lost producer attribution
  in E-iv, and arc 233's own guards were rewritten to match — disclosed honestly, but **a guard
  rewritten to match degraded behaviour is a green test that no longer proves what it was built to
  prove.** STRUCTURAL: every future home carve routing a producer does this.
- **THE REMAINING per-type family**: `String/*` — **373 sites, and `String/concat` is a LIVE
  DUPLICATE** of a registered `:wat::string::concat` (verified: both return `true` on the same
  input). The string carve moved the lowercase family and left the capital one behind.
- **The collections' backend swap** — "probably a week or two". The homes are carved so it costs ONE
  prefix rename on the marked families only; the persistent ones never move again.
- **`wat --check` ACCEPTS AN UNREGISTERED SCALAR TYPE** (`:wat::core::NotARealType` passes). Gate
  banked at `tests/types/probe_diag_typealias_leniency.rs:16`.
- **A gate that proves a file LOADS does not prove it RUNS** — two instances: the console-demo built
  green and died at startup; a scratch probe loaded green and died at runtime. Nothing runs
  `examples/*/wat` or `wat-scripts/scratch-pad/` mains.
- **The megafile carve** — `check.rs` 22,469 and `types.rs`/`freeze.rs` have `check/`, `types/`,
  `freeze/` dirs holding three small files each. A different defect from a missing home.

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

# DESIGN — STONE M2: the corpus takes the map ctor (the OTHER HALF of M)

## ⛔ THIS IS NOT A FOLLOW-UP STONE. IT IS THE SECOND HALF OF ONE STONE.

M made positional variant construction **illegal**. The corpus is written in it. So the tree is not
merely red — **it is non-functional**: `./target/release/wat <any file>` exits **3**, because the
stdlib itself no longer loads. M was drawn as "build the form, migrate later"; that framing died the
moment the refusal reddened the loaded world. Section 7's atomic-commit pattern governs — **M is
committed LOCALLY as a save point (`c8ef970fb`, unpushed) and M2 lands on top; the pair pushes when
the floor is green.**

```
floor at M    5238 run: 2447 passed, 2773 FAILED, 18 TIMED OUT     FLOOR EXIT=100
control run   EXIT=3    ← no wat program starts
one cause     every :reason is "positional variant construction is retired"
the timeouts  14 wat_mcp + 2 sigterm + 2 process-label — tests that SPAWN a wat process whose
              child now dies at startup. NOT a second defect.
```

## WHAT IT DELIVERS

Every positional variant construction in the corpus becomes a map construction naming its declared
fields, in declaration order:

```
(:wat::kernel::RecvOutcome::Message msg)        ->  (:wat::kernel::RecvOutcome::Message {:msg msg})
(:wat::service::Outcome::NoReply s)             ->  (:wat::service::Outcome::NoReply {:state s})
(:probe::Box::Empty)                            ->  (:probe::Box::Empty {})
```

## ⛔⛔ THE CONTRACT DECISION, AND IT IS THE WHOLE STONE: **THE CODEMOD MUST ASK, NOT OBSERVE**

**MEASURED, 2026-09-07 — the 60 distinct enums named in the positional-ctor errors:**

```
28  hand-written `defenum` somewhere in source
32  ⛔ GENERATED — no literal `defenum` ANYWHERE (defservice / defsurface output)
    wat::cache::Cache::Op · Cache::Reply · hologram-svc::Admin · hologram-svc::Status
    lru-svc::Admin · lru-svc::Status · query::Store::Reply · telemetry::Journal::Reply · …
```

★ **A byte-observing codemod goes blind on 32 of 60.** `wat-scripts/fixes/positional-to-kwargs.wat`
— which Stone M's own DESIGN wrongly named as the shape to copy — is explicitly *reflection-free*:
it observes `defrecord`/`defstruct`/`defholon` forms AS BYTES to build a type→field map. Its
`def-head?` does not mention `defenum` at all. Point it at enums and it migrates the 28 it can see,
reports success, and **leaves the build broken with hundreds of sites silently skipped.**

**THE PRECEDENT IS `wat-scripts/fixes/match-arm-to-bracket-map-pattern.wat`** — the codemod RELAND 4
taught to ASK `type-of` rather than guess, after `{:_cur _cur}` turned out to be `{:cursor _cur}`,
an abbreviation no heuristic derives. **Same lesson, one stone later: a codemod that reads field
names out of source text cannot see what a macro generated.**
`[[feedback_a_wrong_name_does_not_fail_it_names_something_else]]`

The reflection path is PROVEN on the pre-M binary — RELAND 4's SCORE measured `eval-with-defs!` of
declaration forms only, with `type-of` answering a `defservice`-generated response type's fields in
declaration order (`err` / `bytes` / `cap` / `cursor` / `path`).

## ⛔ THE METHOD IS THE DANCE — AND IT IS BETTER THAN THE RECORDED ONE

The codemod is itself a `.wat` program **and no `.wat` program can run**. `wat/fix.wat:23` documents
that chicken/egg (its header records that a prior self abandoned the tool because the dance was not
written down). Its step 1 is `git stash push src/…` — **which no longer applies, because M is
COMMITTED, not dirty.** The replacement is strictly safer:

```bash
1.  git checkout HEAD~1 -- src/check.rs src/runtime.rs src/types.rs src/record/construct.rs
2.  cargo build --release            # OLD checker (accepts positional) + any new :wat::fix:: verb
3.  <dry-run on /tmp, diff> then the real run over EVERY path in WORKLIST-296-M2-paths.edn
4.  git checkout HEAD -- src/check.rs src/runtime.rs src/types.rs src/record/construct.rs
5.  cargo build --release
```

★ **No stash to drop.** Restore-from-commit is idempotent and re-runnable; a dropped stash is
unrecoverable. The builder's rule — *"we can commit, just don't push a broken main… the commits
allow save undos without losing work"* — is what makes this available, and it removes the hazard the
recorded dance had to warn about.

`type-of` is available on the pre-M tree: `wat/runtime-typeinfo.wat` exists at `HEAD~1` (296 L
committed at `480f38d05`). **Verified, not assumed.**

## THE WORKLIST — MEASURED, NOT ESTIMATED

```
288 files · 4,933 sites      WORKLIST-296-M2-positional-ctor-sites.txt  (counts + paths)
                             WORKLIST-296-M2-paths.edn                  (the driver's vector)
```

Not the 659 (that was `wat/*.wat` only) and not the DESIGN's 2,209 (that was Option/Result only).
Each count is FIXTURE-LOCAL: errors whose `:file` is the file being checked, excluding the 659
stdlib errors every `--check` reports. Heaviest are service probes, not the stdlib:
`probe_arc170_c2_strike1_mixed.wat` 203 · `probe_arc170_c2_mixed_macro.wat` 203 ·
`probe_arc278_journal_surface.wat` 115 · `wat/fmt.wat` 69 · **`wat/fix.wat` 67 — the codemod's own
framework rewrites its own source** (safe: the executing copy is frozen into the binary; the
rebuild at step 5 picks up the new one).

## ⛔ THE ACCEPTANCE BAR IS FIXTURE-LOCAL ERRORS, NEVER AN EXIT CODE

M's probe rows are **not trustworthy while the world is red**, and the peer caught it: every bar was
`assert_eq!(check(subject), check(control))`, and the refusal reddened the CONTROL, so three rows
passed on `1 == 1`. M's EXPECTATIONS also carried a contradiction — row 1 (`control EXIT=0`) and
STOP-5 (do not migrate the corpus) could not both hold.

**M2 fixes this in the same motion it fixes the corpus**: the probe's `check()` becomes a count of
errors whose `:file` is the fixture, and the migration's own done-when is that same count reaching
**0** corpus-wide. One change, both jobs.
`[[feedback_a_green_from_a_mis_aimed_probe_is_indistinguishable_from_a_working_gate]]`

## OUT OF SCOPE — AFFIRMATIVELY CUT

- **The Option/Result bare-name retirement** (`:wat::core::None` → `:wat::core::Option::None`, the
  four `builtin_variant` arms). Its own stone, next. M2 migrates the CONSTRUCTION GRAMMAR only; a
  site keeps whatever variant spelling it already has.
- **`{:keys}` on `defrecord`** and **`variant <: enum`.** Ruled by the builder as their own stones.
- **wat-fmt / whitespace.** The codemod's edits are span-faithful token inserts; trailing whitespace
  is wat-fmt's job, not this stone's.

## THE FOUR QUESTIONS

- **Obvious?** YES. Every construction names its fields, exactly as its pattern and its wire already do.
- **Simple?** YES. One rewrite rule, applied by the tool that owns corpus rewrites.
- **Honest?** YES. The tool ASKS the declaration instead of guessing from text — which is the only
  way the 32 generated enums are reachable at all.
- **Good UX?** YES. It ends with the toolchain running again, which is the only state that matters.

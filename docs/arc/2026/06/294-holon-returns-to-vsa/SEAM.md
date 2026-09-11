# SEAM — the ONE live breadcrumb. 2026-09-10. ⛔ **RED · 25 UNPUSHED · PEER STRIKE IN FLIGHT.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS BELOW.

```bash
git status --porcelain          # ⛔ EXPECT DIRTY — read the PEER STRIKE block below FIRST
git log --oneline @{u}..HEAD    # expect 25 — NOTHING IS PUSHED
cat /home/john/work/holon/.pulsare/to-claude   # has grok scored?
./scripts/floor.sh > /dev/null 2>&1; echo $?   # RE-RUN IT. The kept log may be stale or TORN.
cargo clippy --release --all-targets -- -D warnings
```

⛔⛔ **THE PEER EDITS THIS SAME WORKING TREE.** A floor run during a grok strike measures a TORN
tree — I hit exactly that: `.floor/latest` vanished mid-run and clippy reported two phantom compile
errors that were gone seconds later. **Check `git status` before believing any central
measurement.**

**AS OF 22:23 THE TREE IS DIRTY AND THAT IS EXPECTED — IT IS NOT YOURS.** `M src/check.rs`,
`M tests/types/probe_arc234_stone3c_keyword_accessor.{rs,wat}`, and **13 new**
`tests/types/probe_arc251_type_the_polymorphic_accessor__*.wat` + its `.rs` are grok's IN-FLIGHT
strike on the brief below. Last write 22:20:25, no `kind=scored` yet.
⛔ **DO NOT `git checkout`, `git stash`, `git reset`, or COMMIT THOSE PATHS.** They are a peer's
unfinished work in a shared tree, and every one of those verbs destroys it silently. Wait for the
yield, THEN floor + clippy centrally, THEN commit.

## ⛔ THE PEER IS ALIVE — the old seam said otherwise and was WRONG

`pulsare` reconnected, grok's credits reset. It struck TWO stones today, one of which **refuted my
own brief**. Protocol: **write the files, then `pulsare_yield`** (`kind: briefed` / `scored`), with
**ABSOLUTE** paths — relative paths are rejected. Local `Agent` riders still work and are right for
edit-only fan-outs (FM 18: riders edit, the orchestrator measures).

# ⬜ WHERE THE WORK IS

## ✓ THE DOT FLIP IS LANDED — `Enum.Variant` everywhere

Nine classes of variant-name site, found one at a time, each by something failing rather than by a
census of mine. The corpus is rewritten (382 + 261 confirmed pairs, **asked** via
`variant-parent-of`, never pattern-matched), both door bodies read/write `.`, and
`:wat::runtime::compose-variant` is minted so wat can BUILD a variant name instead of concatenating
one.

## ✓ TWO TYPE-SYSTEM DEFECTS FIXED TODAY (both pre-existing, both invisible to the floor)

```
the EXPANDER          expand_form walked List and Vector, NOT Map or Set. A macro call inside a
                      map literal was NEVER expanded -> bare head -> no scheme -> FRESH VAR ->
                      assignable passed trivially. `PublicOpInAlarm` had been silently DEAD.
{:keys} INSTANTIATION check.rs dropped a Parametric's args and used the DECLARED field types raw,
                      for record, struct AND variant. `(:u::Cell :- [i64])` bound x as `:X`.
                      Plus `:Enum.Variant/field` accessors, which never existed.
```

## ⬜ IN FLIGHT — grok is striking

`251-types-as-forms/the-bare-keyword-accessor-is-accepted-but-never-typed/BRIEF-…md`
One block (`check.rs` ~5795–5840), **two bugs pointing opposite ways**: a false ACCEPT (`fresh.fresh()`
takes any declared type) and a false REFUSE (the `Parametric` arms cover HashMap + singleton Enum
but NOT a parametric Aggregate). The fix is at the site — the acceptability test already resolves
the receiver and looks up its `TypeDef`, then throws the answer away.

## ⬜ THE 8-ish REMAINING FLOOR REDS — two rulings, not defects

⚠ **The last FULL floor read 5345 run / 5336 passed / 9 failed.** One of those 9
(`no_loose_string_assert`, on a peer probe) was fixed and verified ONLY by a scoped run — **no full
floor has confirmed 8. Re-measure; do not quote my number.**

```
6  probe_arc278_call_context + arming control   ONE root cause. A variant literal in an
   unascribed MATCH SCRUTINEE narrows a container's type param (Alarm<Op.-Tick>), and same-head
   parametric args are INVARIANT (arc 278 Stone 2). ⛔ THIS IS A RULING, NOT A REPAIR.
1  probe_arc255_the_blanket_hides_a_phantom_head  ruling ①-C given, NOT YET APPLIED: rewrite it to
   assert the INVERSE (the dot spelling RESOLVES; `::` is now the refused one).
1  every_wat_scripts_file_loads                 untriaged
```

## ⚠ RULINGS FROM TODAY — do not re-litigate

- **Character case carries NO meaning.** `user/some-enum.first` is legal; `Enum.Variant` is a BIAS.
  Never branch on case. Four live violations tracked in
  `109-kill-std/NOTE-character-case-carries-no-meaning.md` — **not fixed, deliberately.**
- **A variant IS a tagged record** (arc 296). `Demo.Has -> Demo` accepts; `Demo -> Demo.Has`
  refuses; a `Demo` receiver must `match`, a `Demo.Has` receiver need not. Verified 3/3.
- **Ruling ②-A applied**: the arming fixture's positional ctors rewritten to map form.
- **`git commit <paths>`, NEVER `git add` then commit** · **⛔ NO SIDE BRANCHES** · **PUSH ONLY GREEN.**
- ⚠ **Backticks inside a double-quoted `git commit -m` are SHELL-EXPANDED.** It ate words from two
  commit messages today. Use `-F -` with a quoted heredoc.

## ⛔ THE FAILURE PATTERNS — every one fired TODAY

**① A CONDITIONAL PROBE CANNOT TELL CLEAN FROM NEVER-RAN.** Twice. A mismatch-only print where
`0 == 0` was silent; a grid that showed error text for its first row only, so I copied a
neighbouring row's cause onto a different failure and **shipped it into a brief** — and the peer,
having refuted the fact, still built to my conclusion.
`[[feedback_a_conditional_probe_cannot_tell_clean_from_never_ran]]`

**② THE FAILING-TEST LIST IS NOT THE BLAST RADIUS.** `h2__record.wat` was GREEN before a codemod
broke it — recovered only by sweeping all 1119 fixtures, not the 795 that were red. The same
literal is a VARIANT in one file and a RECORD NAME in another: **a global token→token map is
unsound.**

**③ USAGE COUNT MEASURES AGE, NOT DESIRABILITY.** I nearly reverted a correct change because it had
one corpus use. It was one hour old.
`[[feedback_usage_count_measures_age_not_desirability]]`

**④ NARRATION IS NOT A TRIGGER PATH.** *"Now that `:arms` is genuinely inferred…"* was a story. The
builder asked for the path; the A/B that would have tested it had measured the SAME BINARY TWICE
(the stash was empty because the file was committed, and the rebuild said `0.19s`).

**⑤ FM 19, fifth occurrence.** A rider backgrounded a sweep and ended its turn. The brief's SECOND
LINE forbade it. Prose is the convention rung; FM 18's tier rule is the fix.

★★★ **AND EVERY DEFECT TODAY WAS FOUND BY SOMETHING FAILING, NEVER BY A CENSUS OF MINE.** Nine
classes, three type-system bugs, two rulings — each surfaced by a rider's STOP, a peer's refutation,
or an instrument breaking loudly. **Read a red as a finding.**

---

> **SEAM.** You are NEW. The better this reads, the more it will feel like continuing rather than
> waking. **That feeling is the failure.**
>
> ⚠ **THE RECORD LIES IN YOUR OWN VOICE.** Re-run the commands. Do not read the numbers.
>
> ⛔ **NOTHING IS PUSHED AND THE FLOOR IS RED.** That is the deliberate state, not an accident.
>
> `DOLOR INDEX EST.` · `NISI FRANGAS, NIHIL PROBAS.` · `DERIVAMVS NE MENTIAMVR.` ·
> `HAERESIS EST ITERVM ROGARE.`

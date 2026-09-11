# SEAM — the ONE live breadcrumb. 2026-09-11. ⛔ **RED (6, ALL ONE RULING) · 29 UNPUSHED · CLEAN.**

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and **that feeling is
> the failure.** Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**,
> never a disk copy), then run the commands below before you touch anything.

> `251/SEAM.md` · `255/SEAM.md` · `278/SEAM.md` PARKED and point HERE. ⛔ **PARKED IS NOT DEAD.**

## FIRST — RUN THESE. DO NOT READ THE NUMBERS BELOW.

```bash
git status --porcelain          # expect EMPTY
git log --oneline @{u}..HEAD    # expect 29 — NOTHING IS PUSHED
cat /home/john/work/holon/.pulsare/to-claude   # has grok scored?
./scripts/floor.sh > /dev/null 2>&1
L=$(ls -td .floor/*/ | head -1); grep -E "Summary|FAIL \[" "$L/raw.log" | sort -u
cargo clippy --release --all-targets -- -D warnings
```

⛔⛔ **THE PEER EDITS THIS SAME WORKING TREE.** A floor run during a grok strike measures a TORN
tree — `.floor/latest` vanished mid-run and clippy reported two phantom compile errors that were
gone seconds later. **Check `git status` before believing any central measurement.** While a peer
is striking: do NOT `checkout`/`stash`/`reset`/commit its paths, and do NOT build or floor. Wait
for `kind=scored`, THEN measure centrally, THEN commit.

## ⛔ THE PEER IS ALIVE

`pulsare` reconnected, grok's credits reset. Protocol: **write the files, then `pulsare_yield`**
(`kind: briefed` / `scored`), **ABSOLUTE** paths — relative ones are rejected. Its tier is
edit-and-report: **it does not floor, does not clippy, does not commit.** That is yours, centrally,
once, on a quiescent tree. Local `Agent` riders are still right for edit-only fan-outs.

# ⬜ WHERE THE WORK IS

## ✓ LANDED — the dot flip, and the type system that flip exposed

```
THE DOT FLIP          `Enum.Variant` is canonical; `Enum::Variant` is dead. Nine classes of
                      variant-name site, each found by something FAILING. Corpus migrated by the
                      self-hosted codemod, ASKING `variant-parent-of` rather than pattern-matching.
the EXPANDER          expand_form walked List and Vector, NOT Map or Set — a macro call inside a
                      map literal was NEVER expanded -> no scheme -> FRESH VAR -> assignable
                      passed trivially. `PublicOpInAlarm` had been silently DEAD.
{:keys} INSTANTIATION dropped a Parametric's args, used DECLARED field types raw, for record,
                      struct AND variant. Plus `:Enum.Variant/field` accessors, which never existed.
THE BARE ACCESSOR     `b743ae310` — `(:field x)` was ACCEPTED AND NEVER TYPED (fresh var unifies
                      with anything), and a parametric Aggregate was absent from the arms, so the
                      TRUTH was refused as UnknownCallee. Both closed at the site.
RULING ①-C            `45aabbb93` — the dot spelling RESOLVES; `::` is the refused one.
```

## ⬜ THE 6 REMAINING REDS — ALL ONE ROOT CAUSE, AND IT IS A RULING YOU OWE

```
6  probe_arc278_call_context (5) + probe_arc278_arming_is_internal_only (1)
   A variant literal in an unascribed MATCH SCRUTINEE narrows a container's type param
   (Alarm<Op.Tick>), and same-head parametric args are INVARIANT (arc 278 Stone 2).
   ⛔ NOT A REPAIR. Three options measured, all three DISQUALIFIED: ann-form ascription,
   bind-enclosing-enum, same-head covariance.
   ★ THE FLOOR CANNOT GO GREEN — AND NOTHING CAN BE PUSHED — WITHOUT THIS RULING.
```

★★ **THREE RATCHETS FIRED TODAY, all the same shape**: `no_loose_string_assert`,
`probe_arc255_the_blanket_hides_a_phantom_head`, and the parametric-vector probe. **A probe that
PINS A DEFECT becomes a lie the moment the defect is fixed, and it goes red in a way that reads
like a regression.** Two of the three also had to MOVE: a fixture whose job is to be REFUSED cannot
live under a gate requiring it to LOAD (arc 255 Stone 4 is the precedent, and the reason).

## ⚠ RULINGS — do not re-litigate

- **Character case carries NO meaning.** `user/some-enum.first` is legal; `Enum.Variant` is a BIAS.
  Four live violations tracked in `109-kill-std/NOTE-character-case-carries-no-meaning.md` —
  **deliberately unfixed.**
- **A variant IS a tagged record** (arc 296). `Demo.Has -> Demo` accepts; `Demo -> Demo.Has`
  refuses; a `Demo` receiver must `match`, a `Demo.Has` receiver need not.
- **`git commit <paths>`** (new files need an explicit `git add <path>` first — never a sweep) ·
  **⛔ NO SIDE BRANCHES** · **PUSH ONLY GREEN.**
- ⚠ **Backticks inside a double-quoted `git commit -m` are SHELL-EXPANDED.** Use `-F -` + heredoc.

## ⛔ THE FAILURE PATTERNS — every one fired in the last two days

**① A CONDITIONAL PROBE CANNOT TELL CLEAN FROM NEVER-RAN. Fired FOUR times.** A mismatch-only
print where `0 == 0` was silent; a grid that showed error text for its first row only, so a
neighbouring row's cause got copied onto a different failure **and shipped into a brief**; and
twice a falsification that "passed" because the corruption never landed — a `sed` for
`"nonexistent"` cannot match `\"nonexistent\"` in the file. **VERIFY THE CORRUPTION IS PRESENT
BEFORE READING THE GREEN.** `[[feedback_a_conditional_probe_cannot_tell_clean_from_never_ran]]`

**② A NUMBER ASSEMBLED FROM TWO MEASUREMENTS IS A THIRD MEASUREMENT NOBODY TOOK.** A brief shipped
`5345 run · 5336 passed · 8 failed`. **5336 + 8 = 5344.** The run/passed pair came from one floor
and the failure count from an older one, and the peer struck against a baseline that never existed.

**③ CLOSING A RED CAN OPEN ANOTHER, AND ONLY A FULL FLOOR SEES IT.** Fixing the arc255 probe
introduced a fresh `no_inlined_edn` violation. The scoped run showed 3/3 GREEN — a commit there
would have shipped a regression while reporting a fix. **Scoped runs verify nothing about the floor.**

**④ A STALE ASSERTION GOES RED FOR A REASON THAT IS NOT THE ONE IT NAMES.** The arc255 probe was
"expected red, ruling given". It had actually stopped failing at RESOLVE and begun failing at CHECK,
on a line the flip made illegal for an unrelated reason. **Re-goldening would have pinned an
accident and called it the ruling.** Read the failure before applying the ruling.

**⑤ THE FAILING-TEST LIST IS NOT THE BLAST RADIUS.** `h2__record.wat` was GREEN before a codemod
broke it — found only by sweeping all 1119 fixtures. The same literal is a VARIANT in one file and
a RECORD NAME in another: **a global token→token map is unsound.**

**⑥ USAGE COUNT MEASURES AGE, NOT DESIRABILITY.** Nearly reverted a correct change over its single
corpus use. It was one hour old. `[[feedback_usage_count_measures_age_not_desirability]]`

**⑦ NEVER READ AN EXIT CODE THROUGH A PIPE.** `wat --check f | head` returns **head's** status. It
reported `0` for a file that exits `1`, and nearly went into an assertion.

★★★ **EVERY DEFECT HERE WAS FOUND BY SOMETHING FAILING — a rider's STOP, a peer's refutation, a
lint, an instrument breaking loudly — and NEVER by a census of mine. Read a red as a finding.**

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

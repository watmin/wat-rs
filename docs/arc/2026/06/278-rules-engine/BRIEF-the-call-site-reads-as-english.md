# BRIEF — the call site reads as English

Executor: grok. Anchor at `/home/john/work/holon/wat-rs`; `pwd` first. Branch `sns-sqs`, HEAD
`b8a868fca`, tree clean. Read `DESIGN-the-call-site-reads-as-english.md` first.

## THE WORK

Two renames, both codemods. `Alarm`'s field `after` → `delay`, and the seven `:wat::time::` unit
constructors to their plurals. **Nothing is broken; this closes no defect.** The test is whether one
call site reads without its docstring.

## ROOMS

1. **`wat/fix.wat`** — the **BOOTSTRAP / STASH-DANCE** header. `wat/service.wat` is stdlib. R1
   needed this and did not reach it; you will.
2. **`wat/service.wat:67`** — `(defrecord :wat::service::Alarm :- [O] [after <- :wat::time::NonZeroDuration  op <- :O])`.
3. **`wat/service.wat:52`** — the docstring that currently does the field's work: *"a handler
   schedules a self-message by emitting `Alarm`s: `after` a Duration, deliver `op`."* After the
   rename that sentence should be re-read; if it still explains something the name now says, trim it.
4. **`wat/service.wat:1618-1622`** — ⭐ **the stutter**: `(:wat::kernel::after <peer-kind>
   (:wat::service::Alarm/after ~sym) (:wat::service::Alarm/op ~sym))`. Two `after`s, different parts
   of speech, one form. This is the reading that proves the rename.
5. **`src/intrinsic/time.rs:342-513`** — `unit_constructor` and the seven public functions.
6. **`src/check.rs:20792`** — where the seven register; **`src/rete/purity.rs:2330-2348`** — their
   purity rows. Both were touched by the `NonZeroDuration` stone; this is the same block.
7. **`src/value/value.rs:304,313`** — the recorded precedent: `Char`→`char` (Stone 242.1),
   `Rational`→`rational` (Stone C1, *"see the `char` precedent"*). Follow that form.
8. **`wat-scripts/fixes/vocabulary-stops-mumbling.wat`** — the freshest recorded codemod.

## STOP TRIGGERS

1. **`:after` turns out not to be `Alarm`'s at some sites.** It is a bare keyword. The finder must
   match the **form**, not the token — that error has cost this campaign eight times. If the finder
   cannot distinguish them, STOP and report.
2. **You are about to touch anything else in `wat/service.wat`.** R1's seam is a separate stone with
   a known-wrong first draft. STOP.
3. **The floor moves off `5214/5214`.** Every service expands through the `Alarm` record. Capture
   whole, do not re-run.
4. **You are about to skip `Microsecond`** because it has zero call sites. Rename it for symmetry —
   seven siblings that disagree is worse than one unused plural.
5. **A constructor's purity or determinism row changes.** They stay Pure + Deterministic.

## HOW TO WORK

Foreground everything. Floor is `scripts/floor.sh`; **read the Summary line, never a piped exit
code.** On an unintended red: **do NOT re-run**, capture the whole block verbatim, name the arm.

⚠ **Do not write `(:wat::core::None <Type>)`** — phantom form. See
`docs/arc/2026/04/109-kill-std/NOTE-none-is-not-a-function.md`.

⚠ S24 is live: `refused_subscriber_is_retried_not_dropped` can fail with `after-drain=got`.

Leave your work uncommitted. Prior comparable: `SCORE-the-vocabulary-stops-mumbling.md`.

## REPORT

- **one call site quoted, before and after** — that is the contract decision, and it is read, not measured
- `service.wat:1618-1622` after the rename
- both finders' census counts, before applying, against my hypothesis
- both codemods re-run: 0 changes
- the floor Summary line; the circuit, five runs
- every STOP that fired
- **the honest deltas.** My numbers here are a hypothesis; yours are the fact.

# AMEND — STONE 255.82: a bare `def` was never a form

**Drawn 2026-10-02.** **Executor: grok via pulsare, working solo** (it runs the floor). Continues at `25b7ee38f` (grok's
STOP-1). Commit locally on `main`; **do not push**.

## The ruling on STOP-1

Grok's census is accepted as measured: 140 bare heads are locals (32 names), and 28 are unbound. Of the 28, 4 are the
probe (its fourth head, `wat.core.Option.Some`, is the orchestrator's own mis-spelling of a variant constructor: the
probe should spell what it means) and **24 are the bare head `def`** in three scratch files.

**A bare `def` is not a form and never was.** `git log -S` shows those `(def :name expr)` lines were written that way at
birth (`87311ecf3`, 2026-08-04, arc 278 #57 round 1a); they passed `--check` only because a slash-less head was silently
accepted, and they never ran (no `:user::main`, `MainSignatureError` rc 4). The builder's rulings decide the spelling:
call heads become namespaced symbols (`wat.core/def`, as the converted corpus writes `wat.core/defn`), and there is
exactly one way to do a thing (2026-09-23), so a bare `def` beside `:wat::core::def`/`wat.core/def` would be a second
spelling. The 24 are one class: a head that names nothing. That is the cure the brief allowed ("a typo or a dead name"),
not a design question, so the STOP-1 cap does not bind them.

## The work

1. **The three scratch files:** respell `(def` to the declaration form the file's own surrounding code uses
   (`:wat::core::def`, which the door accepts), by a recorded codemod rule or a one-file edit each (three named files:
   say which). Then each file's claim (*"proving the new spellings resolve"*) is checked by `--check` with the head no
   longer silent. If a file's claim no longer holds once `def` is real, say so; a scratch file that proves nothing is
   deleted, with the reason.
2. **The probe:** its fourth head becomes the spelling it means (`wat.core.Option/Some` or the variant's registered
   head), so the probe's three intended heads are the only refusals.
3. **The refusal** as the brief describes, through `is_resolvable_call_head` (the one door), for a call head that is a
   symbol with no `/`, not a local in `locals`, and not accepted by the door. Measure first which rung makes the door
   accept `Some`/`Ok`/`Err` (they must not become legal heads by accident; if the `is_retired` rung is what admits them,
   the refusal must still name their retirement). `UnresolvedReference` has no remedy field: for the wrong join, either
   give the refusal a remedy naming the slash spelling or use the error kind that carries remedies; say which.
4. **Embedded wat:** run the same census over embedded literals (the `wat::embedded_wat` extractor) and list any unbound
   bare head there.
5. The brief's tests and gates, and the floor.

## STOPs

The brief's STOP-1 (now: more than 20 unbound sites **outside** the three files and the probe) and STOP-2 stand. A STOP
means STOP. Append to the SCORE, commit, **do not push**.

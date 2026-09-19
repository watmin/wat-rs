# No stdlib body names a user-declarable name

**Tier B step B.** Excursus `001-sns-sqs`. Predecessors, read them first:

- `../tier-b-the-check-skips-what-it-already-proved/` — Tier B's first refutation
- `../can-a-user-def-change-a-stdlib-verdict/FINDING.md` — the root
- `../a-mention-in-a-quoted-form-is-not-a-call/` — the repair that inverted the witness
- `../every-sweep-names-what-it-reads/SCORE.md` — step A, which scoped step C

## The sentence

> **A stdlib function body may name, in evaluated position, only `:wat::` things.**

That is the completion of a ruling already made. The namespace stone fixed where the stdlib
**declares** (`:wat::repl::*`, "we must only vend `:wat::*`"). This one fixes what a stdlib body
**mentions**. Declaring and mentioning are different channels and only the first was closed.

## Why now, and why it stands alone

⭐ **A justification on the record has expired.** `tests/function/probe_tier_b_stdlib_verdict_is_not_bake_fixed.rs:215`
documents its surface walk as:

> *"Over-approximating on purpose: a name used as DATA counts too, because `walk_for_restricted_call`
> does not know the difference either."*

**`walk_for_restricted_call` now knows the difference.** Since the quoted-mention stone it routes
through `resolve::boundary::quote_boundary` (`src/check.rs:1701`) and does not fire on a leaf in
quoted data. The walk at `:217` is calibrated to a walker that no longer exists, so the pinned
surface of **two** (`:user::main`, `:user::spawn::service-locus`, asserted at `:142`) is an answer
to a question the substrate stopped asking. It is not wrong — it is **stale**, which is worse,
because it reads as current.

⛔ **And that same file wrote a demand against itself that nobody has discharged:**

> *"the inverted witness no longer proves the `check:restricted-call(ALL fns)` sweep RUNS over
> stdlib bodies. It passed before because the sweep fired; it passes now because nothing is left
> for it to fire on. **A future Tier B must derive a NEW control for that sweep** — this one has
> become a control for the repair, not for the sweep, and reading it as the latter is how an
> elision would ship unnoticed."*

You are that future Tier B. **Step C cannot be graded without this control**, because a sweep with
no live control can be elided and every test stays green — which is precisely the failure the
previous stone predicted in writing.

This stone's value does not depend on C landing. If C is abandoned tomorrow, the stale
justification is still stale and the missing control is still missing.

## What is true today (verified 2026-09-18, re-derive rather than trust)

- The 8c sweep: `src/check.rs:750–756`, phase `P_CHECK_RESTRICTED` (`src/freeze/census.rs:247`),
  **3.07 ms** — it walks `sym.functions_iter()`, i.e. the whole stdlib.
- The walker: `walk_for_restricted_call`, `src/check.rs:1670`. Boundary handling at `:1701`;
  the quasiquote-template descent is `walk_restricted_quasiquote_template`.
- The classifier: `quote_boundary`, `src/resolve/boundary.rs:79` — **six** outcomes
  (`AllData`, `Quasiquote`, `MatchesSubject`, `Match`, `MakeRule`, `Ordinary`), and only the
  first two are data regions for this purpose. `is_unquote_escape` at `:102`.
- `:restricted-to` in the wat corpus: **one** site, `wat/spawn.wat:380`. Plus five
  `#[restricted_to]` in Rust (`src/io.rs:1343`, `:1383`, `src/runtime.rs:26001`,
  `src/kernel/spawn.rs:500`, `:586`). ⚠ Re-count both — this is a reading, not a gate.

⚠ **I did not census the evaluated-position surface, deliberately.** A `grep` over `wat/*.wat`
returns ~270 non-`:wat::` FQDN-shaped tokens, but it counts **comments** (`:myapp::`, `:<fqdn>::`,
`:my::` are doc prose) and quoted data alike. That number is a **ceiling with no lower bound** and
must not be quoted as a census. **The AST is the only instrument that can answer this.** Taking
that measurement is the first deliverable, not an assumption of it.

## The work

### 1. Census the surface by CHANNEL, position-aware

Split the existing one-number surface into the channels a name can arrive by, because they have
different closure properties and a merged number hides that (step A's `8d` lesson):

| channel | position-exempt? |
|---|---|
| the function's own path | never — always live |
| a name in its signature (`param_types` / `ret_type` / `rest_param_type`) | never — always live |
| a body leaf in **evaluated** position | — |
| a body leaf inside **quoted data** | exempt, by `quote_boundary` |

Report all four counts with the names. **The evaluated-position count is the stone's number.**

### 2. The gate

Assert the evaluated-position surface. ⛔ **Do not assume it is zero and do not write the gate
around zero before measuring.** If it is non-zero, *that is the finding* — report the names and
their sites and **stop**; a non-empty evaluated surface means a user write can still reach a
stdlib verdict, and step C is refuted a third time. Landing nothing and reporting that is a
complete, successful outcome for this stone.

If it is zero, the gate holds it there. Gate on **the property** — "no evaluated-position leaf in
a stdlib body names something outside the reserved prefix" — with the violating **file, line, name,
and enclosing fn** in the message, and what a violation means (a new door by which a user
declaration can change a stdlib verdict). Not a path list, not a grep.

### 3. ⭐ The fresh control for the 8c sweep — the demand above

A test that goes **RED if the `check:restricted-call(ALL fns)` sweep stops running over stdlib
bodies.** The inverted witness cannot do this any more; derive a new one. The shape is yours, but
it must distinguish *"the sweep ran and found nothing"* from *"the sweep did not run"* — those are
the two worlds a green currently cannot tell apart, and the whole point is that C will try to
create the second one.

### 4. Correct the stale justification

Whatever survives of the over-approximating walk keeps its place — it is still the right early
warning for the path and signature channels, which no boundary exempts. But `:215`'s stated reason
is now false and must say what is actually true. ⚠ **Do not delete the two-name assertion**; a
name entering by *any* channel is still a deliberate decision.

## Scope wall

⛔ **This stone changes no `src/` behaviour and elides nothing.** Tests, and the minimum `src/`
visibility needed to reach `quote_boundary` from a test (it is `pub(crate)`). If the census sends
you toward repairing a violation in `wat/*.wat` — **stop and report**; a corpus edit is its own
stone with its own floor, and `.wat` never gets hand-edited (`CLAUDE.md`: the codemod is the path).

## The four questions

- **Obvious** — one sentence, and the completion of a ruling already given.
- **Simple** — one walk reusing the existing classifier; no fifth hand-rolled copy of the
  quote boundary (`src/resolve/mod.rs:7` names that drift as the thing to avoid).
- **Honest** — replaces an expired justification with a true one, and discharges a demand the
  previous stone filed against itself.
- **Good UX** — silent for users; for a stdlib author it names their own line, instead of blaming
  a user's `def` for nine errors in files they have never opened.

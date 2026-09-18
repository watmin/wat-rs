# DESIGN — a mention in a quoted form is not a call

**Drawn 2026-09-18**, builder-directed: *"this session exists to seek and destroy incorrectness"*, on
being shown that the Tier B refutation is itself a defect rather than a fact of life. **NOT STRUCK.**

## The defect

```
(:wat::core::defn :user::main {:restricted-to [:my::]} [] -> :wat::core::nil
  (:wat::kernel::println "hi"))
```

One metadata map, on the user's **own** function → **9 × `DefRestrictedCallerNotAllowed` across SIX
stdlib files** (`kernel/services/stdio.wat` ×3, `cache.wat` ×2, `telemetry/span.wat`,
`telemetry/journal.wat`, `query/mem.wat`). Reproduced by the orchestrator. Baseline: `"hi"`, exit 0.

**Why:** `walk_for_restricted_call` (`check.rs:1648`) fires on **every `WatAST::Keyword` leaf**, including
leaves inside quoted `(:wat::core::forms …)` **child-program templates**. Nine stdlib `service-forms`
bodies quote `:user::main`.

⛔ **A mention inside a quoted template is not a call.** It is **data** — a name that will be resolved in
a *different program*, at a *different time*, by a *different caller*. Applying a **caller**-restriction
to a template checks the wrong thing at the wrong time.

★ And the code disagrees with its own comment: `check.rs:722` says *"walk every call site; if the call
**head** names a…"*, while `:1648` matches every keyword. ⚠ The comment is the stale one — arc 198
widened this deliberately (*"a restriction governs MENTION, not head position"*) — but that makes **three
comments today** describing narrower behaviour than their code (`Select`'s "different `RefCell`s",
`check_program(&bundle.residue)`'s argument name, this).

## ⛔ PHASE 1: READ ARC 198 BEFORE NARROWING IT

The widening was **deliberate and has a purpose** — presumably to stop a restricted name being smuggled
past the check by not calling it directly (passed as a value, resolved dynamically). **That purpose must
survive.**

1. **Read arc 198's DESIGN/reasoning.** State what it was defending against, in its own words.
2. **Census the corpus, form-aware** (`wat --grep`, never text — text has been wrong four times on this
   family of question): does **any** site rely on a restricted name being caught via a mention that is
   *not* a call? If yes, **STOP and report** — the narrowing would reopen what 198 closed.

## The narrowing, and the boundary that will bite

Quoted forms are data. **But not everything inside a quasiquote is quoted:**

```
`(… ~(:user::main) …)      ← the unquote is evaluated NOW, in THIS program: a real mention
`(… (:user::main) …)       ← template text: resolved later, elsewhere
```

⛔ **The fix must distinguish quoted-and-not-unquoted.** A narrowing that exempts everything under a
quasiquote would reopen 198's hole through `~`. **That boundary is the whole implementation**, and it is
where a plausible-looking fix goes wrong.

## Proof obligations

- the witness dies: `{:restricted-to [:my::]}` on `:user::main` → `"hi"`, exit 0;
- **a real violation still fires** — a genuine restricted call from a disallowed namespace, driven;
- **an unquoted mention inside a quasiquote still fires** — 198's hole stays shut, driven;
- the second witness (`:user::spawn::service-locus`, 10 bodies) also dies.

## Out of scope — named so they are not lost

1. **The blame-inversion diagnostic.** After this fix the stdlib errors vanish, so there is nothing to
   re-blame *here* — but a genuine violation should still name the **declarer**, not the mentioner. Its
   own stone.
2. **The rendezvous contradiction.** `:user::main` is what the kernel invokes, so
   `{:restricted-to [:my::]}` on it is self-contradictory and the user deserves to be told exactly that
   — *"`:user::main` is the entry point the kernel invokes; restricting it to `[:my::]` would forbid
   that."* ⭑ **The one message that would have helped is the one nobody prints.** Its own stone.
3. **Tier B.** This closes the **eighth door**; it does not prove the closure. Re-running that proof is
   Tier B's job, and the prize returns to the full **124.65 ms** rather than the 121.58 ms partial.
4. `runtime.rs:2112`'s `Existing::Equivalent`, `check.rs:6068`, `env.rs:456` — still open from earlier
   stones.

## Trap-doors

1. ⛔ **Do not narrow past the witness.** The target is *mentions inside quoted data*. Anything broader
   reopens arc 198.
2. **`~` unquote is the boundary** — see above. Drive it.
3. **A green floor proves little**: the corpus contains no program that exercises a restricted call from
   a disallowed namespace unless you write one. Drive the positive control.
4. **This is a semantics change to a deliberate decision.** If the census says the corpus relies on the
   current width, that is the finding — report it rather than proceeding.

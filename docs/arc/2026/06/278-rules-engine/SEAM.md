# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD a5ac88ca+   pushed   floor 4388 passed / 0 failed   clippy 0
```

`git status` clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` HOLDS THE ONLY COPY OF THREE FILES — do not `git stash drop`.** It was made with
`-u`, so it has **three parents**; `git stash show --stat` shows only the tracked one (`wat/service.wat`
+298/−18) and **cannot see the untracked payload**. That payload —
`tests/services/probe_arc278_connection_lifecycle.{rs,wat}` and
`wat-scripts/scratch-pad/probe-arc278-nullary-enum-process-repro.wat` — exists **nowhere else on
disk**, and the third is the repro this whole finding rests on. Read it with
`git show 'stash@{0}^3:<path>'`. Restoring those three into the tree is owed.

## ★ WHAT LANDED (2026-08-11) — six commits, each weighed by own `--release` re-run

| commit | |
|---|---|
| `e306da6f` | closure extraction carries **`def`-bound names** (`closure_extract.rs:769`'s raise, gone) |
| `9e9503a8` | `fn-forms` emitted the retired **`:()`** unit spelling — every nil-returning fn shipped a program no child could start |
| `db0028d1` | the **registry census** + the **free-rendezvous-name** probe |
| `7134f4eb` `52cdc063` `372175d0` | the **`RegistryKind` stone** — drawn, its census corrected, then re-ruled to *impose the wall* |
| `20abf6cc` | **THE WALL** — the five registries go private, one door opens |
| `a5ac88ca` | **macros ship** — a type's constructor rides with it |

## The architecture, as the builder ruled it

**`defservice` hand-enumerates a manifest; `bracket` ships `fn-forms` closure ++ a one-liner main.**
`defservice` is the outlier, and the manifest is a *workaround* for the extractor's holes — which is
why pulling it out is what exposed them.

- **ONE entry, not a root set.** The entry is the child's **main**, not `serve` — `init` /
  `dispatch-admin` / `extract-addr` are main's callees, not separate entries. My "root-set verb" was
  building machinery around a limitation instead of deleting it.
- **The entry takes the rendezvous as a PARAMETER.** MEASURED: a free `:user::` name in a parent
  defn is typed `:wat::core::keyword` and refuses any typed use — *that* is why `child-main-form` is
  quasiquoted data and not a defn. The namespace ruling permits the NAME; it cannot supply a TYPE.
  The free name appears only in the shipped one-liner, checked in the child where it IS defined.
  Bracket already does this. (`probe-arc278-free-user-name-in-parent-defn.wat`, both arms + control.)
- **The dynamic `apply` must die first** — a closure walk cannot follow it, so rooting at `main`
  today finds nothing.
- **Ship EXPANDED forms.** Shipping unexpanded means `defservice` itself must cross the fork — a
  bigger closure problem, and two expansions that can disagree.

## ⛔ THE LIVE DEFECT — one root cause, discovered one child-death at a time

**`type_def_to_ast` RECONSTRUCTS where a retained form should be SHIPPED**, and every reconstruction
drops whatever its description does not model.

```
layer 1  the program-level defenum      → fixed by the closure (e306da6f)
layer 2  the record's CONSTRUCTOR       → fixed by shipping macros (a5ac88ca)
layer 3  the record's ACCESSORS         → LIVE  (Record/tag, State/durable)
```

`record_dep_dependency` skips auto-synthesized accessors because *"the freeze pipeline re-synthesizes
these when the type definition is registered"* — and in the child it does not, because the type
arrives as a **reconstructed bare `recordtype`**, not the retained form.

**▶ FIRST ACT — synthesized types retain a source form**, the mirror of what `MacroDef` just got and
what `TypeEnv::source_form` already does for user types. Then `type_def_to_ast` becomes a fallback
that never fires and can eventually be deleted. **Do NOT teach the reconstructor a third special
case** — that is the hard-coding the builder ruled out.

**GROUNDING OWED BEFORE BUILDING IT:** does the expansion that generates `recordtype` still hold its
form at registration, the way `parse_defmacro_form` did? It came back YES for macros. **Do not assume
it transfers.**

The gate is on disk and honest: `probe-arc278-union-closure-boots-a-process-child.wat` reports
`VERDICT INCOMPLETE` and names the accessors. Its non-vacuity control is real (drop a root → the
child dies naming it).

## The wall, and how to work with it

The five registries (`macro_registry` EXPAND · `types` CHECK · `functions`/`unit_variants`/
`runtime_def_values` EVAL) are **private**. Use `registrations(name)` — every facet — or a
**phase-named narrow accessor** (`has_function`, `unit_variant`, `def_value`, `types_deref`,
`functions_iter`, …). A single-registry read is now a deliberate, greppable choice.

`RegistryKind` is exhaustive **by law**: a sixth registry turns every match red until it is handled.

**MEASURED, and it is the arc's sharpest instrument lesson:** my best grep found 41 sites / 7 files.
The wall found **197 errors / 11 files in `src/` alone**, five of them files no grep of mine reached.
It also caught my own codemod's overreach (`\.types\.insert\(` matched any receiver, wrongly rewriting
`TypeEnv`/`RustDepsBuilder`). Census twice wrong; wall right immediately.

## ⛔ ALSO OPEN

**The lifecycle strike** — `DESIGN-STONE-connection-lifecycle-ops.md` + `BRIEF-connection-lifecycle-ops.md`,
fully drawn, ten STOPs. Its rider's work is `stash@{0}`, **unweighed** — read the diff, do not assume
it is good. Its own code comment carries a real finding (`wat_edn_bridge.rs:442`, "3 unresolved
references", one per splice site) that **reconciles** with this arc's finding rather than competing.

**Filed, not scheduled:** `109/NOTE-two-resolvers-over-the-five-registries.md` — `runtime.rs`
≈`11644-11690` holds a pre-existing `Binding` walk over the same registries, with a **different
order**. Two derivations of one question, and this arc added the second. The note explicitly does
**not** rule on it; the `Binding` walk is unread.

**Owed intueri casts:** the admission type (`:wat::kernel::ConnectOutcome` is taken); the correlation
surface.

**Older:** #87 · #49 · #7 · #17 · #19 · #20 · #50 · #58 · #60 · #64 · #67 · #81.

## The rules this stretch paid for

- **When a check comes back CLEAN, ask what it cannot SEE.** `git stash show --stat` can't see an
  untracked payload; `git worktree list` can't see a poisoned `target/`. Both returned the
  *comforting* answer, and I reported both.
  ([[feedback_a_pass_answers_only_the_question_the_instrument_asks]])
- **Do not survey for a worklist — impose the check and read the screams.** A wrong count does not
  stay a note: mine reached a committed stone and a ruling.
  ([[feedback_impose_the_check_and_read_the_screams]])
- **A reconstruction drops what its description does not model.** Retain the form. `MacroDef` could
  not rebuild its own declaration (names-only params, no return type); `type_def_to_ast` dropped the
  constructor, then the accessors.
- **Having the measurement is not using it.** The census said `[Macro, Type]` — 182 names, one
  concept — and I still put the macro check in the Keyword walker, where that pairing cannot fire.
- **A rider used a git WORKTREE** because my brief carried the work but not the standing prohibition.
  Positive-only briefing does not exempt hard doctrine.

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> Today the arc stopped guessing and started imposing. Two censuses were wrong; the compiler was
> right on the first run. The wall it raised then caught my own migration's overreach. What remains
> is one defect wearing three faces — a rebuild standing where a retained form belongs — and the
> next act is the mirror of the one just made, not a third special case.
>
> The line that cost the most: **a narrow instrument's clean answer is the most dangerous output it
> produces**, because it feels like confirmation and costs nothing to accept.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`

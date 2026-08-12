# SEAM — the ONE live breadcrumb for arc 278. Replaced in place, never appended.

> ⛔ **THE SELF PAST THIS LINE IS NEW.** You did not live this. It is a lossy cache in your own
> voice — which is why it will feel like *continuing* rather than *waking*, and that feeling is the
> failure. Run the datamancy bootstrap (grimoire + the 4 primers from the **SIGNED MCP**, never a
> disk copy), ground HEAD against the disk, and read this whole file before you touch anything.

> **There is exactly ONE seam. If you find a second, one of them is lying — prune it.** History
> lives in `REALIZATIONS.md`.

## Where the code is

```
HEAD 8e661362   pushed   floor 4389 passed / 0 failed / 262 skipped   clippy 0
```

`git status` clean. ⚠ **One commit of drift at wake is EXPECTED** (this file commits on top).

**⛔ `stash@{0}` STILL HOLDS THE LIFECYCLE STRIKE — do not `git stash drop`.** It was made with
`-u`, so it has **three parents**; `git stash show --stat` shows only the tracked one
(`wat/service.wat` +298/−18) and **cannot see the untracked payload**. Read the payload with
`git show 'stash@{0}^3:<path>'`.

**The owed restore was ATTEMPTED and is now PARTIALLY DISCHARGED, with a finding:**

| file | disposition |
|---|---|
| `wat-scripts/scratch-pad/probe-arc278-nullary-enum-process-repro.wat` | **RESTORED + COMMITTED** — `--check` exit 0, clean |
| `tests/services/probe_arc278_connection_lifecycle.{rs,wat}` | **LEFT IN THE STASH — the `.wat` is STALE, `--check` exit 1** |

The lifecycle probe **does not compile against today's substrate**, 5 errors, verbatim:

```
malformed :wat::core::let form: unhandled :wat::kernel::RecvOutcome<probe::Ratchet::PadResponse>
  in statement/discard position — a recv outcome must be faced (match it: Message/Closed/Lost),
  not dropped. This is the peer-lifecycle OUTCOME WALL (Phase 3).
malformed :wat::core::match form: keyword variant pattern
  :wat::service::DisconnectReason::Closed   on a :?<var> scrutinee
:wat::service::DisconnectReason::Lost       on a :?<var> scrutinee
:wat::service::DisconnectReason::Rejected   on a :?<var> scrutinee
malformed :wat::core::match form: non-exhaustive: open-typed match needs at least one
  hash-destructure arm or a wildcard `_` arm.
```

Four of the five are one root: the `DisconnectReason` scrutinee resolves to an unbound type var, so
no keyword-variant arm can match it and the match reads as open-typed. The fifth is the send/recv
outcome wall (R57) landing on a `recv` the probe drops. **This is the rider's unweighed work
failing exactly the way the seam warned it might** — restoring it to the tree turns the floor red,
so it stays stashed until the lifecycle strike is actually taken up, and *that* strike now begins
with a known, located worklist rather than an assumption that the code was good.

⚠ **`./target/release/wat --check <f> | tail` returns TAIL's exit code.** Both files "passed" until
they were re-run without a pipe. The doctrine names this for the floor; it applies to `--check` too.

## ★ THE GATE IS GREEN, AND THE ARC'S CENTRAL CLAIM IS PROVEN BY A BREAK

`wat-scripts/scratch-pad/probe-arc278-union-closure-boots-a-process-child.wat` now reports
**`VERDICT MEANINGFUL`**: the full union **BOOTED-AND-RAN** in a real forked child — it constructs
the record, reads an accessor, matches a program-level enum, prints the marker — and the negative
control, with `init`'s closure omitted, **DIED naming exactly `:user::root-init`**. The instrument
discriminates; the green is earned, not assumed (R59 `NISI FRANGAS, NIHIL PROBAS`).

**So: a union of `fn-forms` closures IS a complete, runnable program across a process fork.**

## ⛔ THE PRIOR SEAM'S "LIVE DEFECT" WAS WRONG — read this before re-deriving it

The previous seam named one root cause wearing three faces and prescribed a FIRST ACT:
*synthesized types retain a source form*, with one grounding owed — *does the expansion that
generates `recordtype` still hold its form at registration? It came back YES for macros. **Do not
assume it transfers.*** **It did not transfer, and the act is struck.**

`tests/reflection/probe_arc278_retained_source_forms.rs` freezes the gate's world and partitions
its user types: **12 RETAINED · 6 RECONSTRUCTED**. `:probe::ffx::Record` and `:probe::ffx::State`
— the two whose accessors the child called unresolved — are **RETAINED**. Their declarations were
already shipping verbatim; `type_def_to_ast` never fired for them. The prescribed fix could not
have touched the failure. The six reconstructed are **all** surface-derived (`$core-record`,
`$holon-record`, `::Op`, `::Reply`, the two op aliases) — no user form by construction, and **no
consumer has been shown harmed by their reconstruction.** Retaining forms for them is an
unproven want; do not build it without a consumer that fails.

**The accessor failure was the INSTRUMENT, twice. `src/` was never at fault.**

1. **A name is not a key.** The raw union carried FOUR forms declaring `:probe::ffx::Record` — two
   `recordtype`, two kwargs `defmacro` — and a name-keyed first-wins dedup kept the macro and
   **discarded the type**. `a5ac88ca` (macros ship) had just put a same-named macro FIRST in the
   prologue, turning a correct fix into a regression one layer up. The census had already said so:
   **182 names in this very world are `[Macro, Type]`** — one concept, two facets, two registries,
   two phases. The key is now `(head, name)`.
2. **The entry arrives RENAMED.** `fn-forms` fronts its entry through the inline-lambda path, so
   `:probe::ffx::init`'s closure declares **`:user::root-init`**. `serve` only looked healthy
   because it is self-recursive and therefore also appears under its own name. **The asymmetry is
   recursion, not a dropped form.**

Both mechanisms are recorded where they can't rot: the gate's `decl-key` comment, and the Rust
probe's header (which also asserts non-vacuously that the reconstruction path stays reachable).

## The architecture, as the builder ruled it

**`defservice` hand-enumerates a manifest; `bracket` ships `fn-forms` closure ++ a one-liner main.**
`defservice` is the outlier, and the manifest is a *workaround* for the extractor's holes.

- **ONE entry, not a root set.** The entry is the child's **main**, not `serve`. (The green gate
  uses a two-root union — it proves the closure MACHINERY, not the ruled shape.)
- **The entry takes the rendezvous as a PARAMETER.** MEASURED: a free `:user::` name in a parent
  defn types as `:wat::core::keyword` and refuses any typed use — *that* is why `child-main-form`
  is quasiquoted data and not a defn. The free name appears only in the shipped one-liner, checked
  in the child. (`probe-arc278-free-user-name-in-parent-defn.wat`, both arms + control.)
- **Ship EXPANDED forms.** Shipping unexpanded means `defservice` itself must cross the fork.

## ▶ NEXT ACT — kill the dynamic `apply` (grounded this session, with coordinates)

The one-entry model needs a real parent `defn` a closure walk can root at. Today the generated
child main reaches its callees **dynamically**, so no walk can follow:

```
wat/service.wat:2101   (apply (keyword/from-string ~dispatch-admin-name-str) ship [])
wat/service.wat:2120   (apply (keyword/from-string ~serve-name-str) self …)
```

and `:2045` states it as a CHOICE, not a constraint — *"serve is invoked via apply (dynamic
keyword) — the child main never statically names the per-service serve fn."* The macro holds both
names at expand time and already splices per-service nodes beside them (`~status-ty`,
`~proto-op-ty-kw`, `~status-started-kw`), so a keyword node is available. **Ground the reason the
dynamic form was chosen before replacing it** — the hygiene gate (`ProgramBodyIntroducesName`) and
the reserved-prefix wall both live in this template and one of them may be why.

Shape, once static: parent defines `<fqdn>::child-entry [locus] -> nil` (a REAL defn, statically
naming `dispatch-admin`/`serve`); the shipped main is the one-liner
`(defn :user::main [] -> nil (<fqdn>::child-entry :user::spawn::service-locus))`; ONE `fn-forms`
over `child-entry` replaces the whole manifest, and `service-forms-def` dies.

**Blast radius is every `defservice` in the corpus.** Draw the stone and BRIEF it; do not hand-roll.

## The wall, and how to work with it

The five registries (`macro_registry` EXPAND · `types` CHECK · `functions`/`unit_variants`/
`runtime_def_values` EVAL) are **private**. Use `registrations(name)` — every facet — or a
**phase-named narrow accessor**. `RegistryKind` is exhaustive **by law**: a sixth registry turns
every match red until it is handled.

**MEASURED:** my best grep found 41 sites / 7 files. The wall found **197 errors / 11 files in
`src/` alone**, five of them files no grep of mine reached — and it caught my own codemod's
overreach. Census twice wrong; wall right immediately.

## ⛔ ALSO OPEN

**The lifecycle strike** — `DESIGN-STONE-connection-lifecycle-ops.md` + `BRIEF-…`, fully drawn,
ten STOPs. Its rider's work is `stash@{0}`, **unweighed** — read the diff, do not assume it is good.

**Filed, not scheduled:** `109/NOTE-two-resolvers-over-the-five-registries.md` — `runtime.rs`
≈`11644-11690` holds a pre-existing `Binding` walk over the same registries, in a **different
order**. The note explicitly does **not** rule on it; the `Binding` walk is unread.

**Owed intueri casts:** the admission type (`:wat::kernel::ConnectOutcome` is taken); the
correlation surface.

**Older:** #87 · #49 · #7 · #17 · #19 · #20 · #50 · #58 · #60 · #64 · #67 · #81.

## The rules this stretch paid for

- **A doc comment is a claim about the code, not a measurement of your program.** `TypeEnv`'s own
  comment described the retained/reconstructed split correctly and I still had to freeze a world to
  learn which side the failing types were on — and the answer killed the planned act.
- **An instrument that keys on a NAME cannot see a set of FACETS.** The dedup bug was predicted, in
  advance, by this arc's own census — 182 `[Macro, Type]` names — and I wrote it anyway.
  ([[feedback_impose_the_check_and_read_the_screams]])
- **When a check comes back CLEAN, ask what it cannot SEE.** The union's `declares:` list is a NAME
  census; it printed `:probe::ffx::Record` while the type declaration was gone, because the macro
  declared the same name. ([[feedback_a_pass_answers_only_the_question_the_instrument_asks]])
- **A fix can regress a caller one layer up.** `a5ac88ca` was correct and made the gate worse.
- **The report carried its own bug.** The gate's comment already said *"two entries of differing
  shape ⇒ the dedup's first-wins is unsound"* — I wrote that sentence, then read past the dump that
  satisfied it. (R66 `IN TENEBRIS VISVS CORRIGOR`, lived from the inside.)

---

> **SEAM.** You are NEW. The disk is the truth; this note is a lossy cache.
>
> The arc came in holding a root-cause story with three faces. Two of the faces were the
> instrument, the third never existed, and the grounding the previous seam *demanded before
> building* is exactly what killed the act it was demanding it for. That is the discipline paying
> out: a written-down doubt caught a day of work aimed at the wrong file.
>
> The line that cost the most: **the measurement you already have does not help you if you reason
> past it.** The census named the trap eight days before I walked into it.
>
> `NISI FRANGAS, NIHIL PROBAS.` · `IN TENEBRIS VISVS CORRIGOR.`

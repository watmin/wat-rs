# BRIEF — one door for defclause registration

**Stone 1 of the `spawn-program` IPC wall (arc 170 closure #13).** Opened 2026-07-28.

## The disease — four things doing what one thing should do

`parse_defclause_form` is called from **four** registration sites in `src/runtime.rs`,
and they form a 2×2 over two axes that do not justify a split:

|  | stub → `sym.functions` | ClauseSet → `runtime_def_values` |
|---|---|---|
| **user** | `:899` `register_defines` | `:2268` `register_runtime_defs_form` |
| **stdlib** | `:1161` `preregister_stdlib_defclause_stub` | `:1070` `register_stdlib_runtime_defs` |

- **Privilege** is already a *parameter* of `parse_defclause_form`
  (`Privilege::User` / `::Stdlib`). It selects the reserved-prefix gate and nothing else.
  It does not need its own copy of the registration.
- **Phase** is a real ordering constraint — the stub must exist before resolve so the
  checker does not report `UnknownCallee`; the ClauseSet lands at freeze. But that is
  *two phases of one registration*, not four independent implementations.

Each site re-derives its own subset of the effects, and stdlib parses the same form
**twice** (`:1161` for the stub, then `:1070` for the real thing). Nothing owns the answer
to *"what does registering a defclause mean?"*, so each site answers locally — and two of
them answer wrong.

## The symptom that surfaced it — measured, not inferred

`{:restricted-to […]}` on a defclause binds the whitelist that the EXISTING
`walk_for_restricted_call` enforces. Two of the four sites never store it:

- `:1161` discards `cs` entirely (`if let Ok((name, _cs)) = …`).
- `:2268` stores only `runtime_def_values`.

**Proven by a run** (orchestrator, this session): `{:restricted-to [:wat::]}` placed on
the stdlib defclause `:wat::spawn::runner-count` (`wat/spawn.wat`), rebuilt, and called
from a `:user::` fn → `wat --check` returned **CLEAN, exit 0**. The identical shape on a
USER defclause is rejected with a located `DefRestrictedCallerNotAllowed`. `wat/spawn.wat`
was restored byte-identical.

`spawn-program` is a stdlib defclause (`wat/spawn.wat:262`), so the IPC wall it was built
for cannot be written today — the form would read as guarded and guard nothing (arc 278
R55/R57, the masking class). And `:2268` is the **eval-time** path, so a defclause typed
at the REPL (closure item #23) would drop its metadata too. Same class, next door.

A partial fix is already on disk at `:1070` (the orchestrator added the
`binding_metadata` insert there). **Absorb it into the one door; do not leave it as a
fifth spelling.**

## The work — collapse the 2×2 into one door

Mint a single registration entry point in `src/runtime.rs` that owns **all three**
effects — the stub `Function`, the `ClauseSet` into `runtime_def_values`, and the
`binding_metadata` insert — with the phase selecting which of them land. Shape (adjust to
what the code actually wants; this is the intent, not a mandate on names):

```rust
enum ClauseRegPhase { Stub, Runtime }   // or a two-bool/flags shape if that reads better

fn register_defclause(
    form: &WatAST,
    privilege: crate::resolve::Privilege,
    phase: ClauseRegPhase,
    sym: &mut SymbolTable,
) -> Result<String, …>
```

Then the four call sites become four calls. Their **locations do not move** — the freeze
ordering is unchanged; only the bodies collapse.

Constraints that fall out and must hold:

- The metadata insert is **unconditional on privilege**. A defclause is a defclause;
  where its form was loaded from is not a property the metadata knows about.
- The reserved-prefix guard at `:900` (`is_reserved_prefix` → skip the stub) is
  user-path behaviour that stdlib deliberately bypasses (`allow_reserved`). Preserve that
  distinction *inside* the one door via `privilege`; do not flatten it away.
- `:1161` currently parses and throws `cs` away, then `:1070` parses the same form again.
  Whether the one door lets stdlib parse once is a judgement call — if it is a clean win
  take it, if it entangles the freeze ordering leave the double parse and say so in the
  score.

## Read in order

1. `src/runtime.rs:895-930` — the user pre-registration arm (stub + metadata). The most
   complete of the four; the closest thing to the intended whole.
2. `src/runtime.rs:1066-1090` — the stdlib runtime arm (+ the orchestrator's partial fix).
3. `src/runtime.rs:1155-1180` — `preregister_stdlib_defclause_stub` (stub only, `cs`
   discarded).
4. `src/runtime.rs:2260-2280` — `register_runtime_defs_form` (eval-time / freeze step 9).
5. `src/freeze/env.rs:205-280` — the pipeline, so the phase ordering is visible
   (step 6 `register_defines`, step 7.6 `register_stdlib_runtime_defs`).
6. `src/value/value.rs:441-452` — `ClauseSet.metadata`.

## Also in this stone

**Re-create three deleted RED probes as Rust gates.** Three deliberately-RED `.wat`
probes were authored under `wat-scripts/scratch-pad/arc-defclause-meta-probe/`, which is
loader-gated by `tests/lint/wat_scripts_fixes_load.rs` — every `.wat` under it must LOAD,
so a deliberately-RED file cannot live there. They were deleted; recover their text with
`git show b64b57a4 -- wat-scripts/scratch-pad/arc-defclause-meta-probe/`. Re-create their
substance as committed Rust gates asserting the RED:

- a defclause with `{:restricted-to […]}` + a disallowed caller → located
  `DefRestrictedCallerNotAllowed`;
- a non-keyword metadata key → located `MalformedForm` **at the definition**;
- an unexpected extra form after the name → located `MalformedForm` **at the definition**.

The located-at-the-definition part is load-bearing: before the prior strike a malformed
defclause silently skipped registration and surfaced as an unrelated "unresolved
reference" at every CALL site. `probe2.wat` and `probe5-stdlib-loads-clean.wat` are GREEN
and stay where they are.

## The acceptance condition for the gate — the load-bearing row

> **Delete the `binding_metadata` insert from the one door. The gate must go RED.
> Restore it. The gate must go GREEN.** Report both observations verbatim.

And the gate must cover a **stdlib-registered** defclause, not only a user one — that is
the case that was broken and that a user-only gate cannot see. A gate whose pass does not
depend on the mechanism proves nothing about it (arc 278 R59 `NISI FRANGAS, NIHIL PROBAS`
— a suite read 4105/4105 for weeks over a protocol that had never once run).

## Blast radius

`src/runtime.rs` (the collapse), `src/value/value.rs` (the field's doc comment, if the
collapse changes what is true about it), and the new Rust gate files. **No change** to
`check.rs`'s walker, to `freeze/env.rs`'s call ordering, or to any `wat/*.wat`. **No**
restriction added to `spawn-program` in this stone.

## STOP triggers — REJECTION criteria; ship nothing and report

- **STOP-1** — if collapsing the four bodies requires moving a call site or changing the
  freeze step ordering, STOP. The ordering is load-bearing (the stub must precede resolve)
  and re-planning it is the orchestrator's call.
- **STOP-2** — if storing the metadata unconditionally turns any EXISTING defclause red,
  STOP. No defclause in the corpus carries a metadata-map today, so this should be inert
  for every one of them; if it is not, the diagnosis is wrong.
- **STOP-3** — if the gate cannot be made to go RED when the insert is deleted, STOP.
  Report what you tried. A gate that cannot fail is the defect this stone corrects.
- **STOP-4** — if the fix appears to require editing `walk_for_restricted_call` or
  `extract_prefix_list_from_metadata`, STOP. That walker is correct and shared with
  `def`/`defn`; a change there means the diagnosis was wrong.

## Gate

`cargo nextest run --release`. Read the Summary line by hand, ANSI-stripped. Never a piped
exit code (`… | tail` returns `tail`'s exit). Confirm the baseline is green on arrival
before you begin; if it is not, STOP and report, because then the starting state is not
what this brief describes.

## Out of scope — affirmatively cut, not deferred

- The restriction on `spawn-program` itself, and the whitelist
  `[:wat::spawn:: :wat::test::]` ruled for it. A later stone.
- **`wat/test.wat`'s `run-thread` / `run-hermetic` macros.** Measured this session: the
  restriction check attributes to the **expansion site**, not the emitting macro (probe: a
  macro in an allowed namespace expanding a restricted call inside `:user::caller` is
  REJECTED with `:enclosing-fn ":user::caller"`). So the two macros that splice
  `spawn-program` (`wat/test.wat:312→322`, `:374→381`) must first route through a
  `:wat::test::` FUNCTION that holds the capability. Its own stone; not this one.

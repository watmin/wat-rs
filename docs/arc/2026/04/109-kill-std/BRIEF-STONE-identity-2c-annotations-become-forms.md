# BRIEF — identity 2c: ANNOTATION bindings emit the `:-` form

Authority for WHICH bindings: `TABLE-defservice-type-name-sites.md`, role column = **ANNOTATION**.
DESIGN: `DESIGN-STONE-the-angle-string-is-not-a-type-identity.md`.

## ⚠ The table's line numbers are STALE — twice over

Stone **2a** deleted 6 bindings (13 lines) and stone **2b** split 7 more into per-role aliases.
**Locate every binding by NAME.** The table's names and roles are correct; its numbers are not. Say
in your report which you re-located that way.

## The work

`defservice` builds type names as strings and mints them with
`(:wat::core::keyword/from-string <str>)` — a KEYWORD carrying the angle spelling
`Head<A,B>`. For every binding the table classifies **ANNOTATION**, mint the reference FORM instead:

```clojure
(:wat::core::keyword/to-type-form-colon (:wat::core::keyword/from-string <str>))
```

The string building is untouched. `keyword/to-type-form-colon` is F5-admitted
(`src/macros/eval.rs:676`), so a macro body may call it.

## ★ The converter passes monomorphic names through UNCHANGED — this is your control

Measured:

```
:wat::query::mem-store::State           ->  :wat::query::mem-store::State          (unchanged)
:wat::cache::lru-svc::State<K,V>        ->  (:wat::cache::lru-svc::State :- [K V])
:wat::kernel::Peer<…Reply,…Op>          ->  (:wat::kernel::Peer :- [:…Reply :…Op])
```

So a **monomorphic** service's expansion must come out **byte-identical**, while a **parametric**
service's changes in exactly the annotation slots. That asymmetry is the acceptance row.

## This works now — and did not an hour ago

Blocker 5 shipped (`b9df7a09a`): the expander no longer expands `(Head :- [args])`, so a form
reference to a `defrecord`/`defstruct`-minted type checks. Verified on the exact form this stone
emits:

```clojure
[s <- (:wat::cache::lru-svc::State :- [K V])]     ✅ checks
```

Without it, `state-ty`, `record-ty` and `handle-name` would all have failed.

`extend-type`'s satisfied-surface slot also accepts a form — added during the A-i flight and kept
through S2 — so `dialable-ty` / `typedcap-ty` are safe there.

## What this stone does NOT do

**DECL-NAME and RUNTIME-ARG bindings stay exactly as they are.** Their destinations are **unruled**;
`defservice` will still emit the angle spelling for those after this stone, and that is correct and
legal (both spellings remain valid until ③). Do not convert them, and do not "finish the job."

Nor the CTOR-ARG binding (`selectable-entry-ty-ctor`), nor the two OTHER sites (`surface-kw`,
`launch-head-kw`, and `record-ty-str`'s `keyword/to-string` read — flagged UNRESOLVED by 2b, with an
inline comment saying so).

## ★ The acceptance row — a differential expansion diff

1. **BEFORE editing** — `cargo build --bin wat`, then `macroexpand` **both**: `wat/cache.wat`'s
   parametric `lru-svc` and `wat/query/mem.wat`'s monomorphic `mem-store`. Save both.
2. Edit.
3. Rebuild, re-expand both, diff each against its own BEFORE.

**Expected:**
- **monomorphic — EMPTY diff.** If it moves, the converter is being applied where it should be a
  no-op, or to a binding that is not ANNOTATION.
- **parametric — changes ONLY in type-annotation positions**, each an angle keyword becoming
  `(Head :- [args])`. Read the whole diff. Anything that is not that shape is a finding.

⚠ `wat/service.wat` is the stdlib, `include_str!`-baked at RUST-compile time — the rebuild between
edit and expansion is mandatory, and the BEFORE capture must come first.

## Boundaries

- Do NOT run `scripts/floor.sh` or a full `cargo nextest` — the orchestrator measures centrally.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.
- Touch `wat/service.wat` only.
- Convert ONLY bindings the table marks ANNOTATION.
- Delete any scratch `.wat` that must fail; `tests/lint/wat_scripts_fixes_load.rs` type-checks
  everything under `wat-scripts/`.

## Your own checks

`cargo build --bin wat`, then the expansion diffs above, plus
`cargo nextest run --release -E 'binary_id(wat::services)'` for a scoped run.
Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Diagnostics go to **stderr** — judge by exit code AND empty output, never grep alone.

## STOP triggers — ship nothing further and report

- **STOP-1.** If the MONOMORPHIC expansion diff is not empty, STOP and report it. A monomorphic
  service has no type args, so nothing it emits should move.
- **STOP-2.** If a converted annotation lands in a slot that rejects a form, STOP and report the
  slot and the verbatim error. That is a consumer we have not taught, and it is a finding, not
  something to route around.
- **STOP-3.** If converting a binding requires touching a DECL-NAME or RUNTIME-ARG alias, STOP —
  2b's split exists precisely so that cannot be necessary, and needing it means the split was wrong.

## Your report

The diff. Both expansion diffs — state the monomorphic one is empty, and paste the parametric one in
full. Which bindings you re-located by name. Any slot that rejected a form. What surprised you.

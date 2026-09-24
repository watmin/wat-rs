# BRIEF — STONE 255.25a: a variant is a type the moment its enum is

**Drawn 2026-09-24 against `main` @ `9935d42ad`.** Floor 6058/6058, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 215. **Executor tier: Opus.**

## Why — 255.25 stopped at STOP-3, and the gap predates it

255.25 (C-b4) respelled the markers as `:wat::kernel::Transport.Shared`/`.Wire`, and the stdlib then refused
to start:

```
#wat.type/EdgeFreeTypeName … extend-type :wat::spawn::ThreadOpts → (:wat::spawn::Locus :- [:wat::kernel::Transport.Shared]):
:wat::kernel::Transport.Shared in the target is neither declared in the edge's `:- [P …]` binder nor a known type …
```

The orchestrator reproduced it on `main` with two probes that differ only in how the marker is declared
(committed beside this brief as `probes-255.25a/*.wat.txt`):

- a `defstruct` marker in an `extend-type` target: rc=0;
- **a `Pure` enum variant in the same position: rc=3**, the same `EdgeFreeTypeName`.

**The mechanism** (from the executor's read):

- 255.22's edge wall runs at `extend-type` registration and asks `is_known_type`.
- An enum's variant singletons (`E.V`) are registered only **later**, in a separate
  `TypeEnv::register_variant_types` pass (`src/types.rs:1574`, called from `src/freeze/env.rs`
  :144/:198/:520).
- So at the moment the wall asks, a variant is unknown, while a struct is known the moment it is
  declared.

The brief for 255.25 relied on `wat-scripts/scratch-pad/255-17b-transport-family-shapes.wat`, which never
puts a variant in an edge. That was the orchestrator's error.

## Ruled by the four questions

**V1: a variant's type is registered in the same act that registers its enum.** It is Obvious, Simple,
Honest and Good UX: a variant exists when its enum is declared.

- Rejected **V2**, having `is_known_type` also consult `variant_parent_enum`. It is **Simple N**: two
  authorities for "is this a type", which the *exactly one way* ruling forbids.
- Rejected **V3**, running the edge wall after the variant pass. It is **Obvious N**: a registration wall
  that depends on a pass order invisible at the call site.

## The work

1. Wherever an enum's `TypeDef` is registered (the `defenum` arm of `splice_type_decls`, and any other door
   that registers an enum: **census them**), register its variant singleton types in the same act,
   through one helper.
2. **One way:** the separate `register_variant_types` pass is **retired**, not kept as a second path.
   - If a caller needs it for something the per-enum registration cannot do (e.g. enums registered by a
     door that bypasses `splice_type_decls`), route that door through the helper instead.
   - If the pass cannot be retired, that is STOP-1.
   - `src/freeze/pass_order.rs` pins the pipeline order (255.12). Update it honestly, and re-read every
     "post-step ⇒ unreachable" disposition it names before you change the array.
3. **Door-replace:** `retract_for_door_replace` must retract an enum's variant types together with the
   enum.
4. **Rows** (`tests/types/probe_arc255_25a_*`):
   - the variant probe: rc=0 and prints (pre-stone rc=3);
   - the struct probe: rc=0 (both binaries);
   - a variant of an enum **declared after** the edge, in the same file: measure it and report it. Say
     whether declaration order matters for structs today too; match the struct behaviour, whatever it is;
   - an undeclared variant (`:probe::Tr.Nope`) in an edge: still refused.

## STOP triggers

1. `register_variant_types` cannot be retired, because some caller needs a whole-env pass → STOP, report
   the caller and why.
2. Registering variants earlier changes an existing verdict anywhere in the floor or census → STOP, report
   each verbatim.

## Expectations — fixed before the strike

| what | expected |
|---|---|
| the variant probe | rc=0, prints `"hello"`; pre-stone rc=3 |
| the struct probe | rc=0 both |
| an undeclared variant | refused |
| `register_variant_types` | gone (or STOP-1) |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` against a pre-change census · NEW 2 / RECOVERY 0 · ≤ 215 |

Runtime prediction: 1–2 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name the
  arm. A red caused by the stone is fixed at its cause and the whole floor re-run; say which.
- ⭐ Prove every row can say BOTH words. Capture `rc=$?` on the next statement.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents. If you stop without committing, revert your own edits and
  save the patch to the scratchpad.

## After this

255.25 resumes from its saved patch (`scratchpad/s25/255.25-stopped.patch`, plus its codemod in
`s25/untracked/`).

# BRIEF — STONE 255.48: a featureless surface's members are the declared ones (Hole B, ruling B1)

**Drawn 2026-09-26 against `main` @ `9d73f9b0b`.** **Executor: grok, via pulsare.** A strike: `src/`, `wat/`,
tests. Commit locally on `main`; **do not push**. Then `pulsare_yield kind=scored`.

## The ruling (builder, 2026-09-26, four YES)

**B1:** a surface with **no members** is satisfied **only by a declared `extend-type` edge**, whether or not it
has type parameters. A surface with members keeps width subtyping, unchanged. Today the parametric featureless
`Spawned` already refuses an undeclared record; the non-parametric featureless `Reason` admits any record that
clears its nature floor. After this stone both follow one rule.

Also ruled, for later stones, not this one: **C** (conditional membership via a bounded `extend-type` binder),
**Refuse** (a bounded variable unresolved at the end is an error; the 4 `ord_result_*` tests will annotate `E`),
and **p11 by the unique-solution rule**.

## Read first

- `WEIGH-STONE-255.47-measure-c-and-refuse.md` and its SCORE (§ Hole B: the 68 sites and the 7 types).
- `wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat`: the hello world and `:hello::bad`, the undeclared
  admission.
- `src/check.rs:17638-17680`: `assignable`'s structural surface arm. The aggregate branch calls
  `struct_satisfies_surface` and returns `structural && nature_ok`. A declared edge is decided earlier, by the
  path-to-path `is_subtype` arm, and never reaches this one (255.47 instrumented this: 0 `HOLEB-edge` hits).
- `src/types/surface.rs:1-6` (module doc: width subtyping) and `:48-60` (`members.iter().all`, vacuous on empty).
- `wat/query.wat:73-76`: `Reason`'s comment says any pure record satisfies it ambiently. That sentence is what B1
  retires.

## The change

1. **The arm.** In the aggregate branch of the structural surface arm, a surface whose member list is empty is
   **not** satisfied structurally. Sketch:

   ```rust
   // B1: an empty member list requires nothing, so structure proves nothing; membership is declared.
   // A declared edge was already accepted by the path-to-path `is_subtype` arm above.
   if surf_clone.members.is_empty() {
       return false;
   }
   ```

   Look at the foreign-type branch just below (`17683`, `struct_satisfies_surface` with `&[]` fields) and any other
   caller that can reach a featureless surface. Apply the same rule there, or show it cannot be reached.
2. **The declarations.** Each type that 255.47 found admitted only by the vacuous arm gets one bodiless edge at its
   logical home (next to the type's definition):
   - `:wat::query::Fault` → `:wat::query::Reason` (the stdlib; 45 sites);
   - `:probe::SqliteReason`, `:probe::RedisReason`, `:probe::MongoReason` → `:probe::Reason` (in each probe file
     that defines them: `tests/rete/probe_arc278_open_surface_dispatch.wat`, the `wat-scripts/probes/arc-170/*`
     files);
   - `:probe::Note` → `:wat::query::Reason`;
   - `:env::Rec` → `:env::Portable` (`tests/types/probe_arc293_holder_root_symbol.wat`).

   These are new declarations, one per type, not a rewrite of existing forms. Find every file that defines one of
   these types; a type defined in several probe files gets its edge in each.
3. **The words.** Rewrite `wat/query.wat`'s `Reason` comment and `src/types/surface.rs`'s module doc to say what is
   now true. Add a one-line note wherever a comment claims ambient satisfaction of an empty surface.
4. **The hello world.** In `wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat`, delete `:hello::bad` and
   its comment block, which is now a refused program. The scratch directory is type-check gated, so it cannot hold
   it. Record the refusal in the SCORE with the verbatim message.
5. **Tests.** Add a driven Rust test (beside the surface tests you find) with these rows:
   - an undeclared record is refused by a featureless non-parametric surface;
   - the same record with a declared edge is admitted;
   - a record with the members of a **featureful** surface is still admitted without an edge (width subtyping
     unchanged);
   - an undeclared record is refused by a featureless parametric surface (`Spawned`-shaped).

   The refusal rows assert the **error kind and the names in it**, not `is_err()`.

## Expectations (the orchestrator re-runs these)

| what | how | expected |
|---|---|---|
| the release floor | `scripts/floor.sh` | Summary all passed. Report the number against 6141 at `145d2434a`, plus new tests |
| clippy | `cargo clippy --release --all-targets` | 0 warnings |
| hello world | `./target/release/wat --check wat-scripts/scratch-pad/255-46/hello-extend-type-today.wat` | rc 0 after `bad` is removed |
| the undeclared case | a `/tmp` copy with `bad` restored | rc 1, `TypeMismatch` naming `:hello::Opaque` and `:hello::Orderable` |
| census / delta | `scripts/replay/census.sh --diff`; `scripts/replay/delta.sh` | no new STOP rows; delta NEW 2 / RECOVERY 0 unchanged, or explained |

## STOP triggers

- **STOP-1:** if a site outside the 68 turns red (a type admitted by the vacuous arm that 255.47's census missed),
  add its edge only if it is plainly one of these open-`Reason`-style uses; otherwise STOP and report the site.
- **STOP-2:** if a Rust test asserts ambient satisfaction of an empty surface **as its contract** (not merely using
  it), STOP and report it verbatim. B1 retires that contract, and the builder sees each one before it changes.
- **STOP-3:** if the runtime has its own surface-satisfaction path that disagrees after the change, report it.
  (Measured at the draw: `src/runtime.rs:10001` answers `conforms?` against any surface with `false`, "not yet
  implemented". That is known; do not change it here.)
- A STOP means STOP: report, do not work around it.

## Doctrine

- There is no known flake. On a red: do not re-run; quote the failing block verbatim from `.floor/`; name the arm.
- Capture `rc=$?` on the **next** statement.
- **If this brief contradicts the code, the code wins. Say so plainly.**
- Write `SCORE-STONE-255.48-a-featureless-surface-is-declared.md` beside this brief. Commit the change and the SCORE
  on `main` (`git add -- <paths>`, never `-A`). **Do not push.**

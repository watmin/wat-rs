# BRIEF — 293.W.2f: a process may not dial a shared-memory address

> **Executor: one leaf.** Orchestrator drew this and weighs the kill. Work ONLY in `wat-rs/`.
> Commit nothing. Leave the tree green for the orchestrator to weigh.

## What to do

Make the illegal circuit a **check error**. The live MCP program

```
(echo/start :locus (thread) …)
(bracket/map (process) ["a"] :work :echo eh)
```

type-checks today and dies in EDN. After this stone it must fail at startup/check
with an error that names Shared/Wire/shared-memory — never `RustOpaque`.

This is new-mechanism territory (Address grows T; `/start` must stop erasing it)
composed with pieces that exist: `portable_form`, `address-wire?`,
`infer_listener_prime`'s ThreadOpts vs ProcessOpts split, `bracket::map`'s locus AST.

## Read in order

1. `DESIGN-STONE-293.W.2f-process-may-not-dial-shared.md` (this dir) — the algorithm.
2. `DESIGN-STONE-293.W.2e-address-wire.md` — the mouth. Do not break it.
3. `src/check.rs` `infer_listener_prime` (~9353) — already branches ThreadOpts /
   ProcessOpts / Locus. Stamp T here.
4. `src/check.rs` `bound_type` (~9477), `infer_connect_prime` (~9482),
   `infer_address_wire` (~10858), `is_pure_type` (~12737).
5. `wat/spawn.wat` `Bound` (~278), `Launched` (~291).
6. `wat/service.wat` `addr-ty` (~760), `handle-fields` (~2301), `start-params` /
   `start-fn` (~2209, ~2237). start is kwargs Form A today. That is the erasure.
7. `wat/bracket.wat` `defmacro map` (~838) and `each` (~894).
8. The committed probe: `tests/comms/probe_arc293_W2f_process_dials_thread.{rs,wat}`.

## Decomposition (build between each; the probe is the meter)

### A — phantoms
Register `:wat::kernel::Shared` and `:wat::kernel::Wire` as 0-param type paths
(type arguments only). Not values.

### B — `Address<S,R,T>` in the checker
- Thread listener → `Address<S,R,Shared>`.
- Process listener → `Address<S,R,Wire>`.
- Abstract Locus listener → T fresh.
- 2-arg `Address<S,R>` remains legal and means T unknown (unifies with both).
- `connect` still accepts any T.
- `address-wire?` still unifies `Address<S,R>` (2-arg) or `Address<S,R,T>`.
- `is_pure_type`: `Address<_,_,Shared>` impure; `Address<_,_,Wire>` pure;
  2-arg Address stays pure.

### C — T must survive `/start`
`Handle` becomes `Handle<T>` with `addr <- Address<Op,Reply,T>`.
`/start` must return `Handle<Shared>` when the locus is `ThreadOpts` and
`Handle<Wire>` when the locus is `ProcessOpts`.

Pinned shape: **defclause** with two kwargs clauses (same body, different
locus type + return). A third `[locus <- Locus]` residual is allowed (T unknown).

**STOP-1:** kwargs + defclause + different return types is unproven in the
stdlib (no `defclause` with `& [...]` exists under `wat/`). If it cannot be
expressed cleanly, STOP and report. Do not flatten start to positional. Do not
invent a type function. Do not special-case `*/start` by name in `check.rs`
unless you first STOP and the report says that is the only remaining door.

### D — the raise
`bracket::map` / `each`: when the locus AST's head is a ProcessOpts constructor
(`:wat::spawn::process`, `process/runner-count`, `process/post-spawn`,
`process/env`, `process/max-message-bytes`, `with-label` wrapping those), emit
an `ann-form` (or `require-wire-address`) so each kwargs address unifies against
`Address<?,?,Wire>`. A `Handle<Shared>` address is `Address<_,_,Shared>` →
TypeMismatch.

A symbol locus (`loc`) is a residual — do not guess. The acceptance test uses
`(:wat::spawn::process)`.

### E — ride the cascade
`cargo nextest run --release` / the compiler is the teacher. Apply the rule.
Do not weaken 2e. Do not weaken `probe_thread_kwargs_thread_svc` (legal:
thread map + thread handle).

## The farm is wat-fix (R21)

A third type argument is a corpus rewrite. Read `wat/fix.wat` (header
BOOTSTRAP / STASH-DANCE, then `rename-keyword-prefix` as the closest
vehicle) and copy a `wat-scripts/fixes/*.wat` (e.g.
`reclaim-ipc-prime-names.wat`, `retarget-peer-purity-probes.wat`) for the
script shape: `:user::migrate` + stdin path list + `write-file`.

Write `wat-scripts/fixes/address-transport-arity.wat`. It walks keyword
leaves whose name is a 2-arg `:wat::kernel::Address<…>` or
`:wat::spawn::Bound<…>` and appends `,wat::kernel::Wire` (comment-faithful
text edit, like rename-prefix). Idempotent: a 3-arg form is left alone.

**STASH-DANCE** — do not hand-edit the farm; do not skip this:

1. Add the fix verb / script first (checker still accepts 2-arg).
2. Stash `src/check.rs` `src/runtime.rs` (the rust that rejects 2-arg).
3. `cargo build --release`.
4. `printf '[…EVERY path…]\n' | cargo wat ./wat-scripts/fixes/address-transport-arity.wat`
5. Stash pop.
6. Rebuild. New checker; corpus is 3-arg.

Dry-run on a copy and `diff` first. Generate the path list; never type it.
Python/sed are forbidden. `tools/` Rust binaries are for Rust syntax only —
`.wat` is wat-fix's job.

## Discipline

- wat-rs only. No holon-rs.
- No new `Value` variant. Runtime `Address` entity unchanged.
- `.wat` corpus: wat-fix only. Declaration sites (`wat/spawn.wat` Bound,
  `wat/service.wat` Handle/addr-ty, `wat/fix.wat` if a new verb) may be
  authored; everything else is the recorded migration.
- Commit nothing.

## STOP

1. kwargs defclause cannot express the start split → STOP, report.
2. Tempted to make `connect` reject Shared everywhere → STOP (thread may dial thread).
3. Tempted to overload `peer-wire?` → STOP.
4. Abstract-Locus start is the only caller left failing → residual, not a prompt
   to erase T again.
5. 2e probe goes red → you broke the mouth. Fix before continuing.

## FM 2-bis

`tests/comms/probe_arc293_W2f_process_dials_thread.rs` —
`process_map_of_thread_handle_is_a_check_error`.
Pre-stone: `startup_from_file` is **Ok** (type-checks); `expect_err` panics.
Legal control (must stay GREEN): `probe_thread_kwargs_thread_svc`.

## Calibration

90–180 min. STOP at 240 min on STOP-1. Prior SCORE shape: `SCORE-STONE-293.W.2e.md`.

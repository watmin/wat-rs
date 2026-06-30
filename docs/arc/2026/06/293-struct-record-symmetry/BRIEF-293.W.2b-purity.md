# BRIEF — 293.W.2b: PURITY IS THE AXIS (the build)

> **Executor: one sonnet LEAF.** Orchestrator drew this + weighs the kill forced-clean. Work ONLY in `wat-rs/`,
> NEVER worktrees. Commit nothing — leave the tree green for the orchestrator to weigh + commit.

## The work (one paragraph)
The type system's wire wall is a **purity** wall: a value crosses an address-space boundary **iff it is pure** (holds
nothing but data) vs **impure** (holds a live resource — `Sender`/socket/closure). The uncommitted tree already carries
the structural skeleton of this under an earlier *movement-frame* name (`Mobility { Portable, Anchored }` /
`:wat::enum::Portable|Anchored`). Your job: **rename the skeleton to purity, rename the cause-word everywhere in one
change so no seam survives (`is_portable`→`is_pure`), fix the one mis-declaration the purity check exposed
(`:wat::kernel::Failure` is pure EDN data wrongly `defstruct` → `defrecord`), and drive the gate to 0.** This is NOT a
new design — the model is fully settled; you are renaming + finishing.

## Read in order (the design + the current state)
1. `293/AGGREGATE-MODEL.md § THE PURITY AXIS` — **the canonical model** (purity is the axis; the holder refines it:
   `Struct` permits impurity, `Record`/`Holon` guarantee purity; enums declare `:Pure`/`:Impure` directly).
2. `293/DESIGN-293.W § 293.W.2b` — the strike detail + the superseding banner (movement-frame → purity).
3. The uncommitted skeleton (already on disk, `git status`): `src/types.rs` (`Mobility` enum + `EnumDef.mobility` +
   `parse_defenum` marker + 4 builtin enums), `src/check.rs` (`is_portable_type` enum arm reads the declaration + the
   enum branch of `validate_aggregate_containment`), `src/types/error.rs` (the two containment error variants), and
   ~50 `.wat` fixtures + stdlib with `:wat::enum::Portable` markers already inserted.
4. Prior comparable strike: **293.W.1** (`ff29f135`, the aggregate containment gate) — same shape, one tier up.

## The decomposition (do in this order; build between each)

### A — rename the movement-frame → purity (the marker + the Rust enum)
- `Mobility { Portable, Anchored }` → **`Purity { Pure, Impure }`** in `src/types.rs` (mirrors `Holder`). Method
  `is_portable()` on it → **`is_pure()`** (`matches!(self, Purity::Pure)`). `from_marker_keyword` matches
  **`:wat::enum::Pure`** | **`:wat::enum::Impure`**. **STOP-1:** if a Rust type `Purity` already exists, name it
  `EnumPurity` and say so (do not collide).
- `EnumDef.mobility: Mobility` → **`EnumDef.purity: Purity`**; update all 5 `EnumDef` construction sites
  (4 builtins + `parse_defenum`). The builtins: `StepResult`/`WalkStep` → `Purity::Impure` (in-locus control);
  `ThreadDiedError`/`ProcessDiedError` → `Purity::Pure` (they cross as EDN death reports — VALID **once Failure is a
  record**, sub-task C).
- The error variant `NonPortableVariantFieldInPortableEnum` → **`ImpureVariantFieldInPureEnum`** (+ its `Display`);
  message reframed around purity (a `:Pure` enum may hold only pure variant fields; declare `:Impure` to hold a resource).
- The `.wat` markers: **`:wat::enum::Portable` → `:wat::enum::Pure`**, **`:wat::enum::Anchored` → `:wat::enum::Impure`**
  across all migrated stdlib + fixtures + the `service.wat` macro templates (a careful sweep; the gate verifies load).

### B — rename the cause everywhere (`is_portable` → `is_pure`)
- `Holder::is_portable()` → **`Holder::is_pure()`** (`types.rs:138`) + its 7 call sites.
- `is_portable_type` → **`is_pure_type`** (the fn + its 18 call sites). The "well-known portable scalar paths" comment →
  "pure scalar paths"; the non-portable opaque-path list (ChildHandle/IOReader/Sender/…) is the impure-resource list.
- The aggregate containment error `NonPortableFieldInPortableAggregate` → **`ImpureFieldInPureAggregate`** (+ `Display`,
  reframed). The W.1 pass `validate_aggregate_containment` keeps its name (it's the containment pass for both aggregates
  + enums) but its prose updates to purity. **STOP-2:** if "portable"/"Portable" appears in a context that is genuinely
  about something else (not this purity predicate), leave it + note it — do not blind-rename.

### C — `Failure: defstruct → defrecord` (the 2616-cascade ROOT)
- `:wat::kernel::Failure` is registered `holder: Holder::Struct` at `src/types.rs:~1038` but is pure EDN data (a panic/
  assertion report: `message: String`, `location: Option<Location>`, frames, actual/expected). Flip → **`Holder::Record`**.
- **Verify every Failure field is pure** (`is_pure_type` true). **STOP-3:** if any field is impure (a non-EDN handle),
  STOP and surface it — that's a deeper issue, not a blind flip.
- Update consumers that pattern-match `Holder::Struct && class == "wat::kernel::Failure"` — at least
  `src/test_runner.rs:667` (flip to `Holder::Record`); grep for others. `wat/doctest.wat:23`'s separate
  `:wat::doctest::Failure` record is unrelated — leave it.

### D — reshape the RED probe
- `tests/types/probe_arc293_W2b_enum_recursion.{rs,wat}` → rename to **`probe_arc293_W2b_enum_purity.{rs,wat}`**
  (the predicate reads a declaration, it does not recurse). Content: a `:wat::enum::Pure` enum declaring an **impure**
  (struct) variant field is REJECTED (containment); a `defenum` with **no marker** is REJECTED (mandatory); a record
  holding an **`:wat::enum::Impure`** enum is REJECTED. GREEN after this strike (NOT `#[ignore]`'d).

### E — drive the gate to 0 (the cascade triage)
Run `cargo nextest run --release` (WHOLE workspace). Each failure is exactly one of:
- **`ImpureVariantFieldInPureEnum :X`** containment error → the enum `:X` genuinely holds a resource → flip its marker
  to **`:wat::enum::Impure`** in its fixture/stdlib site.
- **"missing marker"** parse error → a `defenum` was missed → add the marker (`:Pure` default; `:Impure` if it holds a
  `Sender`/`Receiver`/socket/closure/struct).
- a **`_bad.wat`** fixture whose expected error CHANGED (it now hits the mandatory-marker error first) → add the marker
  so it reaches its INTENDED error, then confirm the probe's assertion still holds.
- **`deftest_svc_test_svc_assert_state`** (`wat-tests/service-template.wat`) — `:svc::Request` carries reply-`Sender`s
  → it is genuinely **`:wat::enum::Impure`**; its `make-channel` is a THREAD-tier channel that the 254.1 gate doesn't
  yet know is exempt → **THE ONE LEDGERED IGNORE:** flip `:svc::Request` to `:wat::enum::Impure`, then `#[ignore]` the
  test with the code marker **`// ⛔ IGNORE-LEDGER(293): 293.W.2d tier-aware make-channel — see CLOSE-SEQUENCE`**
  (the unlock is 2d; the row is already in `294/CLOSE-SEQUENCE § THE IGNORE LEDGER`). This is the **only** permitted
  `#[ignore]`.

## Blast radius (bounded)
`src/types.rs`, `src/check.rs`, `src/types/error.rs`, `src/test_runner.rs` + the `.wat` stdlib/fixtures already touched
(+ any the cascade reveals). NO new types beyond `Purity`. NO behavior change to the holder trit, the aggregate model,
or any non-purity code. The `is_portable`→`is_pure` rename is mechanical (a predicate's name).

## STOP triggers (numbered above): a Rust `Purity` collision (A/STOP-1); a "portable" that isn't this predicate
(B/STOP-2); an impure `Failure` field (C/STOP-3). On any STOP: halt, leave the tree building, surface the gap — do NOT
improvise a workaround.

## Gate + floor
`cargo nextest run --release` → **0 failures save the ONE ledgered `:svc::Request` ignore**. `cargo build --release`
clean (no new warnings beyond the pre-existing `all_match`/`head_span`). Read your own diffs end-to-end.

## EXPECTATIONS (the scorecard — fixed before the strike)
| what | command | expected |
|---|---|---|
| enum marker is purity | `grep -rn ':wat::enum::' wat/ wat-tests/ tests/ \| grep -c Pure` | > 0; **zero** `Portable`/`Anchored` remain |
| cause renamed | `grep -rn 'is_portable' src/ \| wc -l` | **0** (all → `is_pure`) |
| Failure is pure | load any program using `ThreadDiedError` | builds; no `ImpureFieldInPureEnum`/`NonPortable…` |
| probe GREEN | `cargo nextest run --release probe_arc293_W2b_enum_purity` | pass |
| floor | `cargo nextest run --release` | 0 failed save the 1 ledgered ignore; ~93+1 skipped |
| no content corruption | read diffs | only purity renames + the marker sweep + Failure flip; nothing else moved |

Runtime estimate: 30–60 min (mechanical rename + cascade triage). Trap-door: the `Failure` flip may ripple into wire/
reconstruction code beyond `test_runner.rs:667` — grep `Holder::Struct.*Failure` and `class == "wat::kernel::Failure"`.

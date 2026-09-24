# BRIEF — STONE 255.25 (C-b4): the markers are a `Transport`, and the child main says `Wire`

**Drawn 2026-09-24 against `main` @ `0006334ac`.** Floor 6058/6058, clippy 0, census 215 non-zero, delta
**NEW 2 / RECOVERY 0**, ledger 215. **Executor tier: Opus.**

## Ruled 2026-09-24 — the four questions, all YES

```wat
(:wat::core::defenum :wat::kernel::Transport :wat::enum::Pure
  :Shared []
  :Wire [])
```

References read `:wat::kernel::Transport.Shared` / `:wat::kernel::Transport.Wire`.

- **Family `Transport`:** the code already calls them *"phantom transport markers"* (`wat/spawn.wat:293`).
- **Variants `Shared`/`Wire`:** shared memory against serialised bytes. Every networked locus to come is
  `Wire`.
- **Namespace `:wat::kernel`:** beside `Address`, the type they parameterise.

Rejected: `InLocus`/`Portable` (Honest N: a `Shared` address is dialled across thread loci), a new word
(Obvious N), no family (Honest N), and `:wat::spawn` (Simple N: kernel's `Address` would depend on spawn).

## Why

`:wat::kernel::Shared` and `:wat::kernel::Wire` are declared as **`defstruct`s** (`wat/spawn.wat:297-298`),
while their own comment says *"Type arguments only. Not values."* A struct reads **impure**, so the
generated child main cannot spell its transport `Wire`. 255.17a measured it: 57 process-child tests red
at the self-peer purity wall. The free letter passed only through `is_pure_type`'s *"unknown ⇒ type
parameter ⇒ pure"* arm. With the markers declared as a closed **`Pure`** family, the child main can say
the truth.

## The work

1. **Declare the family.** Replace the two `defstruct`s at `wat/spawn.wat:297-298` with the `defenum`
   above. Keep the comment and correct it: *"phantom type-level members of a closed family; adding a
   transport is adding a variant (a language update)."*
2. **Respell every reference.**
   - `.wat`/`.wat.bad`: 37 lines in 20 files (census them per site first), by **wat-fix codemod**
     (dry-run, diff, apply with `./target/release/wat`, never `cargo wat`; commit it with a replay
     fixture).
   - `src/`: exactly **one place**, the four helpers `shared_marker`/`wire_marker`/`is_shared_marker`/
     `is_wire_marker` (`src/check.rs` ~:10526–10541), plus `:10591`. Every consumer goes through them;
     confirm by census.
   - `tests/*.rs`: 17 mentions.
   - The respelled Rust literals live in one place. ⚠ Compare markers **by denotation**, not raw string
     (`crate::edn::render::type_denotation`), so a faithful spelling of the variant is the same marker.
     This is CLAUDE.md's recurring class: a string compare with one side normalised.
3. **Purity: keep `(Address :- [S R Transport.Shared])` IMPURE.** It is an in-process resource.
   `is_pure_type`'s `Address` arm (`src/check.rs` ~:15025) keys on `is_shared_marker`. Keep it working on
   the new spelling. A row must show:
   - `(Address :- [… Transport.Shared])` is impure: refused as a `Record` field, or as a wire peer's
     payload;
   - `(Address :- [… Transport.Wire])` is pure;
   - both markers alone are pure.

   Moving this fact from a head-name arm to a declaration is C-b5's, not this stone's.
4. **The child main says `Wire`.** Apply the change 255.17a measured and reverted, saved at
   `scratchpad/255.17a-C-a.patch`: `status-ty-runtime` uses `handle-wire-tp-syms`. Update it to the new
   spelling. The self-peer purity wall must now **accept** `(Status :- [… Transport.Wire])`. The 57
   process-child tests are **this stone's rows**: they must stay green with the child saying `Wire`.
   Rewrite the `:2420-2428` comment to say what is now true.
5. **Measure, and do not land:** with 4 in place, re-apply the parked D2 patch (`scratchpad/park-255.17/`)
   and run the floor. Report whether the 57 stay green, and every red verbatim. Then revert the patch;
   D2 lands in C-b5.

## STOP triggers

1. The self-peer purity wall still refuses `(Status :- [… Transport.Wire])` → STOP. Report the arm
   verbatim.
2. Any consumer of the markers bypasses the four helpers (a raw string compare elsewhere) → report every
   site. Respell and route it through the helpers if there are 5 or fewer; if more, STOP.
3. A variant used as a type argument behaves differently from the struct it replaces anywhere in the
   floor → STOP, report it verbatim. (The family-assignability measurement is
   `wat-scripts/scratch-pad/255-17b-transport-family-shapes.wat`.)

## Expectations — fixed before the strike

| what | expected |
|---|---|
| `:wat::kernel::Shared`/`Wire` in live code | 0 (census; survivors named: replay fixtures, comments) |
| the purity rows | Address+Shared impure · Address+Wire pure · markers pure |
| the child main | spells `Transport.Wire`; the 57 process-child tests green |
| the D2 measurement | the Summary line, and every red verbatim; patch reverted after |
| floor · clippy · census · delta · ledger | green · 0 · `no STOP-8` against a pre-change census · NEW 2 / RECOVERY 0 · ≤ 215 |

Runtime prediction: 2–4 hours.

## Doctrine

- ⛔ **THERE IS NO KNOWN FLAKE. A RED IS A RED.** Capture the whole log, never re-run to green, name
  the arm. A red caused by the stone is fixed at its cause and the whole floor re-run; say which.
- ⭐ Prove every row can say BOTH words on a pre-stone build from this brief's commit. Capture `rc=$?`
  on the next statement.
- Line-pinned `.edn` goldens that move: recapture them with `UPDATE_EDN=1`, and show a `:line`-only diff.
- **If this brief contradicts the code, the code wins — say so plainly.** Commit locally with `git add --
  <paths>`; **do not push.** Spawn no subagents. If you stop without committing, revert your own edits
  and save the patch to the scratchpad; if you cannot, say so.

## Out of scope

Deleting the transport special cases and landing D2 (C-b5), moving Address+Shared impurity to a
declaration (C-b5), C-c, step 3.

---

## ADDENDUM — 2026-09-24, resume on `main` @ `8ec19bd24`

The first strike stopped at **STOP-3**, with nothing committed. A `Pure` enum variant in an `extend-type`
target was refused at stdlib startup (`EdgeFreeTypeName`), because variant singletons were registered in
a later pass than the edge wall. **255.25a closed it:** a variant is a type the moment its enum is
(`WEIGH-STONE-255.25a-…`). 255.26 then cured the debug-build startup panic (`WEIGH-STONE-255.26-…`).

**Resume from the saved work** in `scratchpad/s25/`:

- `255.25-stopped.patch` is the full tracked diff (29 files): the codemod applied, the `src/` helpers
  respelled as constants compared by denotation, `:10591` routed through them, 17 `tests/*.rs` mentions,
  the child main's `Wire`, and the `spawn.wat` comment.
- `untracked/transport-markers-to-family.wat` is the codemod: dry-run verified, idempotent, 34 lines out
  and 35 in across 19 files.

The first executor's census corrects the brief: **36 lines in 19 files**, not 37 in 20. Two recorded
migrations (`address-transport-arity.wat`, `unstamp-transport-wire.wat`) are survivors by design, because
a tool is never its own input.

**Measure whether the patch still applies** to `8ec19bd24`. `src/types.rs` moved in 255.25a and 255.26,
and `src/check.rs` did not. If it does not apply cleanly, **re-run the codemod** on the current tree and
re-apply the `src/`/`tests/` hunks by hand. Never splice a stale patch over moved code.

The rest of the brief stands: the purity rows, the child main saying `Wire` with the 57 process-child
tests as rows, the D2 measurement (applied, measured, reverted), a replay fixture for the codemod, and
all gates. ⚠ The release floor now reads 6065, and the delta baseline is NEW 2.

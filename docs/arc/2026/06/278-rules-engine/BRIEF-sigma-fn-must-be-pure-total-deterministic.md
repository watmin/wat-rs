# BRIEF — the sigma capability predates the axes; make it prove all three

## The gap

`(:wat::config::set-presence-sigma! f)` / `(:wat::config::set-coincident-sigma! f)` install a **user wat
function** that `presence?` and `coincident?` invoke to compute their floor
(`sigma(d)/sqrt(d)`). The only thing checked at install is
`check_sigma_fn_signature` (`src/freeze.rs:624-652`) — **arity and types only**: one param, `:i64 -> :i64`.

Nothing requires the installed function to be **pure**, **deterministic**, or **total**. So it may
`println`, read a clock, or raise — and `presence?`/`coincident?` are classified
`pure: true, deterministic: true` (`src/rete/purity.rs:372-373`) with the rete fence armed on
pure ∧ det **today**. Two verbs the fence has already certified can call arbitrary unchecked user code.

Builder: *"that's a catastrophic gap we must close… sigma must be made pure and total… it predates
either of those enforcements — we were sliding by on type checks — we need more than that."*

**Sharpened by `Encoders::presence_floor` / `coincident_floor` (`src/vm_registry.rs`):** the floor is
memoized in a `OnceLock`. A non-deterministic sigma therefore **latches whatever it returned on the
first call**, making the floor depend on evaluation order.

### ★ Why THREE axes and not the two named

Pure ∧ total would appear to exclude entropy. It does — **by accident, on an unmeasured default-deny
the source itself flags as provisional** (`purity.rs:159-165`):

```rust
":wat::core::Uuid/v4" => OpMeta { pure: true, deterministic: false, total: false }
// "…it is trivially total in the absolute sense but that claim was never measured."
```

`Uuid/v4` is genuinely pure and genuinely total. The day anyone measures its `total` honestly, a
pure ∧ total gate silently admits an entropy-seeded sigma. **Name determinism explicitly**, so the wall
does not rest on a placeholder.

## Read in order

1. `src/freeze.rs:436-520` — the two install sites and their surrounding `:init` block.
2. `src/freeze.rs:624-652` — `check_sigma_fn_signature`, the existing (insufficient) gate.
3. `src/sigma.rs:58-87` — `WatFnSigmaFn::sigma_at`, which `apply_function`s the installed fn.
4. `src/rete/purity.rs:758-790` — `is_pure_expr`, `is_deterministic_expr`, `is_total_expr`. All
   `pub(crate)`, all take `(&WatAST, &SymbolTable)`. `freeze.rs` is the same crate.
5. `src/rete/purity.rs:725-753` — `classify_fn`'s **`FunctionBody::Native` arm**. It consults
   `intrinsic_meta` per axis and default-denies an unproven native. **That is the pattern to mirror
   for STOP-1.**

## The work

Extend the install-time gate so an installed sigma fn must prove **pure ∧ deterministic ∧ total**.

- Add the check where `check_sigma_fn_signature` is already called — `freeze.rs:462` (presence) and
  `:497` (coincident). Both sites, same treatment.
- Prefer extending `check_sigma_fn_signature` itself (rename it if the name stops telling the truth —
  it will; a function called `..._signature` that also checks purity is a name that lies) over bolting
  a second call at two sites.
- On failure, `StartupError::SigmaFn` naming **which axis failed and the fn's path**, so the message
  teaches. Follow the register of the existing messages in that function.
- `FunctionBody::Wat(ast)` → classify the body on each axis.
- `FunctionBody::Native` → see STOP-1.

## Gates — run these, in this order, and report each Summary line

```
cargo build --release
cargo test --release --test lint
```

And a **RED-first probe**, because a gate that has never refused anything is a gate that proves nothing:

- Write a probe that installs a sigma fn violating **each** axis and asserts the startup is refused with
  the axis named. An impure one is easy (`println` in the body). Ground whether you can construct a
  non-deterministic and a non-total `:i64 -> :i64` from ops the classifier actually denies — **if an
  axis cannot be provoked, say so and say why**; do not fabricate a passing assertion for it.
- Assert the **converse**: a plain arithmetic sigma fn (e.g. `(fn [d] -> :i64 d)`) still installs
  cleanly. Without this the probe cannot distinguish "the gate works" from "the gate rejects everything."
- Confirm no existing corpus file installs a sigma fn that this newly refuses — grep
  `wat/ wat-tests/ tests/ wat-scripts/` for both setters and report what you find. **If a real corpus
  caller now fails, STOP and report it**; do not weaken the gate to accommodate it.

Do **not** run the full `cargo nextest run` — the orchestrator weighs the floor centrally, once.

## STOPs — rejection criteria, not permission slots

- **STOP-1 — the `FunctionBody::Native` case is a DECISION, not an assumption.** A native fn has no body
  to walk. The expected shape is to mirror `classify_fn`'s native arm: consult `intrinsic_meta` on each
  axis and **default-deny an unproven native**. If that does not fit at this site, stop and report
  rather than choosing a blanket allow or a blanket deny on your own judgement.
- **STOP-2 — do not touch the two Rust defaults.** `DefaultPresenceSigma` / `DefaultCoincidentSigma`
  (`sigma.rs:36-56`) are installed directly, not through the setter path, and are pure/total/deterministic
  by inspection. They are not in scope.
- **STOP-3 — do not weaken the gate to make an existing caller pass.** If the corpus has a sigma fn that
  fails one of the three axes, that is a finding: report it with the file and the failing axis.
- **STOP-4 — do not extend this to other `set-*!` config setters.** The class question (what else
  predates the purity/totality axes and is guarded by a type check alone) is real and is tracked
  separately. This strike is sigma only.

## Do not

Do not commit. Do not push. Do not stash. Do not revert anything you did not write.

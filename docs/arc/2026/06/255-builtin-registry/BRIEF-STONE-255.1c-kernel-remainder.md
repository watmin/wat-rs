# BRIEF — STONE 255.1c-kernel-remainder · HOME #8: the last thirteen

## You are a rider

You edit and report. **The orchestrator builds the floor, runs clippy, and commits.** Do not run
either; do not commit, push, stash, or revert. **Ending your turn ENDS you** — nothing wakes you. Run
everything in the FOREGROUND and block on it.

Anchor `/home/watmin/work/holon/wat-rs/`; `pwd` first. Any `.claude/worktrees/` path is harness state.

**Two commands are yours:**

```bash
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 cargo build --release
systemd-run --user --scope -q -p MemoryMax=8G -p MemorySwapMax=0 timeout 900 \
  cargo nextest run --release -E 'test(/intrinsic::tests::/)'
```

⚠ That filter cannot see `tests/diagnostics/`. Deleting fourteen arms shifts `src/runtime.rs` and five
line-pinned goldens will go red — expected, mine to ratify. **Report a scoped run, not a floor.**

## The work

Thirteen verbs — everything left under `:wat::kernel::` — into `src/intrinsic/kernel_remainder.rs`
as thin `#[wat_intrinsic]` wrappers around the same delegates; delete the literal arms. After this the
kernel tier's literal dispatch is **empty**.

⚠ **`serve-dispatch-op` has TWO arms** (`runtime.rs:4321` and `:5640`). Both go. That is fourteen arm
deletions for thirteen verbs.

## Read in order

1. `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-255.1c-kernel-remainder.md` — the stone.
2. `src/intrinsic/kernel_resource.rs` — home #7, the most recent shape and the largest. Copy it.
   Note how rows the gate cannot check say so in a `//` maintainer comment naming their `infer_*` arm.
3. The thirteen bodies. The design stone maps every arm; find each delegate and **read all thirteen
   before declaring anything**.
4. `wat/runtime-meta.wat` — `:ControlFlow`, `:CheckGate`, `:Probe`, `:Projection`, `:Reflection`
   prose. You will be editing two of these (below); read all five.

## ★★ THE HEADLINE — confirm it, then register it

`peer-pid` has **18 corpus call sites** and **zero mentions in `src/check.rs`**. No scheme, no
inference arm. It falls through to `check.rs:5561`'s *"silent-by-intent — no scheme found; accept and
pass"*, which returns a **fresh type variable** — args unchecked, arity unchecked.

**Verify this yourself before writing its row** (`grep -cF ':wat::kernel::peer-pid' src/check.rs`).
If it is nonzero, my measurement was wrong and I need to know — that is a finding, not a nuisance.

Registering it does not close the blanket-accept (task #110); it takes one verb out of its shadow.
Say so in the row's maintainer comment.

## Three rulings the taxonomy DEFERRED TO THIS CARVE — you make them, from the bodies

1. **`serve-dispatch-op`** — the taxonomy found *"no clean single-axis fit (dispatch + a
   crash-sentinel broadcast)"* and left it to the carve, recommending `:ControlFlow` with the
   broadcast noted as defensive plumbing. **Derive it; the recommendation is not a ruling.**
2. **`:ControlFlow`'s prose** must be strengthened for `raise!`/`assertion-failed!`: they never
   return — **they abandon evaluation rather than direct it.** One sentence, in `runtime-meta.wat`.
3. **`:CheckGate`'s prose LIES today.** It asserts *"One member today"* naming
   `require-wire-address`, which is not registered and carries no `@Category` — actual membership
   **zero**. Carving it makes the claim true; **fix the sentence so it describes the variant rather
   than asserting a count**, the way `:Probe`/`:Combine` honestly do.

## The axis table — RE-DERIVE, then agree or dissent with the deciding line

| verb | predicted | ⚠ |
|---|---|---|
| `raise!` `assertion-failed!` | ControlFlow | |
| `here` `call-site` `macro-call-site` `fn-forms` | Reflection | |
| `require-wire-address` | CheckGate | first real member |
| `peer-wire?` `address-wire?` | **Probe?** | would be `:Probe`'s FIRST tenants ever |
| `peer-pid` `peer-process` | **Projection?** | tests `:Projection` on a HANDLE, not a record |
| `serve-dispatch-op` `retag-op` | **UNRULED** | yours to derive |

★ **Do NOT file `peer-wire?`/`address-wire?` as `:Probe` because they end in `?`.** `:Probe`'s own
prose warns off exactly that: *"NOT 'returns a bool': `length` returns an i64 and belongs here.
Sorting by return type is the axis-mix that sank the proposed `:Predicate`."* Derive from the DOING.

★ **`:Projection` on a peer is a real question.** Its prose says *"returns a COMPONENT of a compound
value that was already there"*. A peer is an opaque handle, not a record. If `peer-pid` reads a stored
field, that is a projection; if it CALLS something to obtain a pid, it is not. **Read the body.**

## ★ THE STRAIN REPORT — the deliverable that matters most

Mark every one of the thirteen `LANDED` or `ARGUED`. **A verb that fits only after a paragraph of
justification is a FINDING** — write the paragraph AND say it took one. Two of these (`serve-dispatch-op`,
`retag-op`) the taxonomy already declined to rule; if either will not classify cleanly, **that is the
result**, and STOP-1 applies.

## Blast radius

```
NEW   src/intrinsic/kernel_remainder.rs
EDIT  src/intrinsic/mod.rs      one `mod` line
EDIT  src/runtime.rs            14 arm deletions (+ replacement comments); widen delegates
EDIT  wat/runtime-meta.wat      :ControlFlow prose + :CheckGate's false membership claim ONLY
```

No `check.rs`. **No stub schemes.** No other variant's prose.

## STOP triggers

1. **STOP-1 — a body's DOING fits no existing variant.** Report the verb, the deciding line, and what
   it actually does. **Do not file it anyway and DO NOT MINT A VARIANT** — the taxonomy is held
   pending precedent by builder ruling; your finding IS the precedent.
2. **STOP-2 — routing changed.** Registration moves the lookup, never the handler.
3. **STOP-3 — you need `check.rs`.** Including a stub scheme. Report and stop.
4. **STOP-4 — `peer-pid` turns out to be known to the checker.** Then my headline measurement was
   wrong. Report it; do not quietly proceed.
5. **STOP-5 — blast radius insufficient.** Name the file and why.

## What "done" looks like

- `cargo build --release` exits 0
- the scoped run's **full Summary line**, labelled scoped; any failure's whole block verbatim
- **the strain report** — thirteen rows, LANDED/ARGUED, deciding body line each
- your axis table with dissent where you have it, and your ruling on `serve-dispatch-op`/`retag-op`
- confirmation of the `peer-pid` measurement, either way
- `git status --short` **and `git diff --numstat src/runtime.rs`** for the goldens ratification
- the honest deltas

Runtime band: 55–85 minutes. Thirteen bodies; do not rush the reads.

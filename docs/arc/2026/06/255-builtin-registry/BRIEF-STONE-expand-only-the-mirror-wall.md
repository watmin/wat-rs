# BRIEF — STONE 2 of 2: the mirror wall

Refuse an `ExpandOnly` head found in program code, at expand time, beside the wall's existing half.
DESIGN: `docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-expand-only-the-mirror-wall.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
You may run the pre-existing `./target/release/wat` and `--check` freely; **that binary contains
stone 1 but not your changes**, which makes it exactly right for capturing BEFORE behaviour.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. The tree is clean and the
floor is green at 5111 — anything you break is yours.

## Read in order

1. The DESIGN above — especially **why no macro-body context is needed**. Probe A in that document
   is the load-bearing measurement; understand it before you design anything, because the obvious
   implementation is machinery for a state that cannot occur.
2. `src/macros/eval.rs:424-428` — `is_expand_time_legal`, **the wall's existing half**. Your work is
   its mirror and belongs beside it, in the same module.
3. `src/macros/expand.rs:23` (`expand_all`) and `:32` (`expand_all_with`) — the whole-program walk.
   `forms: Vec<WatAST>` is every top-level form in the program.
4. `src/macros/error.rs:95-99` — `MacroErrorKind::RefusedInMacro { head }`, the refusal the existing
   half emits. **Read STOP-1 before you reach for it.**
5. `wat/runtime-meta.wat`'s `ExpandTime` `defenum`, `:ExpandOnly` — the coordinate you are enforcing,
   and the sentence that says what it means.

## The work

### 1 — THE CONTROL, WRITTEN FIRST

Before the wall exists, write the probe that proves the legitimate case survives:
`macro-error` **inside** a `defmacro` body must still `--check` clean. Capture its BEFORE output with
the pre-existing binary and record it in the probe header.

⛔ **This is not paperwork and it is not step 3.** If the wall fires here, `macro-error` is dead at
its only legitimate call site and the stone is worse than not shipping. Writing the control first is
what makes the wall's green mean something.

### 2 — a refusal of its own

Add a `MacroErrorKind` variant for this refusal. Its message must name **the tier and where the verb
IS legal** — a reader who hits it should learn that the verb exists only inside a `defmacro` body,
not merely that they were refused.

### 3 — the wall

In the whole-program walk, refuse a call head whose registry entry declares
`ExpandTime::ExpandOnly`. Consult the registry the way `is_expand_time_legal` already does
(`registry().lookup_entry`) — `src/macros/` already holds that edge, which is why the wall lives here.

⚠ Per the DESIGN's probe: you do **not** need to ask whether you are inside a macro body. A
`defmacro` body is validated by the existing half and is not walked as program code. If you find
yourself threading an "am I in a macro body" flag, stop and re-read probe A.

### 4 — the probe

`wat-scripts/scratch-pad/255-the-mirror-wall.wat`, following the shape of the scratch-pad's other
probes. It must distinguish three cases, not one:

- `macro-error` inside a `defmacro` body — **legal, unchanged** (the control)
- `macro-error` in a `defn` body — **refused** (the target)
- a macro whose template **quotes** a `macro-error` call — **refused** (a defect made visible)

⚠ A committed `.wat` must LOAD. Anything that now fails to check is demonstrated out-of-tree and its
verbatim output recorded in the probe's header — the pattern waves 1–3's probes already use.

## Blast radius

`src/macros/error.rs` (one variant) · `src/macros/` (the wall, beside `is_expand_time_legal`) · the
new probe, plus whatever test file the refusal's own coverage belongs in. No verb bodies move. No
`.wat` corpus change. **No new registrations.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — do NOT reuse `RefusedInMacro`.** It is the OPPOSITE claim. That variant means *"this
verb may not be called INSIDE a macro body"*; yours means *"this verb may ONLY be called inside
one."* Reusing it would make the diagnostic lie to the reader in the exact direction that costs them
the most time. Mint a distinct variant.

**⛔ STOP-2 — the control governs.** If `macro-error` inside a `defmacro` body stops checking clean,
STOP. Do not adjust the control to match the wall; the control is the specification.

**STOP-3 — no new dependency edge.** `grep -c "crate::intrinsic" src/check.rs` must remain **0**.
The site was chosen because `src/macros/` already holds that edge and `check.rs` does not; adding one
there creates a `check → intrinsic → check` cycle that a ruled crate migration has to pay for. If the
wall seems to want to live in `check.rs`, STOP and report why — that is a finding about the design,
not a licence to move it.

**STOP-4 — one verb, not a census.** `macro-error` is the only `ExpandOnly` declarer (measured). Do
not re-declare any other verb's `@ExpandTime` to make the wall look better exercised. A second
candidate is a finding to report.

**STOP-5 — `macros/eval.rs:495` is not yours.** The 58-name residue still lists
`:wat::core::macro-error` although it IS registered, so that row is unreachable dead text. Real, known,
and out of scope. Leave it; say so in your report if you touch anything near it.

## Report

Per-file diff summary; the variant you minted and its message text in full; **the control's
before/after, first** — then the target's and the quoted-template case's; where you put the wall and
why that site rather than the walk's other candidates; confirmation that `check.rs` still has zero
`crate::intrinsic` references; and what surprised you — a walk that did not reach where you expected,
a form kind the DESIGN did not name, or a case where the control and the target could not be
separated.

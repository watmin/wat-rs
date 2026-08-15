# BRIEF — 198: a diagnostic span points at the OFFENCE, not at its container

> Builder ruling, 2026-08-15: *"we do the correct thing - make the system correct - we have
> identified incorrectness, annihilate it."*
>
> Read `DESIGN-STONE-a-restriction-governs-mention-not-head-position.md` first. Baseline: HEAD
> `f5700101` **plus the uncommitted Wave-B1 work** (25 goldens + 26 lifted ignores, 1 test red — that
> red is what this brief closes). Floor **4560 run / 4559 passed / 1 failed / 128 skipped**, clippy 0.

## THE INCORRECTNESS

`8f0e3939` (the mention rule) changed where `DefRestrictedCallerNotAllowed` points — but **only on the
macro-expanded path.** Measured, same file, same run, same error kind:

| call | path | span | covers |
|---|---|---|---|
| `(:my::Vault/secret v)` | direct head | `col 4–21` | `:my::Vault/secret` — **the name** |
| `struct_restricted_empty_sections` case C | direct head | `col 4–20` | **the name** |
| `(:my::Token 7)` | **companion macro** | `col 3–17` | `(:my::Token 7)` — **the whole form** |

The same diagnostic now anchors two different ways depending on whether a macro was in the path.

## THE RULE — stated three times in the record, and it decides this

| arc | rule |
|---|---|
| **233** (`EXPECTATIONS-STONE-233.2.l.md:39`) | *"Error message span must point at the offending **VARIANT**, not the whole enum"* |
| **170** (`BRIEF-SLICE-3-GAP-J…:233`) | *"the span should point to the actual **decl**, not the enclosing `do`"* |
| **167** (`EXPECTATIONS-SLICE-2.md:59`) | *"[via] macro expansion, the error span should point at **the user's source**, not the macro expansion's synthetic fn-form"* |

233 and 170 say the same thing twice: **anchor to the offending thing, never to the container holding
it.** 167 adds the macro-specific half: never anchor to synthetic expansion output.

The code agrees with itself — in `src/check.rs`, `head_span` anchors **115** error pushes; the
whole-form `span` anchors **16**.

**The offence here is naming `:my::Token` from outside its whitelist.** The narrowest thing that IS
that offence is `:my::Token`. `(:my::Token 7)` is its container.

## THE MECHANISM — mapped, but CONFIRM IT BEFORE YOU CHANGE IT

1. `src/macros/parse.rs:329` (`aggregate_kwargs_companion_source`) — the synthesized companion macro
   builds its type keyword from a **string**: `(:wat::core::keyword-node "{bare_name}")`. The original
   node the user wrote is not carried through.
2. `src/edn_shim.rs:1133` — `eval_keyword_node` emits
   `WatAST::Keyword(s, crate::rust_caller_span!())` — a **Rust** span (`edn_shim.rs` itself).
3. `src/macros/expand.rs:851` — `restamp_unknown_spans(form, call_site)` replaces that synthetic span
   with the **macro call site**: the whole `(:my::Token 7)` form.

**`restamp_unknown_spans` is doing arc 167's job** — it is what stops the diagnostic pointing at
`edn_shim.rs`. **Do not remove or weaken it.** Its defect is granularity: it has exactly one span for
every emitted node, so a node that corresponds to a *specific piece* of user source loses that
correspondence.

⛔ **Verify this chain by measurement before changing anything** — `macroexpand` the construction and
read the emitted form's spans (`:wat::core::macroexpand` / `macroexpand-1` exist as forms;
`src/special_forms.rs:247`). The chain above is the orchestrator's read; a rider on this arc has found
a defect in every brief so far.

## ⛔ THE OLD GOLDEN IS NOT THE TARGET — a correction worth having

The pre-existing literal says `col 4–14`. That value was captured in the **pre-293 world**, when the
constructor was `:my::Token/new` and the restriction fired on a **direct call head**. It is not
evidence of what today's form should report.

**The target is: the span of the type keyword as the user wrote it in today's form.** In
`  (:my::Token 7))` that is `col 4–14` — the same columns, arrived at for a different reason. Do not
reason "restore the old number"; reason "point at `:my::Token`", and check the number falls out.

## THE GATE

1. **The three restriction diagnostics agree.** After the fix, all three anchor to the restricted
   NAME. Show the spans for all three, not just the one you changed.
2. **The red test goes green.** `struct_restricted::struct_restricted_ctor_restriction_fires_on_illegal_caller`
   — adjudicate the new face, then convert to an `.edn` golden with `wat::assert_edn_matches_file!`
   and capture. It is the 26th of Wave B1 and the only one left uncaptured.
3. **Negative control.** Show the span is wrong BEFORE your change (`col 3–17`) and right after
   (`col 4–14`), from an actual run, both directions.
4. **167 still holds** — no diagnostic anchors to `edn_shim.rs`, `parse.rs`, or any synthetic form.
   Prove `restamp_unknown_spans` still does its job for a node that has no user-source counterpart.
5. **The floor**: `4560 / 4560 passed / 128 skipped`. The 25 already-captured goldens must not move —
   if any does, that is STOP-2.

## WRITE THE RULE WHERE THE CODE IS

The convention lives in three arcs' **expectations documents** and nowhere in the code it governs.
That is why a change could violate it silently. Record it in the doc comment of whatever you touch —
`restamp_unknown_spans` and/or `walk_for_restricted_call` — in one sentence: *a diagnostic span
anchors to the narrowest user-source node that IS the offence, never to its enclosing form and never
to synthetic expansion output*, citing 233 / 170 / 167.

## STOP TRIGGERS

- **STOP-1 — the mechanism is not what this brief describes.** Report what it actually is; do not
  proceed on my map.
- **STOP-2 — any of the 25 already-captured goldens moves.** This change is meant to affect the
  macro-expanded restriction path only. A wider blast radius is a finding.
- **STOP-3 — the fix requires weakening `restamp_unknown_spans`** such that some node ends up pointing
  at Rust source. That violates 167; report instead.
- **STOP-4 — you are tempted to change the test's expectation to match the code** rather than the code
  to match the rule. That is the inversion this whole strike exists to refuse.

## BLAST RADIUS

`src/macros/expand.rs` and/or `src/macros/parse.rs` and/or `src/edn_shim.rs` (whichever the
measurement names), `tests/types/struct_restricted.rs` + its new `.edn` golden. **No `.wat` corpus
changes.** Do not touch the 7 out-of-scope ignored tests or the 25 captured goldens.

## VERIFY

`cargo build --release --tests`, then `cargo clippy --workspace --all-targets --release -- -D warnings`
(0), then `scripts/floor.sh` — read the **Summary line**, never a piped exit code. Expect
`1 failed → 0 failed` at `4560`. Report the arithmetic.

**On any red you did not intend: do NOT re-run.** Copy the whole stdout+stderr block **verbatim** —
never a `| head` window — name the exact assertion, and report.

## HOW TO WORK

You are a rider. **Ending your turn ENDS you.** ⛔ **Run every build and test in the FOREGROUND and
block on it. Do NOT use `run_in_background`. Do NOT set a monitor. Do NOT poll and stop.** THREE
riders on this arc have died exactly that way and the orchestrator had to recover their runs. If you
find yourself about to wait for a notification, you are about to die — run the command in the
foreground instead.

Anchor at `/home/watmin/work/holon/wat-rs`; `pwd` first. **The working tree holds uncommitted Wave-B1
work (25 goldens, 26 lifted ignores) — do not revert it, do not re-ignore anything.** Leave your work
uncommitted. Never `git commit`/`push`/`stash`/`revert`/`checkout --`; `stash@{0}` holds unrelated work.

## REPORT

- the measured mechanism, and where it differed from this brief's map
- the spans of **all three** restriction diagnostics, before and after
- the negative control both directions
- the 26th test's adjudication and its captured golden
- the floor Summary line verbatim with the arithmetic
- every STOP that fired
- **the honest deltas — especially anywhere this brief did not match the disk.**

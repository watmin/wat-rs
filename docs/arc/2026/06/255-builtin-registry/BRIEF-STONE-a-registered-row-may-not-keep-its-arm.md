# BRIEF — STONE: a registered row may not keep its dispatch arm

Delete five dead arms and build the wall that stops the sixth. DESIGN:
`docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-a-registered-row-may-not-keep-its-arm.md`.

You are a rider. **Ending your turn ENDS you** — nothing wakes you, no notification is coming. Make
text edits and report; your turn ends when your report is written. The orchestrator builds, floors
and clippies centrally — you do not run `cargo build`/`test`/`nextest`/`clippy` or `scripts/floor.sh`.
**You may not spawn sub-agents.** Work only in `/home/john/work/holon/wat-rs`; verify with `pwd`
first. Do not commit, push, stash, revert, or `git checkout --` anything. Tree clean, floor green at
5118, HEAD `f4858bb1e`.

## Read in order

1. The DESIGN, whole — especially § "What is actually dead" and its two recorded wrong measurements.
   **Both wrong answers are the ones you will reach if you measure the obvious way.**
2. **`src/runtime.rs`'s `dispatch_keyword_head_value`** — find where
   `crate::intrinsic::registry().lookup(head)` sits inside it. That door is why the five arms below
   are unreachable, and why arms in `eval_tail`/`step_list` are not.
3. `src/intrinsic/mod.rs`'s existing gates — `checker_skip_debt_is_named_and_frozen` and
   `registry_membership_gap_a_is_named_and_frozen` — for the house shape.

## The work

### 1 — delete five dead arms, and ONLY the arm lines

```
src/runtime.rs:1975   ":wat::core::record?"        => crate::record::access::eval_record_q(...)
src/runtime.rs:2321   ":wat::core::u8"             => crate::numeric::convert::eval_u8_cast(...)
src/runtime.rs:2377   ":wat::core::bool::to-string"=> eval_bool_to_string(...)
src/runtime.rs:2427   ":wat::core::not"            => eval_not(...)
src/runtime.rs:2670   ":wat::core::show"           => eval_show(...)
```

All five are inside `dispatch_keyword_head_value`, all five carry a registry row with a handler, so
the registry-first door answers first and the arm can never be reached.

⛔ **Delete the arm line. Nothing else.** Each is followed by comment blocks recording what OTHER
arms were retired here and why (arc 255 Stones C/D, arc 237). Those are the record; they stay. Leave
a one-line retirement note at each cut in the shape the surrounding comments already use.

⚠ The handler functions themselves (`eval_not`, `eval_show`, `eval_bool_to_string`,
`crate::numeric::convert::eval_u8_cast`, `crate::record::access::eval_record_q`) **stay** — the
registry dispatches to them. You are deleting a dead *route*, not a body.

### 2 — the gate

A `#[test]` asserting: **no registered row that carries a handler has a literal dispatch arm inside
`dispatch_keyword_head_value`.**

The predicate needs BOTH halves:

```
entry.handler.is_some()                              ← Kind::SpecialForm rows have None
∧  the arm lies within dispatch_keyword_head_value    ← not eval_tail, not step_list
```

Read `runtime.rs` with `include_str!` (the probe at `probe_can_doc_types_reconstruct_the_checker_scheme`
already does this — copy the technique), bound the function's span from its `fn` line to the next
top-level `fn`, and search only inside it. The failure message names each offending row and says to
delete its arm.

## Blast radius

`src/runtime.rs` (five lines out, five retirement notes in) · `src/intrinsic/mod.rs` (one gate) ·
whatever the compiler names. No `.wat` corpus change. No registrations. **No verb changes behaviour
— every one of the five is already reached through the registry today.**

## STOP triggers — each REJECTS; ship nothing further on that point and report

**⛔ STOP-1 — `fn`, `if`, `let` and `match` HAVE LIVE ARMS IN THE SAME MATCH. DO NOT DELETE THEM.**
They are registered, they sit in `dispatch_keyword_head_value`, and they look exactly like the five.
They are `Kind::SpecialForm` with `handler: None`, so the registry door **skips them** and the arm is
the only dispatch they have. Deleting them breaks the language's core syntax. **This is why the gate
must test `handler.is_some()` and not a name list.**

**⛔ STOP-2 — DO NOT EXEMPT THOSE FOUR BY NAME.** A hand-list rots the moment a fifth special form
registers. Derive the exemption from `handler.is_some()` on the row itself. If you find yourself
writing `":wat::core::fn"` into the gate, stop — the predicate is wrong.

**⛔ STOP-3 — `eval_tail` AND `step_list` ARE NOT THIS MATCH.** `runtime.rs` has three matches that
dispatch on these names. `:wat::core::u8` has an arm in `dispatch_keyword_head_value` (dead, delete)
**and** one in `step_list` (live, keep). A gate or a sweep that does not bound itself to the one
function is measuring the wrong population — the DESIGN records a probe that made exactly this error
and called `:wat::holon::Blend` dead.

**⛔ STOP-4 — THE HISTORICAL COMMENTS STAY.** The ~60 lines around these arms record which arms were
retired in which stone. Deleting them is revisionism, and a future reader with no record of the
retirement would re-add the arms.

**⛔ STOP-5 — THE HANDLER FUNCTIONS STAY.** You are deleting five match arms, not five functions.

**STOP-6 — the gate must be able to fail, and must NOT fire on a special form.** Two properties, two
sabotages: re-add one deleted arm → red naming that row; confirm `fn`'s arm present → still green.
⚠ You cannot run them. Report both as **unverified reasoning**, explicitly, as the last three riders
correctly did. The orchestrator executes them.

## Report

Per-file diff summary; the gate verbatim; **how it bounds `dispatch_keyword_head_value`'s span** and
why that bound is right; confirmation that the four `Kind::SpecialForm` arms are untouched and that
the gate exempts them by `handler.is_some()` and not by name; the `step_list`/`eval_tail` arms you
left alone; your STOP-6 sabotage reasoning for both properties. Then: **what surprised you** — a
sixth dead arm the DESIGN did not name, a row whose handler status you could not determine, or a span
boundary that turned out to be ambiguous.

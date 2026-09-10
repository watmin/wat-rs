# BRIEF — STONE ⑤-A: `:wat::program::self-peer` becomes a special form

Group **A** of stone ⑤. **Read `DESIGN-STONE-5a-self-peer-is-a-special-form.md` first, INCLUDING
its ⛔ CORRECTED section** — the first version of that design was wrong about the macro and the
correction is the part that decides your diff.

## The work

`wat/bracket.wat` — the STDLIB — calls `:wat::program::self-peer` at three sites. The verb has a
runtime arm and a LITERAL inference arm in the checker, and **no `TypeScheme` and no registry row**.
Its contract lives in a `match` arm nothing can enumerate, which is why stone ③'s fold (484 names)
does not reach it. Register it as a SPECIAL FORM, pointing at the two implementations that already
exist.

## Read in order

1. `src/runtime.rs:11014` — `eval_program_self_peer`. **Its signature changes** (below).
2. `src/runtime.rs:2381` — its ONE caller, inside `dispatch_keyword_head_value`, whose own
   parameters are `(head, args, list_span, env, sym)`. Both new arguments are already in scope.
3. `src/check.rs:10117` — `infer_program_self_peer`. `role = check` emits source only, so this
   signature is **unconstrained**. Do not touch it.
4. `src/intrinsic/special/macroexpand.rs` — the model for a `#[wat_special_form]` + doc block whose
   module doc records what was MEASURED. Match that register.
5. `src/runtime.rs:11108–11118` — `:wat::runtime::argv`'s doc block. The AXIS PRECEDENT: same
   ambient-reading shape, four of five axes come from it.

## The four edits

**① Widen the eval fn to the canonical `NativeHandler` shape.** The `role = eval` shim generates
`#fn_ident(args, list_span, env, sym)`; the fn currently takes two.

```rust
fn eval_program_self_peer(
    args: &[WatAST],
    list_span: &Span,
    _env: &Environment,
    _sym: &SymbolTable,
) -> Result<Value, EvalBreak> {
```

Body unchanged — it needs neither. Update the one call site at `runtime.rs:2381` to pass them.

**② `#[wat_special_form_impl(":wat::program::self-peer", role = eval)]`** on that fn.

**③ `#[wat_special_form_impl(":wat::program::self-peer", role = check)]`** on
`infer_program_self_peer`.

**④ The declaration** — a new `src/intrinsic/special/program_self_peer.rs`: a unit struct with
`#[wat_special_form(":wat::program::self-peer")]` and the doc contract below, plus one alphabetical
`mod` line in `src/intrinsic/special/mod.rs`.

## The axes — RULED. Transcribe the grounds; do not re-derive them.

```
@Category      Ambient       reads `services::current_self_peer()` — a runtime ambient, not a
                             fact about a value the caller holds. `:wat::runtime::argv`'s ground.
@Purity        Pure          reads the ambient; no mutation, no I/O. It does NOT create the peer —
                             `current_self_peer()` RETURNS an existing one. argv's ground.
@Determinism   Deterministic installed once per locus, read many; within a locus it does not move.
@Totality      Partial       ⭐ MEASURED, not inherited. Outside a spawned locus it RAISES:
                             `MalformedForm — "no self-peer — (:wat::program::self-peer) is only
                             valid inside a spawned process service; root has no owner-link"`,
                             exit 1. argv leaves this `Unreviewed`; here the path was read AND run.
@ExpandTime    RuntimeOnly   `:RuntimeOnly`'s own definition — "needs state that does not exist yet
                             at expand time" — describes this verb literally.
```

★ Two of the five are STRONGER than argv's. Say so in the doc block: copying `Unreviewed` forward
would have been copying a deferral.

`@arg` is OPTIONAL for a special form (`:wat::core::let` carries `@syntax` + `@ret` and no `@arg`).
Both of self-peer's arguments are TYPE-shaped — `eval_program_self_peer` gates them on
`is_type_arg_shaped` — so use `@syntax`.

`@example` must be **`@example-norun`**, and its reason is the measured raise above: the verb cannot
execute outside a spawned process service, so no inline example can run.
`[[NOTE-a-norun-example-asserts-nothing]]` stands — the reason must name the measurement, not
gesture at difficulty.

## Blast radius

`src/runtime.rs` (one signature + one call site + one attribute) · `src/check.rs` (one attribute) ·
one new `src/intrinsic/special/program_self_peer.rs` · one `mod` line. **No body changes. No
`TypeScheme`. No change to `walk.rs` — the blanket is a later stone.**

## Acceptance

```
cargo build --release
(:wat::runtime::metadata-of :wat::program::self-peer)     now ANSWERS   (was silent)
(:wat::core::render-doc     :wat::program::self-peer)     now ANSWERS   (was silent)
cargo nextest run --release -E 'test(every_special_form_carries_check_and_eval_impls)'   PASS
cargo nextest run --release -E 'test(probe_arc209) or test(probe_arc272)'                unchanged
```

★ The third is the wall that refused stone ③a-i. It passes here only because both role pointers are
real. If it fires, read it — it names the missing role.

## STOP triggers — each is a REJECTION. Ship nothing; report.

**STOP-1.** If widening the eval fn breaks a caller you did not expect — STOP and name it. The
design measured exactly ONE call site; a second means the design's census was wrong.

**STOP-2.** If you find yourself giving `self-peer` a `TypeScheme` — STOP. It has an inference arm;
a scheme would be a second authority for one question.

**STOP-3.** If `every_special_form_carries_check_and_eval_impls` still fires after both attributes
are in — STOP with its verbatim message. That is the ③a-i wall and it means a role is not landing.

**STOP-4.** If an axis looks wrong once you are in the code — STOP and say so WITH EVIDENCE. Two of
these five were measured specifically because the precedent had deferred them; a third correction is
welcome, a silent substitution is not.

**STOP-5.** If the `@example` could in fact be made runnable — STOP and report how. `-norun` is
justified by a measurement, and a measurement can be wrong.

## Tier

You edit and report. Run the acceptance commands, foreground, and nothing else. **Do NOT run
`scripts/floor.sh` or `cargo clippy`** — the orchestrator runs those centrally, once, on a quiescent
tree. **Do NOT commit.**

Work in `/home/john/work/holon/wat-rs`; verify with `pwd` first. Any path containing
`.claude/worktrees/` is harness state and must not be operated on.

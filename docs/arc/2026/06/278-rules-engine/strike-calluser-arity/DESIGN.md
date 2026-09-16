# DESIGN-STONE — there is no such thing as an argument without a parameter

> **Origin (2026-08-31).** Class D3 of `VIGILIA-2026-08-30-WORK-LIST.md`, found by `struere`.
> Driven here at HEAD `4a77fa915`. **All three arms reproduce, and the first one is a silent
> wrong answer through the public surface.**

## Why

`exec_program_on` (`expr_ir/eval.rs:405-414`) fills the callee's frame:

```rust
for (i, v) in args.iter().enumerate() {
    if let Some(&slot) = program.params.get(i) {
        let idx = slot as usize;
        if idx < inner.len() { inner[idx] = Some(v.clone()); }
    } else if i < inner.len() {
        inner[i] = Some(v.clone());          // ← an argument with NO parameter, written by INDEX
    }
}
```

Nothing compares `args.len()` to `program.params.len()`. The `else if` gives a surplus argument a
meaning it must not have: it lands in the slot whose number happens to equal its argument
position.

### Driven — three arms, three different wrong answers

Fixture `:exp::cool`, whose `where` fence is `(?c < 20)` over `Temp{10}`, `Temp{30}`. Untampered
import fires **1** hit. The fence prog's root is replaced with a `:user` call:

| arm | tamper | result |
|---|---|---|
| **1 — surplus collides with a declared param slot** | 1 param at slot **1**, args `[10, 30]` | **ACCEPTED, hits = 0.** `inner[1]=10`, then the surplus `30` overwrites it at `i=1`; `30 < 20` is false, the fence rejects everything. Had the surplus been ignored: 2. Had arity been checked: refusal |
| **2 — surplus past the frame** | 1 param at slot **0**, `frame_len=1`, args `[10, 30]` | **ACCEPTED, hits = 2.** `i=1` is not `< inner.len()`, so the argument is **silently dropped** |
| **3 — missing argument** | 1 param at slot 1, **zero** args | refused as `#wat.runtime/UnboundSymbol {:message "unbound symbol: slot 1"}` — a diagnostic naming a **compiler-internal slot index**, with a span pointing at the *caller's* wat line. No arity, no callee named |

Arm 1 is the one that matters: **the engine returned a different answer, silently, from wire
input.** Arm 2 is the same defect answering the opposite way. Arm 3 is the same missing check
surfacing as a diagnostic about an internal number.

## ⛔ Class A again, and the lowering does not save it

`lower_expr` (`expr_ir/mod.rs:627-628`) builds `Expr::CallUser { program, args }` from
`lower_args(&items[1..])` and `lower_rete_defn(...)` **without comparing them**. The invariant is
held by the type checker — one door — and assumed by the lowering and by the wire. That is A1's,
A2's, A4's and A6's shape for the fifth time.

## ★ THE ONE CONTRACT DECISION

**A length mismatch is refused at `exec_program_on` — the one place where `args` and `params`
meet — and the surplus branch is DELETED.** Not bounds-checked, not made safe: deleted. An
argument with no parameter has no meaning to give it.

**The check goes there and nowhere else, and that is the point.** A wall at the import door would
be a second copy of an invariant the executor still would not hold — which is precisely the
failure this arc has now found five times. `exec_program_on` is downstream of *every* door: the
wire, the lowering, and `exec_foldl`. Put it where the two quantities meet and there is no other
door to assume anything.

## The algorithm

1. `exec_program_on` refuses when `args.len() != program.params.len()`, with
   `RuntimeErrorKind::ArityMismatch` — the kind `exec_foldl` (`:433`) already uses for the same
   class, naming the callee and both counts.
2. Delete the `else if i < inner.len()` branch. With the length equal, `params.get(i)` is always
   `Some`, so the branch is unreachable by construction rather than by discipline.

## ⚠ The one exemption to verify, not assume

`exec`'s `Expr::CallUser` arm has an `args.is_empty()` short-circuit commented *"Literal fn value
— foldl applies it via exec_foldl"*. Read against the disk: `exec_foldl` (`:403` in its body)
always applies `&[acc, x]` — two args to a two-param lambda — and the HOF path that merely
*extracts* a program without running it is `callee_program` (`:456`), a different function. So the
empty-args branch is an allocation shortcut for a genuine **zero-parameter** call, which the new
check passes.

**If that reading is wrong, a green test will go red.** That is the signal, and it must be
reported rather than special-cased: an exemption carved to keep a test green would re-open exactly
the branch this strike deletes.

## Blast radius

`src/rete/expr_ir/eval.rs` and `tests/rete/probe_arc278_export.rs`. The fixture mouth the probes
need — `:user::import-and-hits` (Export → hit count, the missing observation point; `import-hits`
builds its own export internally and `import-one` stops at the Session) — **is already added** to
`tests/rete/probe_arc278_export.wat`.

## Out of scope — AFFIRMATIVELY CUT

- **A sixth import wall.** Rejected above, by the ★ decision, for a stated reason. Do not add one.
- **Fixing the `UnboundSymbol` diagnostic's wording.** Arm 3 stops being reachable once arity is
  checked; the message only ever appeared because the check was missing. If it survives the fix,
  that is a finding.
- **`EXEC_SP` / the `RefMut` span (D4).** The neighbouring finding in the same function's module.
  Its own strike.

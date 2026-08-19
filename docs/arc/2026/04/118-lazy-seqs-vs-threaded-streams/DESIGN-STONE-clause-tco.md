# DESIGN STONE — clause heads get TCO. A language capability, not a stream fix.

**Builder, 2026-08-18, on being told a `defclause` head does not tail-call: *"that is very, very
wrong…. how do we add TCO to clauses, now?"*** — the correct response to a missing feature being
described as a fact of the language.

## The defect

`eval_tail` (`src/runtime.rs` ~4334) emits a tail call for exactly one kind of head:

```rust
other if sym.has_function(other) => emit_tail_call(func, args, env, sym, list_span)   // plain defn
_ => eval_inner(ast, env, sym)...                                                     // EVERYTHING ELSE
```

A `defclause` head resolves to a `ClauseSet`, not an entry in `sym.functions`, so it lands in `_`
and **recurses on the real stack.** `if` · `match` · `let` · `do` · `and` · `or` · `ann-form` all
have tail-aware siblings (task #59 added the last three). **Clause heads were never given one.**

MEASURED — two programs, byte-identical bodies, a 200,000-element Stream:

```
recursing into a `defclause` head   →  SIGSEGV (stack exhausted)
recursing into a plain `defn`       →  completes, 19999900000
```

It is silent: a `debug_assert` cannot catch it and the floor never has, because nothing in 4714
tests recurses deeply through a clause head. Same silent-SIGSEGV class as tasks #58/#86.

⚠ **Scope of the blast, stated honestly: this is not a stream defect.** Every `defclause` in wat —
`reduce`, `into`, `conj`, the arithmetic family, every user-written multi-arity verb — is
non-tail-recursive today. The stream tier is merely where it finally got measured.

## ★★ THE TRAP THAT DECIDES THE DESIGN — `:ensure` cannot survive a tail call

`eval_call_to_defclause_with_vals`'s own doc (`runtime.rs:8364`):

> *"Stone 237.3: extended with `:guard` evaluation (before body) and `:ensure` post-condition check
> (after body)."*

**A tail call abandons the calling frame by definition. There is no frame left to run an `:ensure`
post-condition in.** So:

- **`:guard`** runs BEFORE the body → it is part of *selection* → unaffected. Fine.
- **`:ensure`** runs AFTER the body → **a clause carrying `:ensure` MUST NOT be tail-called.**

The stone therefore tail-calls **only** clauses with no `:ensure`, and that exclusion is
**structural and explicit**, never incidental. A silent tail-call of an ensure-bearing clause would
delete a post-condition the author wrote and the checker promised — a correctness hole far worse
than the stack one being fixed.

## ⛔ CORRECTION 2026-08-18 — the first plan was WRONG. This is the real one.

**As drawn, this section said "one arm plus a seam — the pieces already exist," proposing that
`eval_tail` select a clause and hand it to `emit_tail_call`. That cannot work**, and the reason is
the same mistake as the claim this stone exists to fix: I read two function signatures and inferred
a design without checking the type that flows between them.

```rust
EvalSignal::TailCall { func: Arc<Function>, args, call_span }      // the signal carries a FUNCTION
pub struct Clause { args: ArgSpec, guard, ensure_fn, body: Arc<WatAST> }   // a clause is NOT one
```

A clause is a body evaluated in a scope built per call. `emit_tail_call` has no way to carry it, and
`apply_function`'s trampoline has no way to resume it. Routing a clause into that signal would
require either a second signal variant or a second trampoline in the clause dispatcher.

### The real fix — synthesize the Function at REGISTRATION, not at dispatch

The shapes ARE compatible: `Clause.args` is an `ArgSpec` whose `fixed_params` are `Identifier`
binders, which is exactly what `Function` wants.

```
Function { name, type_params, params, param_types, ret_type,
           rest_param, rest_param_type, body, closed_env, rete }   (value/environment.rs:46)
Clause   { args: ArgSpec, return_type, guard, ensure_fn, body }    (value/value.rs:393)
```

So:

```
1. Clause gains `func: Arc<Function>` — built ONCE at registration, not per call.
     params/param_types/rest ← args        ret_type ← return_type
     body ← clause body                    closed_env ← None (clauses are top-level)
   Four existing construction sites to copy: runtime.rs 874, 1205, 1652, 1859.
2. eval_tail gains ONE arm: head resolves to a `wat__core__clauses` ClauseSet
     → evaluate args → select the clause (guards included, unchanged)
     → if it has NO `:ensure`  → emit_tail_call(clause.func.clone(), vals)
       if it HAS one           → today's ordinary call, frame intact for the post-check
```

**`emit_tail_call` is unchanged. The signal is unchanged. `apply_function`'s trampoline is
unchanged** — it already loops on `TailCall`, and a synthesized clause Function is just a Function.
That is what makes this small; but the seam is at REGISTRATION, and I had its location wrong.

⚠ **Cost of the synthesis:** one `Arc<Function>` per clause, built at load time. `Clause` already
derives `Clone`, so this must not turn a cheap clone into a deep one — the `Arc` keeps it cheap, but
that is a row to verify rather than assume.

## The four questions

- **Obvious? YES.** Every other tail-carrying form already has this; clause heads are the omission.
  A reader asking "why does `defn` recurse forever but `defclause` blow the stack" has no answer today.
- **Simple? YES.** One dispatch arm; one function split along a seam that already exists.
- **Honest? YES.** It fixes the defect instead of routing around it — and it is the reason my
  `reduce-walk` exists. It also refuses to tail-call `:ensure` clauses rather than silently dropping
  a post-condition.
- **Good UX? YES.** A user writing a recursive multi-arity verb gets the same guarantee a `defn`
  gives. Today they get a segfault at a depth nobody documented.

## Rooms

| what | where |
|---|---|
| the tail dispatcher, and the `_` arm the fix targets | `src/runtime.rs` ~4308–4340 |
| `emit_tail_call` — already evaluates args | `src/runtime.rs` ~4375 |
| the non-tail head arm that routes to clause dispatch | `src/runtime.rs:7302` |
| `eval_call_to_defclause` (evaluates args, delegates) | `src/runtime.rs:8344` |
| **`eval_call_to_defclause_with_vals` — the seam to split** | `src/runtime.rs:8364` |
| `:guard` / `:ensure` handling inside it | same fn, per its doc |
| the tail-aware siblings to copy for shape | `eval_if_tail` `:4402` · `eval_match_tail` `:4560` |

## The gate

| # | assertion |
|---|---|
| 0 | ★ **NON-VACUITY, already on disk** — the A/B pair: recursing into a `defclause` at 200k SIGSEGVs today; the plain-`defn` twin completes. Capture both verbatim BEFORE the change |
| 1 | ★★ after the change **BOTH complete**, same sum — a RED→GREEN on a committed artifact |
| 2 | ★★★ **`:ensure` IS STILL ENFORCED.** A clause with an `:ensure` that FAILS must still raise, in tail position, at depth. This is the row that matters; a green row 1 with a broken row 2 is a worse substrate than we started with |
| 3 | `:guard` still selects correctly (guards decide which arm runs — selection must be unchanged) |
| 4 | `NoMatchingClause` diagnostics unchanged — same message, same attempted-clause list |
| 5 | a non-tail clause call is byte-identical in behaviour (the common path must not move) |
| 6 | floor GREEN via `scripts/floor.sh` — the Summary line |
| 7 | clippy 0 · `#[ignore]` 13 |

Row 2 is the stone. Rows 4 and 5 are what make it safe to ship language-wide.

## Out of scope — affirmative cuts

- **Reverting `reduce-walk`.** It is a workaround for this defect, but it is also correct and it
  additionally removed a three-call walk from `reduce`'s 2-arity arm. Whether it stays once clauses
  TCO is a separate, small ruling — **not** folded in here to make this stone look bigger.
- **The six remaining three-call Stream walks** (`remove`, `take-while`, `drop-while`, `take-nth`,
  `reductions`×2) — B2's real completion, and unrelated to tail position: they are lazy producers
  whose depth is bounded by laziness. Measured: all survive 100k.
- **B3 (delete the memos)** — still blocked on those six, not on this.

## What this stone does NOT claim

That deep recursion is now safe everywhere. It gives **clause heads** the tail path that `defn`
heads already have. A non-tail recursive body — a clause recursing inside a `cons`, an argument, or
an `:ensure`-bearing clause — still consumes stack, correctly, and still dies silently if it goes
deep enough. **That is task #58's territory** (stack exhaustion is an unhandled SIGSEGV), which this
stone does not touch and does not fix.

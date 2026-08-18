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

## The change — the pieces already exist

Selection needs evaluated args; `emit_tail_call` **already evaluates args before signalling**. And
`eval_call_to_defclause_with_vals` already takes evaluated values and picks the arm. So:

```
1. split eval_call_to_defclause_with_vals (runtime.rs:8364) into
       select_clause(&cs, &vals) -> Result<&Clause, …>     ← pure selection incl. :guard
       the existing apply path                             ← unchanged for the non-tail case
2. add an eval_tail arm (runtime.rs ~4334): head names a ClauseSet
       → evaluate args
       → select_clause
       → if the selected clause has NO `:ensure`  → emit_tail_call(clause.function, vals)
         if it HAS one                            → fall through to today's ordinary call
```

No new signal variant, no new machinery. One arm plus a seam in a function that already does the
work. `eval_call_to_defclause_with_vals` takes `sym` but **not** `env` — selection is
environment-independent, which is what makes the split clean.

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

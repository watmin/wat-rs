# BRIEF — 109 step ①b: the remaining three constructors

Step ① (`f454c4650`) wired `Vector`, `HashMap`, `HashSet` to accept `(Head [type…])` in both the
checker and the runtime. Three were STOP-3'd because they have **no leading-type path at all** and
step ①'s brief forbade touching their `infer_*` fns. That constraint has done its job. This stone
lifts it for exactly those three:

```
Tuple · PersistentMap · PersistentVector
```

**Your role: you write the text. The orchestrator builds, floors, and clippies.** No `cargo`, in any
form. `./target/release/wat` is prebuilt and will NOT reflect your Rust changes — use it only to
sanity-check wat syntax, never to verify your work. Foreground everything; ending your turn ends you.
Do not commit, push, stash, or revert.

## ★ This is not cleanup. It closes a gap 109 filed and could not fix.

`109/NOTE-typed-literal-constructors.md` records: *"a complex/collection literal constructor cannot
declare its element/value type. It infers the element type from the FIRST value and rejects any later
value that doesn't match — so you cannot write a literal whose declared element type is a common
supertype holding heterogeneous elements."* Its worked example, re-measured today and **still
failing**:

```wat
(:wat::core::PersistentMap 0 (:user::A :x 1) 1 (:user::B :y "s"))   ; → 1 type-check error
```

The bracket is the form that fixes it — which makes the contract decision below the whole point of
the stone, not a detail.

## ⛔ THE ONE CONTRACT DECISION — the declared type is the UNIFICATION TARGET

Accepting the bracket is not enough. Today these three take `T = fresh.fresh()` and unify it against
the **first element**, then require every later element to match that. If the bracket merely supplied
a starting point, the note's case would still fail.

> **When a bracket is present, the declared type IS the target every element unifies against — never
> the first element's type.** A declared supertype must therefore accept heterogeneous members.
> When no bracket is present, behaviour is EXACTLY as today: infer from the first element.

If you cannot make a declared supertype accept heterogeneous elements without changing unification
itself, **STOP and report** — that is a substrate question, not a constructor one.

## Read in order

1. **`src/check.rs`, `infer_hashset_constructor`** — the WORKING template. `let t_ty = match &args[0]
   { WatAST::Keyword(k, _) => …parse as a type… }`. That leading-type read is what the three lack.
2. **`src/check.rs`, the three to change:**
   `infer_tuple_constructor 14142 · infer_persistentmap_constructor 13950 · infer_persistentvector_constructor 14014`
3. **`src/check.rs`, `unwrap_type_param_bracket`** — step ①'s helper, already `pub(crate)`. Reuse it
   or bypass it, your call — but say which and why. ⚠ Splicing alone is NOT sufficient here: the
   spliced type keyword would go to `infer()` as a value and hit the Doctrine-1 guard (arc 242,
   `check.rs:1894`). These fns need a real leading-type READ, like the template.
4. **`src/runtime.rs` dispatch:** `Tuple 6233 · PersistentMap 6442 · PersistentVector 6448`.
5. **The runtime ctors:** `eval_tuple_ctor` at **`src/runtime.rs:11694`** (⚠ NOT in `collection/eval.rs`);
   `eval_persistentmap_ctor` **`src/collection/eval.rs:1179`**; `eval_persistentvector_ctor`
   **`src/collection/eval.rs:1658`**.

⚠ Confirm every line number by matching surrounding code, not by trusting it. An earlier brief in
this arc carried six numbers that were extrapolated rather than measured; the rider that checked them
is the only reason it cost nothing.

## The shapes

```wat
(:wat::core::PersistentVector [:wat::core::i64] 1 2 3)        ; T declared
(:wat::core::PersistentMap [:wat::core::String :wat::core::i64] "a" 1)
(:wat::core::Tuple [:wat::core::i64 :wat::core::String] 1 "a") ; per-POSITION types
(:wat::core::PersistentVector 1 2 3)                           ; ★ no bracket — unchanged, still infers
```

`Tuple` is the odd shape: its bracket declares one type **per position**, so the arity of the bracket
must equal the arity of the values. Decide and state what a mismatch does.

## Blast radius

`src/check.rs` (three fns) · `src/runtime.rs` (one ctor + three dispatch arms) ·
`src/collection/eval.rs` (two ctors). No lexer. No `.wat`. No `tests/`. No renderer. Do not touch the
three constructors step ① already wired.

## STOP triggers — each rejects; none is a fallback

1. A declared supertype cannot hold heterogeneous elements without changing unification. STOP; report.
2. Any bracket-less form changes behaviour. STOP — this step is additive.
3. `Vector` / `HashMap` / `HashSet` behaviour changes at all. STOP — they are done and out of scope.
4. `Tuple`'s per-position bracket cannot be made to work. STOP; report, and ship the other two.

## Acceptance criteria

- All six constructors accept `(Head [type…] …values…)` in checker AND runtime.
- ★ The note's case builds: two different record types under one declared supertype in a
  `PersistentMap`.
- Every bracket-less form still behaves exactly as before — `(PersistentVector 1 2 3)` included.
- `Vector` / `HashMap` / `HashSet` untouched.

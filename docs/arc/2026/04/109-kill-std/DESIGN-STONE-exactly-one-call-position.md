# DESIGN — exactly one call position, and `(Head :- [])` IS `Head`

> *"how do we have exactly one?"*
> *"annihilation is our greatest pleasure - we return with exactly one way to do this"*
> — the builder, 2026-08-23

Two blockers stopped `defservice` emitting the binder in the previous stone. Both are the same
disease at different layers, both are measured, and both are small once located.

## Blocker 1 — the call position has TWO implementations

Both live in **one function**, `infer_list` (`src/check.rs:2540`):

```
infer_list
  …
  if k.contains('/') {           ← 4962   surface-method arm, fires FIRST
      split_type_params_pub(…)   ←        reads type args from the ANGLE SUFFIX
      …its own arity check…
  }
  …
  peel_param_spec(args)          ← 5494   generic-call arm — learned `:-` in 69933d362
  …its own arity check…
```

One call form. Two arms. **The peel lives inside one of them**, so teaching "the call position" taught
half of it. Measured cost: emitting the binder at a surface-method call gave
`ArityMismatch: expected 7 argument(s); got 9`, ×257.

★ **The rule that fixes it, and it generalises:**

> **Whether a call carries a param-spec is a property of the FORM. Which arm handles it is a property
> of the CALLEE.** Doing form-level destructuring inside callee-level branches is what turns one
> implementation into N.

So the peel is **hoisted above the dispatch** — to where the call form is first destructured, before
anything branches on what the head resolves to. Both arms then receive `(type_args, args)` already
separated and neither implements extraction.

And it kills the old carrier for free: the surface arm's `split_type_params_pub` read is the
**pre-`:-`** way of carrying type args at a call site. Once the peel is hoisted, that read has nothing
left to do — it dies with the angle form rather than needing its own retirement.

`src/runtime.rs:7046` is the same arm at the runtime layer, with `7432` already peeling. Same hoist.

## Blocker 2 — `(Head :- [])` is not `Head`, and the code says why

```clojure
(:wat::core::defn :u::takes [x <- (:u::Plain :- [])] -> :i64 …)
  called with (:u::Plain :n 9)
  →  ":u::takes: parameter #1 expects :u::Plain; got :u::Plain"
```

**A type that does not match itself.** `(Head :- [])` parses to `TypeExpr::Parametric { args: [] }`;
the bare name parses to `TypeExpr::Path`. Different Rust variants; `assignable` does not unify them.
They now RENDER identically (this session's renderer stone), which is why the error reads as it does.

The cause is a deliberate, documented decision at `src/types.rs`'s `:-` arm of `parse_type_form`:

> *"unlike the unmarked arm below, there is no `!inner.is_empty()` guard here, because under `:-` an
> empty bracket is a legitimate zero-length param-spec (`(Tuple :- [])`), not something to guess
> about."*

Arc 109 ②-i-b made `(Head :- [])` a DISTINCT thing from absent, on the reasoning that `:-` declares
rather than sniffs. **That is superseded.** The builder's rule, stated explicitly this session:

```
not expressed        →  :- []
expressed and empty  →  :- []
otherwise            →  the binders chosen
```

If absent and `:- []` are the same thing, `Parametric{args:[]}` must not exist. Normalising at that
one door is a one-line change — **not** the 370 construction sites or the 95 match sites, because
that door is where `:- []` enters.

⚠ **One door is where the measured defect enters; it is not proof no other door builds the empty
form.** So the stone also IMPOSES the check — assert the empty form is never constructed, run the
floor, read the screams. `[[feedback_impose_the_check_and_read_the_screams]]`. A census scoped to the
site that happened to bite is the error this arc has paid for repeatedly.

## What ships

```
1. hoist peel_param_spec above `if k.contains('/')` in infer_list — both arms consume it
2. the same hoist at src/runtime.rs:7046
3. delete the surface arm's split_type_params_pub angle-suffix read
4. (Head :- []) normalises to Path at parse_type_form's `:-` arm
5. impose the no-empty-Parametric check and close whatever screams
6. THEN: wat/service.wat emits the binder, and the five mono-vs-parametric branches go unconditional
```

Step 6 is the deliverable the previous stone had to revert. Steps 1–5 are exactly what it was blocked
on, and the rider that hit both STOPs already knows the shape of 6.

## What this does NOT do

- **The minting wall and `symbol-node`'s wall.** Next stone, once nothing mints.
- **The purge of the angle parsers.** After the wall, with a green floor to prove them dead.
- **`keyword/from-string` → `(:wat::core::keyword "x")`.** Filed as its own NOTE this session; the
  verb-equals-type family gets decided together with `List/of` and `char/of`, not piecemeal.

## The four questions

- **Obvious?** YES. "Where does a call learn its type arguments?" gets one answer instead of two, and
  `(Head :- [])` stops being a second spelling of `Head` that does not equal it.
- **Simple?** YES. A hoist, a deletion, and a normalisation. The stone removes two implementations and
  one type-variant state; it introduces nothing.
- **Honest?** YES, and both blockers are dishonesty of the same kind: an arity error that names the
  wrong cause, and a type error that says a type differs from itself. Each is the substrate reporting
  a distinction it should not have.
- **Good UX?** YES. `(:S/method :- [T] recv arg)` becomes writable, which is what a reader would try
  after `(:f :- [T] arg)` works — and today it answers with an arity count that explains nothing.

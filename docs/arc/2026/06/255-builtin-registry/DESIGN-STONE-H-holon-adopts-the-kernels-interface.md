# DESIGN — STONE H: holon adopts the interface the rest of the kernel uses

> Builder, 2026-08-28: *"holon was some of the first tooling — we built wat to better use holon — it
> is very likely needing some degree of corrective change relative to the rest of the code base…
> this homing exercise is flushing out all odd/unexpected behaviors…"* and then, on whether to fix
> the blocker or move on: *"it strongly sounds like the holon tooling needs to migrate to a now sane
> interface that the rest of the kernel uses."*
>
> He is right, and the blocker is not the one I named. **holon is not span-carrying. It is
> arity-carrying**, and that is a single root cause with a mechanical fix.

## The measurement — a clean binary, and this one has both controls

```
handler signatures under src/intrinsic/holon/     95 of 95   `args: &[WatAST]`   ← VARIADIC
the collections, pre-migration (at 4fad41b35~1)   12 of 12   `m: &WatAST` etc.   ← FIXED ARITY

hand-rolled `if args.len() != …` checks
  in src/intrinsic/holon/                          89
  in the collections O-iv-b migrated                0
```

⚠ **This is a literal-pattern count with a positive AND a negative control** — 89 where the shape
predicts them, 0 where it predicts none — unlike the three span classifiers this design already
retracted. It is quoted because it can be checked in one command, not because a regex agreed with
itself.

## The chain, and it is one link long

`crates/wat-macros/src/wat_intrinsic.rs` generates an arity check **only for the fixed-arity shape**:

```rust
let body = if is_variadic {
    wrap_call(quote! { #fn_name(args, env, sym, list_span) })     // ← NO arity check. None.
} else {
    // … `if args.len() != #n { ArityMismatch … }` …
};
```

So:

> **holon declares itself variadic → the macro checks nothing → every handler hand-rolls an arity
> check → every hand-rolled check needs `list_span` to raise its error → every handler "uses a span".**

That is why 95 of 95 came back span-using. It was never a property of what holon *computes*. It is
the downstream shadow of one declaration choice made before `#[wat_intrinsic]` generated anything.

★ **And it explains the shape of the whole arc's difficulty with holon.** HOME-8 carved it, Stone G
gave it provenance, O-iii's generator cannot reach it — each stone met the same wall from a different
side, and the wall is a signature.

## The correction — declare the real arity

```rust
-#[wat_intrinsic(":wat::holon::to-holon")]
-pub(crate) fn eval_holon_to_holon(args: &[WatAST], env: &Environment, sym: &SymbolTable, list_span: &Span)
-    -> Result<Value, EvalBreak> {
-    if args.len() != 1 { return Err(RuntimeError::new(list_span.clone(), ArityMismatch{…})); }
-    let v = eval_inner(&args[0], env, sym)?.value_owned();
-    to_holon_inner(v, args[0].span())
-}
+#[wat_intrinsic(":wat::holon::to-holon")]
+pub(crate) fn eval_holon_to_holon(v: &WatAST, env: &Environment, sym: &SymbolTable, _span: &Span)
+    -> Result<Value, EvalBreak> {
+    let val = eval_inner(v, env, sym)?.value_owned();
+    to_holon_inner(val, v.span())
+}
```

The macro's generated check replaces the hand-rolled one — **same `RuntimeErrorKind::ArityMismatch`,
same `op`, same shape** — and `list_span` stops being needed.

★ **THE COMPILER IS THE INSTRUMENT, and that is the point of doing it this way.** Where `list_span`
becomes genuinely unused, the compiler says so. No pattern, no census, no fourth regex: the verbs that
can drop their span identify *themselves*, and the ones that cannot keep it and say why. This design
has already retracted three text instruments for this exact question.
`[[feedback_impose_the_check_and_read_the_screams]]`

## What is NOT mechanical — the residue this stone must surface, not bury

- **Genuinely variadic verbs stay variadic.** `eval_holon_from_holon` accepts 1 *or* 3 args; its
  arity is a range, so its hand-rolled check is honest and stays. Expect a handful.
- **`eval_holon_from_holon` also parses a runtime `-> :T` annotation** —
  `(from-holon h -> (:wat::core::HashMap :- [K V]))`. **Arc 258.4 retired that ascription**, and
  Stone P6-a corrected two `if` doc comments this same day that still described it. **holon still
  implements it.** Out of this stone's scope; it needs its own draw and probably the builder's word
  on whether the 3-arg form survives at all.
- **`eval_holon_from_holon` returns `TrackedValue`** (Stone G provenance) and stamps
  `Provenance::RuntimeBuilt { call_span }`. That is a REAL call-span use that no arity change
  removes.

## What this unblocks, and what it does to Stone Q

**Q is not cancelled and its argument is unchanged** — a span is not binding state, `apply` holds one,
the value door drops it. But **Q's necessity and size are downstream of H**, because most of holon's
apparent span-dependence is the arity artifact. **Do not size Q until H has landed and the compiler
has said which spans survive.**

Order: **H → re-measure with the compiler → then Q or not → then O-iv-c.**

## Decomposition

| | stone | population | why the split |
|---|---|---|---|
| **H-1a** | `subspace` 10 · `engram` 10 · `reckoner` 8 · `hologram` 7 | 35 | proves the shape on four files before the big one |
| **H-1b** | `atom` | 60 | the bulk, on a proven shape |

## The four questions

- **Obvious? YES.** A verb of arity 1 declares one parameter. A reader meets the arity in the
  signature instead of in a hand-written `if` twenty lines down.
- **Simple? YES.** It deletes code and adds none: the check the macro already generates replaces the
  one written by hand, everywhere the arity is genuinely fixed.
- **Honest? YES**, and this is the load-bearing one. Declaring `args: &[WatAST]` says *"this verb
  takes any number of arguments"*. For 89 of these handlers that is **false** — they take exactly N
  and immediately say so in an `if`. The signature currently lies and the body corrects it.
- **Good UX? YES.** `metadata-of` reports `:arity` from the declaration, so today every holon verb
  reports **variadic** regardless of its real arity — the same defect Stone P2 just fixed for
  `:wat::core::if`, in a second place. This fixes it for 89 more.

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

---

## ⛔ H-1a SHIPPED — and it REFUTED this design's own hypothesis about spans

**35 verbs converted, −542/+235.** All 35 reported `:arity -1` before; after, all report their real
N (17×1, 11×2, 3×3, 3×4, 1×5). No verb in these four files is genuinely variadic — the only one in
holon is `from-holon` (1-or-3), and it is `atom.rs`/H-1b.

★ **THE HYPOTHESIS WAS WRONG, AND THE COMPILER SAID SO.** This design argued *"most of holon's
apparent span-dependence is the arity artifact"*. Measured by the compiler, per verb:

```
list_span became UNUSED     5 of 35     the arity check was its only reader
list_span STILL USED       30 of 35     require_subspace / require_engram / require_reckoner /
                                        require_numeric / require_encoding_ctx / with_mut — all
                                        take a call span and locate their TypeMismatch at it
```

**Five, not most.** The arity fix was right and worth doing on its own merits, but holon's span
dependence is overwhelmingly REAL, carried by its `require_*` helper family. **Stone Q is therefore
NOT optional for holon — it is required**, and this is the first sizing of it that did not come from
a pattern I had to retract. `[[feedback_impose_the_check_and_read_the_screams]]`

## ★ AND SPLITTING THE COLLAPSED `@arg` LINE EXPOSED FIVE DOC LIES

Every one of these handlers documented its arguments as ONE collapsed line —
`@arg args… :wat::core::Value the reckoner, the prediction's conviction, and whether it was
correct, in order`. `doc_arg_ret_types_match_checker_scheme` compares per-argument doc types against
the checker's scheme; **a single collapsed line has no per-argument type to compare, so the gate
verified nothing.** Declaring the real arity gave it something to check, and it immediately failed
five times:

| verb | doc said | checker says |
|---|---|---|
| `Reckoner/resolve` arg 1 | `:wat::core::Value` | `:wat::core::f64` |
| `Reckoner/observe` arg 3 | `:wat::core::Value` | `:wat::core::f64` |
| `Reckoner/new-continuous` | `:wat::core::Value` | `:wat::core::f64` |
| `Reckoner/new-discrete` arg 3 | `:wat::core::Vector` | `(:wat::core::Vector :- [:wat::holon::HolonAST])` |
| `Hologram/make` arg 0 | *(see below)* | `[:wat::core::f64 :-> :wat::core::bool]` |

**This is the same shape as Stone P6-a**, which published two inverted `if` doc comments by making
`show-source` reach them. A claim nothing could check is not a claim that was true.

⚠ **AND THE ORCHESTRATOR ADDED ONE OF THE FIVE.** The rider had typed `Hologram/make`'s filter as
`:wat::core::fn`; I "corrected" it to `:wat::core::Fn` on the grounds that four other corpus sites
spell it that way. **Both were wrong, and the authority was two directories over the whole time** —
the checker says `[:wat::core::f64 :-> :wat::core::bool]`. Reaching for a corpus majority instead of
asking the enforcing gate is the same reflex as reaching for a grep instead of the compiler.

⚠ **A finding recorded, not chased:** `require_numeric` accepts `Value::i64` **or** `Value::f64`,
while the checker admits only `f64` for every parameter that reaches it. **Its `i64` arm is
unreachable through the checked path** — only via `eval-ast!`. Three separate declarations of one
parameter's type (doc, checker, body) and all three disagreed; the docs now match the checker,
because the checker is what a caller actually meets.

⚠ **Clippy caught what the floor could not, again:** `eval_reckoner_new_continuous` is a 5-arg verb,
so its Rust signature is 8 params with the `env`/`sym`/`span` tail — one over
`clippy::too_many_arguments`. Carried as `#[expect(…, reason = …)]`, not `#[allow]`, so it goes red
if the signature ever shrinks under the limit. `[[feedback_an_exemption_is_earned_when_the_alternative_is_worse]]`

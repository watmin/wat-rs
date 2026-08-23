# DESIGN — reap the angle machinery: 16.2M calls, zero findings

> *"a fatal blow has been struck... now... we reap what is ours to reclaim - the heresy of illegal
> parametric type declarations is annihilated"* — the builder, 2026-08-23

`<K,V>` became unexpressible in `0811c3009` — written, minted, or rendered. The machinery that existed
to parse it is still compiled, still called, and now finds nothing.

## This is not dead CODE. It is dead WORK.

Every purge candidate was instrumented and one full floor run measured:

```
                                 calls        type-heads found    callers
canonical_callable_name       14,481,408              0              13
split_name_and_type_params     1,128,732              0              11
split_type_params                617,719              0               4
split_method_name_type_params          0        never called          4
                              ──────────
                              16,227,859              0
```

**16.2 million calls per floor run. Zero type-heads.** Of ~25,400 calls whose argument contained an
angle at all, **every single one was the comparison operator `::<`** — which these functions
explicitly do NOT strip (the balanced-suffix rule: strip only when the name also ends in `>`).

So this is not "delete unreachable code." It is **removing a no-op from the hottest path in the
runtime**, executed sixteen million times per floor to do nothing. Correctness and cost point the
same way.

⚠ **And `split_method_name_type_params` has 4 callers and 0 calls.** Callers that never execute are a
separate finding from a no-op that always executes — the rider must say which of the four are
unreachable and why, not simply delete the function and let the compiler sort it out.

## What is already gone, and must not be re-derived

`check.rs`'s explicit-type-suffix arm — the hand-rolled parser that split `<T1,T2>` and bound a
surface method's type params from it — **was removed by the call-position hoist** in `c6c614fe2`. Only
a comment at `check.rs:4964` still refers to it. That comment is prose describing a mechanism that no
longer exists and is in scope here.

## What must SURVIVE, and it is the trap

```clojure
:wat::core::<     :wat::core::>     :wat::core::>=     <-     ->
```

The operator names contain `<` and `>`. They are the entire reason these functions have a
balanced-suffix rule in the first place. **A purge that removes the rule along with the parser will
take the operators with it** — and the 25,400 measured calls are exactly those operators flowing
through. Any replacement must leave them untouched, and the proof is that they still dispatch.

## The two callers that are NOT purge candidates

`split_type_params_pub` has four references; two are live and neither parses a call head:

```
src/types.rs:836          base_of_rendered_type     strips a base off a RENDERED type string
src/types/surface.rs:997  message_is_declared       same
```

⚠ Both are the shape that has bitten **three times** in this arc: *a search for a character that no
longer exists does not fail — it succeeds wrongly.* The renderer now emits `(Head :- [args])`, so a
rendered string contains no `<`, and a `find('<')` there silently returns the whole string.
`base_of_rendered_type` was already taught the `:-` form during the renderer stone; `message_is_declared`
was not examined. **It must be, before anything is deleted around it.**

## What ships

```
1. split_method_name_type_params  — 0 calls: delete, and account for its 4 callers
2. canonical_callable_name        — delete; its 13 call sites use the name directly
3. split_name_and_type_params     — delete; 11 call sites
4. split_type_params / _pub       — delete the call-head parsing; keep whatever
                                    base_of_rendered_type / message_is_declared genuinely need,
                                    taught the `:-` form, not the angle form
5. check.rs:4964's stale comment  — the mechanism it describes is gone
6. a rune, or the existing one_param_spec extended — so a hand-rolled `<`-parse cannot return
```

## The four questions

- **Obvious?** YES. After it, no code in the substrate parses a syntax the language cannot express.
- **Simple?** YES. Deletion, plus one honest question about two rendered-string callers.
- **Honest?** YES, and it is the axis this stone is really about: keeping a parser for a dead syntax on
  the hot path implies the syntax might still arrive. It cannot. The code says otherwise 16 million
  times a run.
- **Good UX?** YES — for the next reader, who will not have to work out whether `canonical_callable_name`
  is load-bearing. Measured: it is not.

## Out of scope, affirmatively cut

- **`keyword/to-type-form`'s angle half.** Its live caller at `wat/service.wat:434` is a transition
  shim accepting either spelling. Retiring the angle half is a separate question about that shim's
  contract, not a deletion of dead code.
- **`keyword/from-string` → `(:wat::core::keyword "x")`.** Its own NOTE; the verb-equals-type family
  gets decided together.
- **The 411 wat + 591 rust comment lines** still written in the retired spelling. FM 14 Bucket B, its
  own sweep, and a blind pass would erase the lines that RECORD the retirement.

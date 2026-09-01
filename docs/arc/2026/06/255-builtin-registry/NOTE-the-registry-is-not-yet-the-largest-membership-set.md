# NOTE — the blanket-accept is arc 255's founding target, and it is still on disk

> **Builder, 2026-09-01:** *"a nonexistent `:wat::` verb type-checks clean — we have been attacking
> this for weeks or months... this is the registry's target."*
>
> I had filed it as *"documented, not a finding of this stone."* It is the **arc's opening defect
> statement**, unshipped. This NOTE measures the distance.

## The arc said it would be deleted. It is verbatim on disk.

`docs/arc/2026/06/255-builtin-registry/DESIGN.md`, the founding text:

> *"with no registry to ask 'is this defined?', `resolve` falls back to
> `if is_reserved_prefix(head) { return true }` — blanket-accepting any leaf, deferring wrong names
> to a runtime 'unknown function' (the 30-minute wat-lru crawl)."*
>
> *"**The reserved-prefix blanket-accept hack is DELETED**; builtins resolve through the same path as
> user forms. One resolution path for everything."*
>
> *"The undefined-func class dies as a **side effect** of fixing the real defect."*

`src/resolve/walk.rs`, `is_resolvable_call_head`, today:

```rust
if is_reserved_prefix(head) {
    return true;
}
```

The registry was built — 457 rows. **The hack it was built to delete is untouched, and the side
effect never happened.**

## ⛔ It is a RESPONSIBILITY LOOP, not one hack

Each layer defers to the other, in its own comment:

```
resolve/walk.rs   "A wrong name under those prefixes (e.g. :wat::holon::Bogus) fails DOWNSTREAM
                   at runtime or lowering ... leaf-level validation is THE TYPE CHECKER'S CONCERN."
check.rs          blanket-accepts (`CheckResult::ok(fresh.fresh())`) and defers to the runtime's
                   UnknownFunction door.
```

Neither owns it, so nothing catches it. **The arc's own example still reproduces**, measured against
the built binary:

```
(:wat::holon::Bogus 1 2)              --check → ACCEPTED     run → UnknownFunction
(:wat::linkedlist::lenght …)          --check → ACCEPTED     ← a ONE-LETTER TYPO ships
```

## The measurement — why it has not been deleted

I made the change (registry membership instead of the prefix short-circuit) and ran the whole corpus:

```
578 of 599 .wat files FAIL      — 96%
```

Not a reason to stop; a diagnostic. The failing names, collected across the corpus and deduped, are
the **exact worklist** of what the registry cannot vouch for:

```
121 distinct names
    0   are registry rows          ← asking the registry costs nothing that works today
   68   have a checker SCHEME but NO registry row
   53   known by NEITHER
```

★★★ **The decisive number is 68.** `check.rs`'s `register_builtins` knows 68 callables the registry
does not. **The registry is not the single sole authority — it is not even the LARGEST membership
set.** That is the gap, stated in one line.

And the 53 that nothing structural knows, by kind:

```
special forms   fn · def · match · quote · do · ann-form · quasiquote · defclause
                extend-type · derive · macroexpand · macroexpand-1
types           :wat::type::{i64,String,Vector,Tuple} · PersistentVector · PersistentMap
rete forms      :wat::rete::core::{and,or,if,let,fn,match,map,filter,foldl,reduce} · …
verbs           = · not= · < · > · <= · >= · and · or · first · second · third · str
                println · None · tuple-get · reduce-walk · edn::write · stream::lazy
```

★ **The registry holds exactly TWO special forms — `let` and `if`.** `def`, `fn`, `do`, `defn`,
`quote`, `match` are not rows. `runtime.rs` still carries **55** literal `:wat::core::*` dispatch
arms. The mechanism to register them exists and is used (`#[wat_special_form_impl]`, `Kind::SpecialForm`,
the third inventory stream folded in `registry()`); the population was never migrated.

## What this makes true

**The blanket-accept cannot be deleted first.** It is not the defect — it is the *symptom* of the
registry not yet being the largest membership set. Delete it today and 96% of the corpus stops
resolving, because the registry cannot vouch for `fn`.

The order is forced, and it is the reverse of how it looks:

1. **the registry becomes complete** — the special forms, the 68 scheme-only verbs, the types
2. **then** `resolve` asks the registry instead of the prefix
3. **then** the undefined-func class dies, exactly as the founding DESIGN said — *as a side effect*

## ⚠ What I got wrong, twice, in one hour

1. I reported *"a nonexistent `:wat::` verb type-checks clean"* as **not a finding of this stone**,
   citing `tests/cli/retirement_table_reachable.rs` documenting it. Documented is not the same as
   accepted. It is the arc's target, and I walked past it because a test's doc comment mentioned it
   in passing. `[[feedback_the_refutation_i_brought_was_already_in_the_document]]`
2. The immediately preceding stone's DESIGN claimed the new check covers *"every row the registry
   knows and the checker does not"* — false, measured at 48/71, and the claim reached a code comment
   before anything measured it.

Both are the same shape: **a claim about coverage, written before the coverage was measured.**

## ⬜ The first stone, if the shape is ruled

Register the special forms. It is the largest bucket by corpus frequency (`fn` 3,952 calls, `def`
3,431, `match` 1,613), the mechanism already exists, and it is the bucket that makes the other two
tractable — a corpus that cannot resolve `fn` cannot be measured for anything else.

⚠ **Open shaping question, the builder's to rule:** a special form has no rank-1 `TypeScheme` and no
`NativeHandler` — `IntrinsicEntry` carries `handler: Option<NativeHandler>` and `Kind::SpecialForm`
precisely for this. So the question is not *whether* they can be rows, but what a row must ASSERT to
be worth having: membership alone (kills the undefined-func class), or membership + `@syntax` +
arity, which is what would let the checker stop hand-rolling their arms.

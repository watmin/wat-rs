# NOTE (arc 109) — the F5 pure-combinator allow-list: where it lives, and the diagnostic that misdirects

**Filed 2026-08-21 after β-ii-a′.** Three refusals in one stone, each costing a full build cycle,
each taking the WHOLE STDLIB down (3029 failures) because the refusal fires **at macro definition**.
Written so the next rider into a macro body does not pay for the same three.

## Where the gate is

`src/macros/eval.rs:351` — `fn is_pure_total(head: &str) -> bool`, a `matches!` over ~57 heads.
**Default-deny**: any keyword-headed sub-form in a program-body macro whose head is not listed is
refused. Arc 249 stone 249.2b-i (`F5 CLOSED`).

The blessed `string::` set, in full, since it is what macro authors reach for most:

```
concat · contains? · ends-with? · interpolate · join · length · split
starts-with? · subs · to-bool · to-lowercase · to-uppercase · trim
```

The list's own doc states the maintenance rule, and it is the opposite of "route around it":

> *"The suite teaches completeness: a false-refusal (a pure head missing from this list) makes a
> stdlib test RED. **Add it here.** A missing effectful head is harmless (stays denied)."*

So a genuinely pure head that is refused is a **bug in the list**, fixed by adding it — not by
rewriting the macro to avoid it.

## ★ THE DIAGNOSTIC MISDIRECTS — measured

`:wat::core::string::index-of` is refused by F5 with:

> *"keyword head `:wat::core::string::index-of` refused at macro expand time — not on the
> pure-combinator allow-list (default-deny F5 gate, arc 249 stone 249.2b-i); only pure-total heads
> are permitted"*

**That verb does not exist.** Called from ordinary code it answers
`#wat.runtime/UnknownFunction {:message "unknown function: :wat::core::string::index-of"}`.

The gate checks the allow-list before the head resolves, so a **nonexistent** verb is reported as a
**purity violation**. A reader following the list's own maintenance rule would go add `index-of` to
`is_pure_total` — and only then discover there is nothing to bless. The message names a real gate
and the wrong defect.

★ This is the same class as the `:wat::` blanket in `--check` (255 #110): a check that answers
confidently about a name it never looked up.

## The three refusals, and what each actually means

| attempted | refused | the real rule |
|---|---|---|
| a top-level `(defn :wat::service::tp-suffix->syms …)` called from the macro | at DEFINITION | **A macro body may not call user-defined functions at all.** Only blessed primitives. There is no "define a helper next to the macro" option. |
| `(mapv :wat::core::symbol-node xs)` — a bare primitive keyword as a VALUE | at EXPANSION, as `TypeMismatch … expected "wat::core::fn"` | `mapv`/`map` want a fn VALUE. In a macro body use **`foldl` + `conj`** — the idiom `:wat::core::keyword/of` uses at `wat/core.wat:1328`, with an inline `(:wat::core::fn …)` literal. |
| `string::index-of` | at DEFINITION | the verb does not exist; see above |

⚠ **The first is the expensive one and the least obvious.** A rider writing wat naturally reaches
for a helper `defn`; the language allows it everywhere EXCEPT here, and the failure is not local —
the stdlib stops loading and 3029 tests go red at once.

## What a macro body CAN do, from this file's own working code

`wat/service.wat` and `wat/core.wat` between them establish the vocabulary, all verified in use
INSIDE a macro body: `first` · `rest` · `empty?` · `conj` · `foldl` · `get` · `length` ·
`ast->children` (145 uses corpus-wide) · `ast-name` (131) · `ast-kind` (78) · `ast-span` ·
`keyword/to-string` · `keyword/from-string` · `symbol-node` · `macro-error` · `if` · `let` ·
`fn` literals · `Vector`/`HashMap` constructors and their ops · the blessed `string::` set above.

**`macro-error` is the way a macro raises a structured, catchable failure** (`wat/core.wat:632`,
`:782`) — see `NOTE-a-macro-cannot-diagnose-with-option-expect.md`, which this closes.

## The suggested fix, NOT taken here

Adding `index-of` to `is_pure_total` would be wrong twice over: the verb does not exist, and arc 109
is REMOVING string surgery rather than enabling more of it. The gate's *diagnostic* is the defect
worth fixing — it should resolve the head first and say "unknown function" when that is the truth.
Bounded to `src/macros/eval.rs`'s refusal path; not scoped to a stone yet.

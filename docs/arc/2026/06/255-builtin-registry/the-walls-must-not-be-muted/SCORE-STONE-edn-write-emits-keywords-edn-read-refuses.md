# SCORE — STONE: `edn::write` stops emitting keywords `edn::read` refuses

Amended after `[[REFUTE-the-fold-activates-a-silent-corruption]]`. Part 1 (the fold)
is **reverted**. Part 2 (`try_ns` refuses `/` in the name) **is** the fix. Part 3
(`fqdn_of` comment) kept. No commit.

## The correction

Folding `:wat::core::HashMap/length` to `:wat.core.HashMap/length` made `try_ns`
succeed, so the verbatim path was never taken, and `ns_to_wat_path` decoded it
as `:wat::core::HashMap::length` — **the wrong name, silently.** Arc 213's
witness fired: `CORRUPTED — wrong keyword`.

Without the fold, Part 2 routes the unspellable leaf through machinery that
already existed:

```
try_ns("wat.core", "HashMap/length")  -> Err
keyword_from_wat_path Err arm        -> verbatim_keyword
#wat.ast/Keyword {:path ":wat::core::HashMap/length"}
bridge.rs:727                        -> the SAME keyword
```

Arc 213 now prints:

```
NOTE: :wat::core::HashMap/length round-tripped correctly (EDN str:
#wat.ast/Keyword {:path ":wat::core::HashMap/length"}). STOP trigger may be resolved.
```

Decoded == original. The witness was not weakened.

## Rows 1–2 — write, then read

Before: `":wat.holon/Hologram/make"` (two slashes, `read` refuses).

After: `#wat.ast/Keyword {:path ":wat::holon::Hologram/make"}` — valid EDN, exact
path. Unit: `type_method_keyword_is_carried_verbatim_and_reads_back`.

## The five goldens — third shape, one keyword each

Not the two-slash form, not the fold. The unspellable sentinel became tagged
carriage; every other keyword in the file is untouched.

```
:wat.core/__internal/special-form
  ->  #wat.ast/Keyword {:path ":wat::core::__internal/special-form"}
```

Same for `__internal/type-decl`, `__internal/primitive`. `:wat.vec/length` and
`:wat.core/if` stayed. `git diff` per file is that one node.

`contract-06` (Option/expect) is the same class:
`(:wat.core/Option/expect …)` → `(#wat.ast/Keyword {:path ":wat::core::Option/expect"} …)`.

The five `lookup_form_*` sentinels now match the path string
`:wat::core::__internal/special-form`.

## `try_ns` (kept)

```
try_ns("wat.holon", "Hologram/make") -> Err("name must not contain /")
try_ns("wat.holon.Hologram", "make") -> Ok
try_ns("wat.core", "/")              -> Ok
```

## `--test lint`

```
cargo nextest run --release --test lint
  Summary [  90.743s] 118 tests run: 118 passed, 0 skipped
```

The `one_name_grammar` rune is gone — `split_clojure_symbol_ns_name` went with
the fold.

Census **571 · 85 · 52**. `fqdn_of` still comment-only.

## The wire is honest, not folded

Unspellable wat names are carried as themselves. The reverse
`Bytes::to-hex` / `Bytes/to-hex` ambiguity is still arc 251's. This stone does
not pretend otherwise.

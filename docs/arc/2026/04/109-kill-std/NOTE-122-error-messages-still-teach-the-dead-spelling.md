# NOTE — 122 hard-coded error messages still teach the dead spelling

**Filed 2026-08-23**, from R4's finding during the prose sweep, confirmed and censused by me.

## User-visible, at the worst possible moment

```clojure
(:wat::core::nth "not a vector" 0)
```

```
:expected "Vector<T>, List<T>, PersistentVector<T>, or WatAST"
```

Three of those four names cannot be written. The message is hard-coded at `src/check.rs:9481` as a
`CheckErrorKind::TypeMismatch { expected: … }` string literal.

**This is the moment a user is most looking for guidance — they have already made a mistake — and the
substrate answers in a language it refuses.**

## The census

Hard-coded string literals naming a wat type in the retired spelling, Rust generics excluded:

```
src/runtime.rs                          35
src/collection/eval.rs                  29
src/check.rs                            22
src/collection/infer.rs                 16
crates/wat-edn/interop-tests            7
src/types.rs                            2
                                       ───
                                       122
```

Samples: `":wat::core::Vector<wat::WatAST> of param name nodes"`, `":AST<wat::WatAST>"`,
`"#wat.kernel/Address (name field must be Vector<i64>)"`, `":wat::core::Option<wat::core::i64>"`.

## ★ The FOURTH channel of one defect, and why every census missed it

The rule this campaign has been enforcing is **a type must reach a user in a spelling the reader
accepts.** Four channels violate it, found in this order:

```
1. RENDERED types      format!("{}<{}>")           FIXED  (the renderer stone, 64a8fa5a0)
2. DOC @arg/@ret        starts_with(':') validation FIXED  (the validator stone, f82dc6de1)
3. FUNCTION types      Fn(A,B)->C printed          FOUND  (NOTE, unfixed)
4. HARD-CODED PROSE    122 string literals         FOUND  (this NOTE)
```

Every census I ran looked at **code that parses or renders a type**. A string literal naming a type is
neither — it is prose that happens to live in Rust. So the instrument could not see it, four times
running, while I reported each channel closed.

`[[feedback_scope_the_check_from_the_rule_not_the_diff]]` — but the sharper reading is
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`: each census was accurate about
what it measured and silent about what it could not, and I quoted the silence as coverage.

## What is owed

- **The 122 rewritten** to the surviving spelling — `(:wat::core::Vector :- [:wat::WatAST])`.
- ⚠ **Not by regex.** `Vec<T>`, `Arc<Function>`, `Cow<'_, [WatAST]>` are Rust's own generics and appear
  in the same files; the discrimination is the work, exactly as it was for the `.rs` comment slice.
- **A gate, or it returns.** The three fixed channels each got one: a rune, a validator, a
  positive-controlled test. A sweep with no gate is a moment. The natural shape here is a rune over
  string literals in diagnostic positions — with earned exemptions for the ones quoting history.
- **And the fifth channel, before claiming completion.** The lesson of four consecutive misses is that
  the next census should ask *"through what channel can a type name reach a user?"* and enumerate the
  channels — not *"where does the code parse a type?"*

## Scope

Out of the prose sweep's scope (Rust string literals are code, not comments). R4 correctly reported
rather than edited. Not tracked elsewhere; this NOTE is the record.

Kin: `NOTE-a-function-type-prints-in-a-spelling-you-cannot-write.md`,
`NOTE-a-runtime-census-cannot-see-a-dormant-minter.md`, `NOTE-the-guides-are-not-executable.md`.

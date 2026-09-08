# NOTE — `is-type?` shares the blindness the P-1 floor exposed

**2026-09-08.** Stone Q shipped `:wat::runtime::is-type?` as *"one authority over three
mechanisms."* Stone P-1's floor proved the authority is over **two stores of at least four**, and
`is-type?` is built on the same pair. Recorded beside Q rather than edited into it.

## The evidence

`src/reflect/verbs.rs:1546` `eval_is_type`, its whole membership test:

```rust
let stripped = type_kw.strip_prefix(':').unwrap_or(&type_kw);
let known = types.contains(&type_kw) || crate::runtime::is_builtin_primitive(stripped);
```

`TypeEnv::contains` (`src/types.rs:601`) is `types ∪ builtin_names`. So `is-type?` asks:

```
TypeEnv.types · builtin_names · is_builtin_primitive       asked
UseDeclarations       (src/rust_deps/mod.rs:290)           NOT asked   :rust::* FFI imports
subtype_edges         (src/types.rs:542)                   NOT asked   derive markers
```

P-1's floor named the consequence in real code: `:rust::test::Greeting` (a `use!`d FFI type) and
`:t::Marker` / `:wat::spawn::Spawned` (derive markers) are all legitimate, usable annotations that
the two-store question calls unknown.

## ⚠ Status of this claim

**Source-derived, not yet measured.** It is read directly off `eval_is_type`'s body, and the same
predicate demonstrably produced five false refusals inside `first_unknown_named_type` on the P-1
floor — but `is-type?` itself has not been asked these names. The probe is one file:

```wat
(:wat::core::use! :rust::sqlite::Connection)
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::runtime::is-type? :rust::sqlite::Connection))   ;; predicted false
  (:wat::kernel::println (:wat::runtime::is-type? :wat::spawn::Spawned)))       ;; predicted false
```

It was not run because a peer build held the binary; a number taken mid-rebuild is an instrument
artifact. Run it before acting on this note.
`[[feedback_state_what_the_instrument_can_see_before_quoting_it]]`

## What follows

Whatever union RELAND-1 lands for the annotation wall, **`is-type?` should consume the same one** —
not a parallel copy. Two predicates answering "is this a type?" from different store sets is the
exact shape this campaign keeps finding. `[[feedback_a_name_checked_against_a_partial_set]]`

⚠ And it sharpens Q's own ruling. *"`type-of` is STRUCTURE; `is-type?` is MEMBERSHIP"* still holds —
but membership over WHICH set was never pinned, and "the union" turned out to be a moving number.
`HAERESIS EST ITERVM ROGARE`.

# NOTE — `variant-name` reports an enum as not-an-enum

**Observed 2026-09-08** while probing for stone P-1. NOT P-1's subject; recorded so it is a visible
work item rather than a thing I noticed and let die in the conversation.
`[[feedback_nothing_blocks_it_is_not_a_work_item]]`

## Measured

```wat
(:wat::core::variant-name (:wat::core::Option::Some {:value 42}))
```

```
#wat.runtime/TypeMismatch {:message ":wat::core::variant-name: expected enum, got
wat::core::Option `(Some 42)`" :op ":wat::core::variant-name" :expected "enum"
:got {:type "wat::core::Option" :rendered "(Some 42)" :provenance nil}}
```

Run against `target/release/wat` at `8022e21b7`, RUN EXIT=1.

## Why it is worth a note

`:wat::core::Option` **is** an enum — `wat/core.wat:2125` declares it with `defenum`, and the same
session's `type-of` answers `:kind #wat.runtime/TypeKind.Enum {}` for it. So the diagnostic's
`expected "enum", got "wat::core::Option"` is self-refuting on its face: the thing it names as not
an enum is the enum.

⚠ **The mechanism is UNMEASURED.** Do not theorise from this note. The two obvious shapes — the
receiver arrives as an already-constructed `Value` whose runtime carrier is not what the verb's
guard tests for, versus a name/spelling comparison in the guard — predict different fixes, and this
campaign has been burned repeatedly by the second class (a `format!`/`split`/`==` on names, three
times in arc 278 alone). Read the guard before proposing.
`[[feedback_an_adjacent_implementation_is_not_the_subject]]`

## What it is NOT

- Not blocking P-1, P-2, or the head migration.
- Not evidence the enum ctor is wrong — the value printed correctly (`#wat.core/Option.Some
  {:value 42}`) in the same program, one line earlier.

# NOTE — a bare enum variant is an OPTIONAL SLOT. Make it illegal. (2026-09-06)

> **Builder:** *"i want it illegal.... not a now thing.... having this optional behavior is very
> unwanted..... our formatter should insert empty vec... not omission."*

## THE FACT — measured, both spellings legal today

```
(:wat::core::defenum :bv::Ev :wat::enum::Pure
  :Shutdown                                  ← BARE.  wat --check  exit 0
  :Admin [msg <- :wat::core::String])

(:wat::core::defenum :ev::Ev :wat::enum::Pure
  :Shutdown []                               ← EMPTY VECTOR.  wat --check  exit 0
  :Admin [msg <- :wat::core::String])
```

**81 bare variants in `wat/` alone** (`wat/program.wat:14-16` — `:thread` / `:process`).
`wat/spawn.wat:195-203` mixes both spellings inside ONE enum.

## ⛔ WHY IT MUST GO — an optional slot is a shifting index, and it has bitten twice

This is not an aesthetic objection. **An optional slot changes the meaning of every child position
after it**, and arc 277 was cut by exactly that twice in one day:

- **`Slot glued=3` mis-indexed a GENERIC `fn`.** `fn`'s grammar is `(fn [params] -> :RetType body)`,
  so the return type is child 3 — but `(fn :- [T] [params] -> …)` has an optional param-spec that
  shifts everything by two, and the rule withheld the wrong child. The fix was to stop indexing and
  read the FORM (`[[DESIGN-STONE-the-form-is-the-authority-for-the-arrow]]`).
- **`defenum` was the one form the layout rules had to special-case**, precisely because bare and
  field-carrying variants share one child list with no positional regularity.

★ **And the builder has already ruled the same question the other way for `defn`:** an empty
arg-spec `[]` is **not** an exception — it gets its own line like any other. `defenum` is the same
question and deserves the same answer: **one spelling, no optional slot.**

## ⭐ THE MIGRATION IS ALREADY BUILT — the formatter is the codemod

> **Builder:** *"our formatter should insert empty vec... not omission."*

⚠ **This is a CATEGORY CHANGE for wat-fmt and it must be stated, not slipped in.** Every rule to
date decides only **whitespace and line breaks**. `[[NOTE-the-double-underscore-contagion]]` drew the
scope line explicitly: *"`wat fmt` formats LAYOUT. It never touches a NAME."* **Inserting `[]` adds a
TOKEN to the source.**

It is defensible, and the reason is precise:

```
a RENAME       can capture, shadow, change meaning     -> not the formatter's (the `__` sweep is a codemod)
inserting `[]` both spellings are legal and EQUIVALENT -> value-preserving, like whitespace
```

⭐ **And it makes the formatter the CURE rather than a suggestion**, which is
`[[SELF-FIXING-TOOLCHAIN]]`'s whole claim: *"a tool alone is a suggestion. A tool PLUS a rule that
finds every place the old form survives and rewrites it is a CURE."* Run `wat fmt` over the corpus
and the 81 sites converge; **then making the bare form illegal is a checker change with ZERO
migration**, because the formatter already did it.

## THE ORDER — and it is the whole reason this is a NOTE and not a stone

```
1  wat fmt INSERTS `[]` for a bare variant          ← 277, and it is the migration
2  the corpus is formatted, so 0 bare variants remain
3  the CHECKER refuses a bare variant               ← arc 109, and it costs nothing by then
```

⛔ **Doing 3 before 1 breaks 81 sites and needs a `wat-fix` codemod** (R21). Doing 1 first makes 3
free. **The order is the point.**

## ⚠ WHAT IS NOT DECIDED HERE

- **Whether wat-fmt should insert tokens in OTHER cases.** This NOTE argues the `[]` case on
  value-equivalence; it does not open a general licence. **A second insertion wants its own
  argument.**
- **Whether the checker's refusal is a hard error or a lint.** Arc 109's call.
- **The `:examples` fat arrow** — unrelated, deferred, `[[NOTE-the-examples-container-wants-a-fat-arrow]]`.

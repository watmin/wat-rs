# SCORE — STONE 251.8b: the tuple is stored, not re-derived

No commit. Floor and clippy left to the orchestrator. This stone's diff is
`crates/wat-reader/src/identifier.rs` only. Behaviour is identical, including
the currently-wrong `wat.core//` split.

`Identifier` holds `(ns, name)` plus the flat spelling (so every accessor keeps
`-> &str`) and `scopes` alongside. The split runs once in `bare`. `namespace()`
returns `&self.ns`.

---

## Row 1 — it builds

`cargo test --release -p wat-reader` **110 + 2 passed**. `-p wat-macros --lib` **91 passed**.
`one_name_grammar` **5 passed**.

## Row 2 — the tuple is STORED

```
foo            ns="$bound"     name="foo"
wat.core/+     ns="wat.core"   name="+"
```

`namespace()` is `fn namespace(&self) -> &str { &self.ns }`. A field, not a `rfind` result.

## Row 3 — every accessor keeps its `&str` signature

`as_str` · `namespace` · `leaf` · `path` · `receiver` · `method` · `deprimed` — all still `-> &str`.
The flat spelling is stored so `as_str` has something to point at. 251.8a's promise held; no
signature change across the 147 `bare` sites.

## Row 4 — `foo` is `[$bound, foo]`

`Identifier::bare("foo").namespace()` = `$bound`, `is_reference()` = `false`, stored `name` = `foo`.

## Row 5 — `wat.core/+` is `[wat.core, +]`

`namespace()` = `wat.core`, stored `name` = `+`, `method()` = `+`.

## Row 6 — NO accessor re-derives

`rfind('/')` in accessor bodies (`as_str`/`namespace`/`leaf`/`path`/`receiver`/`method`/`prime`/`deprimed`)
= **0**. The one `rfind('/')` on `Identifier` is in `bare` (row 7). Free-function `receiver`/`method`
on `&str` still split — they are the keyword-raw-string grammar, not Identifier accessors.

`receiver()` asks `flat.contains('/')` only to distinguish a binder (`""`) from `$bound/foo`
(prefix `$bound`): both have stored ns `$bound`, so the tuple alone cannot tell them apart. It
does not split. The prefix itself is `&self.ns`.

## Row 7 — construction derives ONCE, at the chokepoint

`Identifier::bare` is the only construction. Last-`/` split happens there. `add_scope` copies the
already-stored fields. 147 call sites untouched.

## Row 8 — `scopes` is untouched and NOT in the tuple

`add_scope` / `scopes()` unchanged. Hygiene rides alongside. Eq/Hash still key on `(flat, scopes)`
so identity matches today. Debug still prints `name` as the spelling, so `{:?}` does not grow a
third name member.

## Row 9 — behaviour is IDENTICAL for every existing input

`identifier_accessors_match_the_free_functions_on_todays_inputs` — `foo`, `wat.core/+`,
`wat.core//`, `:S/mk`, `:wat::cache::Lru`, `:sort'`, `:sort'/apply`, `x`, `$bound/foo`. Every
accessor equals today's free-function / last-`/` answer. Existing 251.8a tests still pass.

## Row 10 — including the shapes that are currently WRONG. REPORTED, not fixed.

`wat.core//` today:

| | today (stored) | builder's model |
|---|---|---|
| namespace | `"wat.core/"` | `"wat.core"` |
| name / method | `""` | `"/"` |

Last-`/` split: the final slash is the boundary, so the ns keeps a trailing slash and the name
is empty. Five live division operators sit on this shape. **This is a reader question. Not this
stone.** Changing it would make row 9 unprovable. Next stone.

## Row 11 — nothing outside the crate, from this stone

`git diff --stat -- crates/wat-reader/` = `identifier.rs` only.

**Named, not this stone.** The tree already carries uncommitted work from
`SCORE-STONE-pprintln-is-the-one-printer` and `SCORE-STONE-a-keyword-is-a-keyword`
(`src/edn/render.rs`, `src/services/verbs.rs`, `src/load/stdlib.rs`, goldens, `wat/doc.wat`).
Not reverted. Not part of this diff's justification beyond naming them.

## Row 12 — negative control

Reverted `namespace()` to `rfind` on `flat`. `namespace_borrows_the_stored_field` went **RED**
at `identifier.rs:430`: `namespace() must return the stored ns field` (`ptr::eq` against
`self.ns`). Restored; test **passed**. A value-equal rfind would still pass every other test —
this one asserts the borrow is the field.

## Row 13 — floor (ORCHESTRATOR)

Not run.

## Row 14 — clippy (ORCHESTRATOR)

Not run.

---

## Targeted checks (executor)

```
cargo test --release -p wat-reader          110 + 2 passed
cargo test --release -p wat-macros --lib    91 passed
cargo nextest run --release --test lint -- one_name_grammar   5 passed
```

## Files

```
crates/wat-reader/src/identifier.rs    the representation + the accessor bodies + rows 4–6, 9, 10, 12
```

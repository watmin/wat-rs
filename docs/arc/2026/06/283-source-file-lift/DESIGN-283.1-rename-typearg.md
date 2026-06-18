# Arc 283.1 — `rename-keyword-prefix`: reach type-args + stop corrupting prefix-siblings

> **STATUS: STRIKE-READY (2026-06-17).** RED probe `tests/probe_arc283_1_rename_typearg.rs`
> (`#[ignore]`'d). The 283 dogfood surfaced this: `rename-keyword-prefix` is start-anchored, so it
> (a) MISSES a symbol used as a type-argument (`Vector<t::Old>`), and — found by the probe —
> (b) CORRUPTS any prefix-sibling (`:t::OldExtra` → `:t::NewExtra`, because it has no boundary check).
> Bug (b) is an L1: silent data corruption. Annihilate both with one boundary-aware whole-name rewrite.

## The two bugs (proven)

Renaming `:t::Old` → `:t::New` over
`(:u::f [xs <- :wat::core::Vector<t::Old> y <- :t::OldExtra] -> :t::Old (:t::Old/make xs))`
yields at HEAD:
`(:u::f [xs <- :wat::core::Vector<t::Old> y <- :t::NewExtra] -> :t::New (:t::New/make xs))`
- `Vector<t::Old>` **unchanged** — the type-arg is embedded in a keyword starting `:wat::core::Vector`,
  so `starts-with? name ":t::Old"` is false (bug a, the gap).
- `:t::OldExtra` → `:t::NewExtra` — `starts-with?` matched, the prefix was spliced, `Extra` dangled
  (bug b, corruption — no boundary check exists today).

The current logic (`fix.wat:559-573`): `if (starts-with? name old-prefix) → splice prefix`. Start-anchored,
no boundary, no embedded-occurrence handling.

## The fix — boundary-aware whole-name rewrite (THE CONTRACT)

Replace the leaf logic with: for **every** keyword leaf, compute `new-name` by rewriting **every valid
occurrence** of the colon-stripped old-prefix → colon-stripped new-prefix within the name; if
`new-name != name`, emit `(off, length(name), new-name)`.

**Colon-strip:** `old-bare = strip-leading-colon(old-prefix)` (e.g. `:t::Old` → `t::Old`); same for
`new-bare`. (Strip the first char iff it is `:`.) Matching the colon-stripped form subsumes BOTH the
head occurrence (right after the keyword's leading `:`) AND the type-arg occurrence (inside `<…>`).

**A match of `old-bare` at char-index `i` in `name` is VALID iff ALL hold** (this is the whole bar-raise):
- **present:** `i + len(old-bare) <= len(name)` and `subs(name, i, i+len(old-bare)) == old-bare`.
- **left-valid:** `(i == 1 && char-at(name,0) == ":")`  — the head, right after the single leading colon
  — **OR** `char-at(name, i-1) ∈ { "<", ",", " " }` — a type-argument position.
  (This EXCLUDES `:other::t::Old` — there `t::Old` is preceded by `::` namespace separator, neither
  case — so an unrelated symbol that merely ends in the path is never touched.)
- **right-valid:** `i + len(old-bare) == len(name)` (end) — **OR** `char-at(name, i+len(old-bare))` is
  NOT an identifier-continuation char, i.e. ∉ `[a-zA-Z0-9_-]`. (`>` `,` `/` `:` space all terminate →
  valid; a letter/digit/`_`/`-` → INVALID, so `:t::OldExtra` is left alone. `:` is a terminator so
  `:t::Old::Variant` / `:t::Old/make` cascade correctly.)

**Implementation — a char-walk (uses `subs`, the arc-281 keystone):** a recursive helper over the index:
```clojure
(rename-in-name name old-bare new-bare i acc):
  i >= len(name)                                  → acc
  valid-match-at(name, i, old-bare)               → (recur name … (+ i (len old-bare)) (concat acc new-bare))
  else                                            → (recur name … (+ i 1) (concat acc (subs name i (+ i 1))))
```
`valid-match-at` = present ∧ left-valid ∧ right-valid (above). `is-ident-char?` = a `HashSet`/range check
over the char (or `string::contains?` of an identifier-charset string). Then in `rename-prefix-edits`:
`new-name = (rename-in-name name old-bare new-bare 0 "")`; `if (!= new-name name)` emit the whole-name edit.

## Proof

- `tests/probe_arc283_1_rename_typearg.rs` (un-ignore): renaming `:t::Old`→`:t::New` over the fixture →
  `Vector<t::New>` (type-arg), `-> :t::New` + `:t::New/make` (head + accessor still rename),
  `:t::OldExtra` UNCHANGED (boundary), no `Vector<t::Old>` survivor.
- deftest in `wat-tests/` (a fix-tool test home, or beside the existing rename tests if any) — same cases.
- **Floors unchanged** (this only hardens a tool): lib 929/36, deftest, deporder 0.
- The arc-269 vehicle that first used `rename-keyword-prefix` must still pass (regression — start-anchored
  renames still work; that is the head case, now a sub-case of the boundary rule).

## Out of scope (rejected, not deferred)

- A full type-expression parser for keyword names — the boundary rule is exact for the qualified-name +
  `<type-args>` + `/accessor` + `::Variant` grammar wat uses; a parser is gold-plating here.
- Renaming inside STRINGS / comments — `rename-keyword-prefix` operates on keyword AST leaves only
  (comment-faithful is the whole point); embedded wat in Rust `r#"…"#` strings stays manual (noted in 283).

## Four questions

- **Obvious?** YES — "rename the symbol everywhere it appears as that symbol, nowhere it doesn't" is what
  a rename should always have meant.
- **Simple?** YES — one boundary predicate + one char-walk replaces the start-anchored splice; the head
  case becomes a sub-case of the general rule.
- **Honest?** This is the point — bug (b) made the tool silently lie (corrupt siblings); the boundary
  guard makes the rename only ever touch the real symbol. Proven by the decoy surviving.
- **Good UX?** YES — every future type rename (the sweep, arc 282) now Just Works through `Vector<>`/
  `Option<>`/`HashMap<>` and never corrupts a neighbor.

## Blast radius

`wat/fix.wat` — `rename-prefix-edits` rewritten to the boundary-aware whole-name rule + the
`rename-in-name` char-walk helper + `is-ident-char?`/`left-valid`/`right-valid` helpers. A wat deftest.
Un-ignore the probe. No Rust changes (rides `subs`/`length`/`string` already present). THEN 283 re-runs
its dogfood (now total) on top of this hardened tool.

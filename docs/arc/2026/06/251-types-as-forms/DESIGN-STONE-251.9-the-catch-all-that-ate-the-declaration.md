# DESIGN — STONE 251.9: the catch-all that ate the declaration

**Phase 1 of the builder's three-phase ruling** (*"we need to update the internal such that
keyword-as-symbol and symbol are handled correctly.. then we move the code.. then we rip keyword
support from defs and heads"*). This stone closes the ONE failure mode in phase 1 that is **silent**.
Everything else in phase 1 already fails loudly, and loud failures do not need a stone to be safe.

## The defect, measured

A symbol-headed declaration is **not refused — it evaporates.**

```
(:wat::core::defenum :probe::Color :wat::enum::Pure [Red Green])   EXIT=3  "malformed enum variant"
(wat.core/defenum    :probe::Color :wat::enum::Pure [Red Green])   EXIT=0  ⛔ SILENT
```

Identical forms, both malformed. The legacy spelling is correctly rejected; the symbol spelling
**passes clean**, because the parser that would have rejected it never ran. Referencing the type
afterwards proves nothing registered (`ReturnTypeMismatch` — `:probe::Color::Red` is unknown).

The site is `src/declare/parse.rs:199` `is_declaration_form`:

```rust
let head = match &items[0] {
    WatAST::Keyword(k, _) => k.as_str(),
    _ => return false,          // ⛔ a Symbol head → "this is not a declaration"
};
```

`false` sends the form to `FormOutcome::Evaluated`. It evaluates, yields a value, and **grows the
session by nothing.** No error, no diagnostic, no test failure — the floor cannot see a declaration
that was never made.

⚠ This is not a type hole. `#95` (`infer_list`'s gate) is a separate, LOUD-once-widened defect and
is NOT this stone.

## The class, and why a hand census cannot close it

The builder named it: *"this code base has been plagued with underscore matches being poorly
handled."* Every `_ =>` in a head reader turns *"a Symbol arrived"* into *"this isn't the thing"*.
A census of the victims treats the stem. The wall treats the root: **remove the catch-all from the
head readers so a Symbol has an explicit answer, and let the compiler enumerate the sites.**

This file already carries the scar from the same neighbourhood. `DECLARATION_HEADS`'s own doc
records a shipped bug where ONE LIST ANSWERED TWO QUESTIONS — a top-level `let` classified
`Declared`, its value discarded, `wat --repl` printing nothing.

## THE ONE CONTRACT DECISION

**One door for reading a form's HEAD; names are out of scope.**

```rust
/// The FQDN of a form's head, whatever spelling wrote it. `None` = this node is not a head
/// that names anything (a literal, a vector, a bound symbol).
pub(crate) fn head_fqdn(node: &WatAST) -> Option<Cow<'_, str>>
```

- `WatAST::Keyword(k, _)` → `Cow::Borrowed(k)` — byte-identical to today.
- `WatAST::Symbol(id, _)` where `id.is_reference()` → `Cow::Owned(ns_to_wat_path(id.receiver(), id.method()))`.
- everything else, including a `$bound` symbol → `None`.

**Why the symbol mapping is safe HERE and nowhere else.** `ns_to_wat_path` is the function stone
*a keyword is a keyword* was halted over (`fc870908c`) — it cannot disambiguate `Type/member`. It is
safe at this door because **every caller immediately tests set-membership or literal equality**
(`DECLARATION_HEADS.contains`, `k == ":wat::core::def"`). A mis-mapped head therefore yields *"not
this form"* — the same answer the catch-all gives today — never a **wrong** form. The guess has no
path to a wrong decision, which is the difference between this door and the halted one.

### ⛔ NAMES ARE OUT OF SCOPE — affirmatively cut, not deferred

A declared NAME cannot be mapped **to the registry's current keying**. `probe.Note/make` would
have to become `:probe::Note/make` or `:probe::Note::make`; segment-counting does not discriminate
(`wat.core/first` is a legitimate 2-segment receiver) and capitalisation is ruled out by the builder.

⚠ **AMENDED 2026-09-06 — the ambiguity is an INTERIM LOOKUP problem, not a language problem.** The
TUPLE was never ambiguous: `wat.core.HashMap/length` is `[wat.core.HashMap, length]` — the type is a
NAMESPACE SEGMENT, and `Type::member` vs `Type/member` was one thing spelled two ways. The builder,
2026-09-06: *"`Type::member` … `Type/member` … is just a syntax we should never have had"*, and its
end state is not a mapping at all but per-namespace verbs — `(wat.core/length x)` dispatching, with
`wat.map/length` / `wat.vec/length` as the concrete homes (109's domain). The dot notation is
modelled **only for variants** (`probe/Color.Red` — arc 296 stone H), where the dot sits in the NAME
half; a dot LEFT of the slash is ordinary namespace nesting.

So the interim mapping has **two** cures, not one:
- **(a)** arc 255 closes `is_resolvable_call_head`'s reserved-prefix shortcut (it accepts a `:wat::`
  namespace without validating the leaf — `src/resolve/normalize.rs:424`) so the registry can answer.
- **(b)** re-key the registry on the TUPLE, which is what 251.8b set up by making `Identifier` STORE
  `(ns, name)`. Then nothing is mapped back and `ns_to_wat_path` is deleted outright — already on
  arc 251's own FINAL MEASUREMENT list (`DESIGN.md`: the codec's ~27 sites go).

Either cure unblocks names. Neither is this stone.

This costs nothing, because **names already fail LOUDLY**: `"malformed :wat::core::defenum
declaration: name must be a keyword; got symbol"` (`src/types.rs:4778` and siblings). A loud refusal
is a correct interim state. Tracked as the successor stone, gated on 255; NOT deferred prose.

### Also out of scope, affirmatively

- **TYPE readers** (`parse_type_keyword` at `declare/parse.rs:586·718·875`, `function/parse.rs:511`) —
  owned by 251.2 (type namespace) / 251.3 (parametrics-as-forms), both STRIKE-READY with probes.
- **Marker keywords stay keywords forever** — metadata-map keys (`declare/parse.rs:322`),
  `:guard`/`:ensure` (`function/parse.rs:417·426`), and every `WatAST::Keyword(…)` that CONSTRUCTS
  rather than reads. The clojure line is: *a keyword that NAMES something becomes a symbol; a
  keyword that IS a marker stays one.* A blanket `_`-removal would break these.
- **`#95`** — `infer_list`'s gate. Measured separately: 10 lines, 0 compiler errors, floor 5206/5206.
- **The wat macros** (`wat/Record.wat:171`, `wat/core.wat:2048` reading a name via
  `:wat::keyword::to-string`) — that is the `wat.symbol/*` vocabulary stone, and it is a NAME
  problem, cut above.

## The room map (hand-classified from a grep — THE COMPILER IS THE AUTHORITY)

```
src/declare/parse.rs         205 · 236 · 267 · 278 · 362 · 413 · 546 · 563 · 666 · 689 · 888
src/declare/preregister.rs    77 · 123 · 278 · 415 · 488
src/declare/register.rs      274 · 285 · 290 · 531 · 778 · 786 · 1767
```

23 sites by hand. **Do not trust that number** — this session produced four census errors that were
each a pattern rather than a site. Removing the catch-all makes the compiler emit the real list;
that list is the population, and any site the hand map missed is a finding to record, not a surprise
to absorb.

## Four questions

| | |
|---|---|
| **Obvious?** | YES — "a Symbol reached a head reader" gets a named answer instead of `false` |
| **Simple?** | YES — one door, one act repeated; the compiler produces the worklist |
| **Honest?** | YES — a silent pass has no representation left; the population comes from the compiler, not from me |
| **Good UX?** | YES — during the drive, an unconverted site says so at its own line instead of vanishing |

## What this buys phase 3

With both arms explicit at one door, *"rip keyword support from heads"* is **deleting the Keyword
arm of `head_fqdn`** — one edit, and no `_` grows back to hide the next case.

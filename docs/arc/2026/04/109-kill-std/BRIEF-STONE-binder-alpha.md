# BRIEF — arc 109, binder strike α: the seven Rust declaration parsers accept `:- [T …]`

Every `def*` gains an OPTIONAL param-spec immediately after its name. α does the **Rust** half —
the seven parsers that read a declaration's name — and touches no `.wat` file at all.

```clojure
(:wat::core::defenum :user::E :- [T] :wat::enum::Pure  :A [f :- T])
(:wat::core::typealias :user::A :- [T] (:wat::core::Vector :- [T]))
```

Design: `DESIGN-STONE-the-declaration-binder.md` (sibling). Read it first — it carries the measured
RED baseline for all seven and the two-layer room map.

**This stone is ADDITIVE.** The name-embedded `<T>` spelling keeps working everywhere. ③ hard-cuts.
A change that makes `:my::ns::Wrapper<T>` stop parsing is a defect in this stone, not progress.

## Read in order

1. **`src/types.rs:4247`** — `parse_declared_name(head, form, decl_span) -> (String, Vec<String>)`.
   The shared name parser: it splits the name keyword at `<`, validates each param
   (`:4292`: rejects whitespace, `<`, `:`), and returns `(raw_name, params)`. It is the ONLY thing
   that populates `type_params` today.
2. **`crates/wat-reader/src/identifier.rs:145`** — `namespace()` and `is_reference()`.
   `namespace()` answers `$bound` for any name with no `/`; `is_reference()` is exactly
   `namespace() != BOUND_NAMESPACE`. **This is your validation predicate — do not hand-roll one.**
3. The seven callers, each `fn parse_X(args: Vec<WatAST>, decl_span: Span, …)`, each doing
   `let mut iter = args.into_iter(); let name_kw = iter.next()…; parse_declared_name(…)`:

```
src/types.rs:3782         parse_defenum
src/types.rs:3972         parse_newtype
src/types.rs:4014         parse_typealias
src/types.rs:4065         parse_typeunion
src/types.rs:4168         parse_aggregate      ← recordtype + aggregatetype (defrecord LOWERS here)
src/types/surface.rs:530  parse_defsurface
src/types/defstruct.rs:520 parse_defstruct     ← defstruct LOWERS here
```

## The work — ONE door, seven call sites

Add beside `parse_declared_name` in `src/types.rs`:

```rust
/// Consume an optional `:- [T …]` binder from the arg stream, immediately after the name.
/// `name_params` is what `parse_declared_name` read from the name's `<…>` spelling.
fn take_declared_binder<I: Iterator<Item = WatAST>>(
    head: &str,
    name_params: Vec<String>,
    name_span: &Span,
    iter: &mut std::iter::Peekable<I>,
) -> Result<Vec<String>, TypeError>
```

Behaviour:

- **No binder present** → return `name_params` unchanged. Every existing form is untouched.
- **Binder present** (`iter.peek()` is `WatAST::Keyword(":-")`) → consume it AND the
  `WatAST::Vector` that must follow, and return the vector's entries as the params.
- **Both present** — `name_params` non-empty AND a binder → `TypeError`. Two binders on one
  declaration is a contradiction; it arises only from a half-applied codemod, never from someone
  writing it, and it must not silently pick one.
- **Each entry must be a bare Symbol** whose `Identifier::is_reference()` is **false** — i.e. no
  `/`, so `namespace()` answers `$bound`. That single predicate rejects all three bad shapes with
  one diagnostic:
  ```clojure
  :- [:a :b]           ; keyword VALUES, not names
  :- [U [F :-> T]]     ; a function TYPE, not a name
  :- [T [f :- T]]      ; a field vector nested into the binder
  ```
- **Store bare names** in `type_params: Vec<String>` — the same strings
  `parse_declared_name` produces. ⛔ **NEVER write `"$bound/T"`.** `identifier.rs:145`'s own doc:
  *"the namespace is DERIVED from the spelling … 251.8b is where derived swaps for stored."*
  Encoding the derived namespace into storage is the artifact 8b exists to remove.

Then each of the seven becomes, mechanically:

```rust
let mut iter = args.into_iter().peekable();          // was .into_iter()
let name_kw = iter.next()…;
let (name, name_params) = parse_declared_name(HEAD, &name_kw, &decl_span)?;
let type_params = take_declared_binder(HEAD, name_params, name_kw.span(), &mut iter)?;
```

Everything downstream of that line in each parser is UNCHANGED — the binder is consumed before
their first positional slot, so `defenum`'s purity marker, `defsurface`'s `:nature`, `typealias`'s
expression and the rest all still arrive where they already arrive.

## Blast radius

`src/types.rs` (one new fn, five call sites) · `src/types/surface.rs` (one) ·
`src/types/defstruct.rs` (one). **No `.wat` file. No macro. No lexer. No change to
`parse_declared_name`'s existing `<…>` path. `defn` is NOT in this stone** (it is γ, and its macro
is the other layer).

## STOP triggers

1. **STOP-1** — if any `<T>`-spelled declaration stops parsing, STOP. Additive only.
2. **STOP-2** — if a parser cannot take a `Peekable` without restructuring how it consumes its
   positional slots, STOP and report which one. Six adapting and one fighting is a finding about
   that parser, not a reason to special-case it.
3. **STOP-3** — if you find yourself writing a check for "is this a binder name" that is not
   `Identifier::is_reference()`, STOP. 251.8a collapsed four hand-rolled versions of that question
   into one door; a fifth is the defect that stone removed.
4. **STOP-4** — if the both-spellings error cannot be raised inside `take_declared_binder` because
   a caller has already consumed something, STOP and report the caller.

## How this lands

You are a rider. **Text edits only.** The orchestrator builds, floors and clippies centrally, once,
after the tree is quiescent. Do not run cargo, do not commit, do not stash, do not revert. Run
everything in the FOREGROUND — your turn ends when your edits are on disk and your report is
written, and ending your turn ends you.

Report: the diff per file; the exact text of each new diagnostic; which parsers took the
`Peekable` cleanly and which needed more than the four-line shape above; anything that contradicted
this brief on disk — a previous rider on this arc was right to refuse a correction of mine that was
wrong, and the brief is my claim, not the ground.

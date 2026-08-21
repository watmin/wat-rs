# BRIEF — arc 109 Stone ②-i-b: `:-`, the parameterization operator (+ the Tuple arm)

`:-` declares **"the thing on the left is parameterized by the thing on the right."** It already
says that for an arg-spec and for a ret-type. This stone makes it say it for a parametric type's
type-arguments too — the same declaration, the same operator — and finishes the one renderer arm
②-i scoped out.

```clojure
[n :- wat.type/i64]                                              arg-spec      (already)
:- wat.type/i64                                                  ret-type      (already)
(:wat::core::Vector :- [:wat::core::i64])                        type args     (this stone)
(:wat::core::Tuple :- [:wat::core::keyword :wat::core::keyword] :some :keyword)   constructor
```

Design: `DESIGN-STONE-2i-b-the-parameterization-operator.md` (sibling). Read it first.

**Dual-read.** This stone ADDS the `:-` spelling and makes the renderer EMIT it. The unmarked
`(Head [types])` keeps parsing exactly as today. ③ hard-cuts the unmarked form — that is the
campaign's existing rhythm and it is not yours to shortcut.

## The RED baseline (measured at HEAD, this is what you are turning green)

```
(:wat::core::defn :user::f [p :- (wat.type/HashMap :- [wat.type/String wat.type/i64])] :- wat.type/i64 1)
→ malformed :wat::core::fn form: invalid type keyword: malformed type expression "[…]":
  function-type bracket needs a `:->` arrow: `[arg… :-> ret]`
```

The `:-` becomes arg #1 and the bracket becomes arg #2, so the args-tail no longer looks like a
lone Vector, the bracket-unwrap does not fire, and the bracket falls through to the standalone
function-type production. That error text is your before-picture.

## Read in order

**The type position — one production.**
1. **`src/types.rs:4528`** — the args-tail match inside `parse_type_node`. Today:
   `[WatAST::Vector(inner, _)] => bracketed`, else positional. It gains a second bracketed arm for
   `[WatAST::Keyword(":-"), WatAST::Vector(inner, _)]`. Four lines. Everything downstream —
   including the `raw_head == "wat::core::Tuple"` branch at `:4540` — reads `args` after this and
   needs no change.

**The value position — two shared helpers, twelve call sites, and you touch the helpers.**
2. **`src/check.rs:11993`** `unwrap_type_param_bracket` — the UNCONDITIONAL splice used by
   `Vector`/`HashMap`/`HashSet`. Call sites: `check.rs:3005, 3145, 3207` and
   `runtime.rs:6230, 6466, 6505`. Adding a leading-`:-` arm here is internal; **no call site changes.**
3. **`src/check.rs:12027`** `is_type_bracket_candidate` — the CONDITIONAL sniff used by
   `Tuple`/`PersistentMap`/`PersistentVector`, whose first arg may legitimately be a data vector.
   Call sites: `check.rs:14062, 14165, 14330` and `runtime.rs:6257, 6479, 6494`.
   **Read its doc comment before you touch it** — it names this stone's whole reason for existing.

**The renderer.**
4. **`src/edn_shim.rs:1306–1312`** — the `Parametric` arm's arg-bracketing. It gains the operator.
5. **`src/edn_shim.rs:1322–1332`** — the `Tuple` arm. Mode-blind and flat today; it gets what
   `Parametric` has: the 4-way head ladder, bracketed args, and the operator.
6. **`src/edn_shim.rs:1216–1221`** — the fn-doc bullets that document Tuple as out-of-scope for
   `mode` and the empty tuple as `(wat.type/Tuple)`. Both become false; the doc is deliverable.
7. **`src/edn_shim.rs:1364`** — the one parse call that switches to the preserving entry point.
8. **`src/types.rs:4334`** — `parse_type_expr_with_span`; the preserving sibling goes beside it.

## The discrimination rule, and the ONE new wall

The builder's rule, verbatim:

> *"`(:wat::core::Tuple :- [])` — in an arg-spec or type-spec (any spec that prefixes their
> receiving type with ` :- `) this is a **type declaration** for a tuple that has no member; in any
> other location its a **tuple literal** with no members."*
>
> *"`(:wat::core::Tuple :- [:wat::core::i64 :wat::core::keyword] 42 :some-keyword)` — this is
> **illegal in an argspec**, as its a literal.. param-spec must never have any initial values for a
> type declaration, param-spec may have initial members in any other location."*

The first half needs no code: the two positions are already different productions, so the same bytes
mean a type inside a `:-`-prefixed spec and a literal everywhere else, exactly as `[a b]` is a
binding vector in `let` and a literal elsewhere.

**The second half is a WALL and it is yours to build.** In `parse_type_node` (`src/types.rs:4460`+),
when the head is a parametric and the args tail is `[Keyword(":-"), Vector(inner), …rest]` with
`rest` NON-EMPTY, that is a literal sitting in a type slot. Emit a clean `MalformedTypeExpr` naming
exactly that — do not fall through to the positional arm.

Today the fall-through gives a diagnostic that names the wrong defect entirely:

```
[p <- (wat.type/Tuple [wat.type/i64 wat.type/keyword] 42 :k)]
→ "malformed type expression \"[…]\": function-type bracket needs a `:->` arrow: `[arg… :-> ret]`"
```

The author wrote a literal in a type slot; the checker told them their function type is malformed.
The new error must say what happened and what to do — carry a `remedy` if the surrounding error
shape supports one (see `crate::remedy::Remedy`, `src/check.rs:378`), otherwise a plain reason:

> a type declaration cannot carry initial values — `(Head :- [types] v…)` is a LITERAL, and a
> literal is not a type. Drop the values here, or move the form out of the type position.

★ **Do not build the mirror wall.** A literal with values in a VALUE position is legal and is the
whole point of the constructor form; nothing rejects it.

## The contract decisions, pinned

**One — a single door for the type-param bracket.** Do NOT add a `:-` arm to six pattern-matches.
Introduce ONE `pub(crate)` helper in `src/check.rs` beside the two it subsumes:

```rust
/// Split a constructor's args into (type-param bracket, its span, the values).
/// `:-`-marked  → the bracket is types BY DECLARATION; no content sniffing.
/// unmarked     → falls back to `is_type_bracket_candidate` (dual-read; ③ deletes this arm).
pub(crate) fn split_type_param_bracket<'a>(args: &'a [WatAST])
    -> Option<(&'a [WatAST], &'a Span, &'a [WatAST])>
```

The six conditional call sites call this instead of matching `Some(WatAST::Vector(inner, bspan)) if
is_type_bracket_candidate(inner)`. That puts ③'s cut behind one door: delete the unmarked arm and
`is_type_bracket_candidate` dies with it.

★ **When `:-` is present, the sniff MUST NOT run.** That is the entire point — a declared bracket is
types because it was declared, not because its contents looked typish.

**Two — the preserving parse.**
`pub fn parse_type_expr_preserving_with_span(kw: &str, span: &Span) -> Result<TypeExpr, TypeError>`,
byte-identical to `parse_type_expr_with_span` except `canonicalize=false`, and it **still calls
`reject_any`**. It returns `Result`, never `Option` — `parse_type_expr_audit` (`src/types.rs:4561`)
is the existing `canonicalize=false` path and it swallows errors into `None`, which is why the verb
cannot reuse it. This stops `src/types.rs:4728` collapsing `:wat::core::nil` into `Tuple(vec![])`,
so `nil` renders back as `nil`.

## What the renderer emits after this stone

```
:wat::core::nil                     →  :wat::core::nil
Result<nil,String>                  →  (:wat::core::Result :- [:wat::core::nil :wat::core::String])
:(i64,i64,String)                   →  (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64 :wat::core::String])
:(i64,)                             →  (:wat::core::Tuple :- [:wat::core::i64])
:()                                 →  (:wat::core::Tuple :- [])
:wat::core::Vector<i64>             →  (:wat::core::Vector :- [:wat::core::i64])
```

No rung emits a bare head. No rung is special-cased — one path, `args.len()` never consulted.
`:-` is a Keyword and is identical in both head-spelling modes.

## Three forms that are unwritable today and must be writable after

Each measured at HEAD. These are acceptance rows, not motivation:

| you want | today | after |
|---|---|---|
| a 2-tuple of keyword values | `(:wat::core::Tuple [:a :b])` → `ArityMismatch: expected 2, got 0` | `(:wat::core::Tuple :- [:wat::core::keyword :wat::core::keyword] :a :b)` |
| an EMPTY tuple literal | `(:wat::core::Tuple [])` → **`[[]]`**, a 1-tuple holding an empty vector | `(:wat::core::Tuple :- [])` |

⚠ The second row is why `split_type_param_bracket`'s `:-` arm must NOT inherit the sniff's
`!items.is_empty()` guard. Under `:-`, an EMPTY bracket is a type-param list of length zero — a
declaration — and the empty tuple literal it declares is a real value. Under the unmarked arm the
old guard stays exactly as it is (dual-read), which is why the two arms cannot share one rule.

## The goldens — do NOT grep for them

The renderer's output is pinned by golden files across `tests/`, and **the honest instrument for
finding them is the floor, not a search.** My own tight-looking pattern for "a container head
followed by a bracket" returned 27 files; a looser one returned 326, most of them `let` bindings.
Neither is the answer.

So: make the change, and the orchestrator's central floor run names the golden set by going red.
Update each red golden **to the predicted bytes above**, never to whatever the binary printed —
a golden rewritten from observed output cannot fail. If a golden's new content is not obviously an
instance of the table above, STOP and report it rather than blessing it.

## Blast radius

`src/types.rs` (one production, one new fn) · `src/check.rs` (two helpers → one door, six call
sites re-pointed) · `src/runtime.rs` (six call sites re-pointed) · `src/edn_shim.rs` (two arms, one
call site, the fn doc) · the goldens the floor names · the two probes in `wat-scripts/scratch-pad/`
gain `:-` rows.

**No lexer change. No change to `src/types.rs:4728`. No change to the `Path` or `Fn` arms — the
`Fn` arm is correct and is not yours. No corpus `.wat` migration — that is ②-iii.**

## STOP triggers

1. **STOP-1** — if the unmarked `(Head [types])` form stops parsing anywhere, STOP. This stone is
   additive; ③ does the cutting. A red that proves the OLD spelling died is a defect in this stone.
2. **STOP-2** — if the `:-` arm cannot be added to `split_type_param_bracket` without also changing
   `eval_*_ctor` in `src/collection/eval.rs`, STOP and report. ①b established that the callee fns
   stay untouched and the splice happens at the dispatch site; if that no longer holds, the shape
   changed and the orchestrator re-plans.
3. **STOP-3** — if switching the call site to the non-canonicalizing parse changes the rendering of
   anything other than a `nil`-derived type, STOP and report the input and both spellings.
4. **STOP-4** — if you find yourself writing a rule that decides "types or values" by inspecting a
   bracket's CONTENTS anywhere `:-` is present, STOP. That is the guess this stone exists to remove.
5. **STOP-5** — if the new wall (values in a type slot) cannot be raised inside `parse_type_node`
   without also rejecting the legal VALUE-position constructor `(Head :- [types] v…)`, STOP and
   report. The two live in different productions and must not need a flag threaded between them; if
   they do, the shape is wrong and the orchestrator re-plans.

## How this lands

You are a rider. **Text edits only.** The orchestrator builds, floors, and clippies — centrally,
once, after the tree is quiescent. Do not run cargo, do not build, do not commit, do not stash, do
not revert. Everything you do run, run in the FOREGROUND: your turn ends when your edits are on disk
and your report is written, and ending your turn ends you.

Report: the diff shape per file, which call sites you re-pointed and which you left, anything that
surprised you, and any STOP you hit.

# Arc 171 — Comma → apostrophe in fixed-arity dispatch suffix

**Status:** opened 2026-05-10 per arc 170 REALIZATIONS pass 14
decision. Sized at author time per pass 14 disposition.

**Sibling-coordinated with arc 172** (Scheme → Clojure macro
flavor swap). Both arcs together complete the lexical
EDN-compliance pivot that slice 1f-W opened (commas inside
`<>` were the first thread; commas in dispatch suffix +
commas as unquote-token are the remaining two).

## Motivation

Per REALIZATIONS pass 14 (lock-in 2026-05-10):

> *"we have scheme macros now.... we need to swap to clojure
> macros... we also need to impose our pending rules... type
> declarations may only be keywords. keywords may not contain
> underscores. underscores are reserved for swapping from
> commas when transmitting EDN ... :wat::core::HashMap<wat::core::String_wat::core::i64>
> further... symbols may not contain commas, however they can
> use underscores..."*

The current substrate uses `,` as a meaningful character in
three positions:
1. **Inside `<...>`** — type-arg separator. Arc 170 slice 1f-W
   shipped the position-aware lexical rule + wire-encoding
   swap (commas ↔ underscores).
2. **In fixed-arity dispatch suffix** — `:wat::core::-,2`
   means "the binary minus dispatch entry." Arc 171 (this arc)
   swaps to `:wat::core::-'2`.
3. **In quasiquote bodies** — `,name` is unquote, `,@list` is
   splice (Scheme/CL tradition). Arc 172 swaps to Clojure
   `~name` / `~@list` and makes `,` pure whitespace per EDN.

Once arc 171 + 172 ship, **`,` carries no meaning in wat
source — pure whitespace per EDN spec.** wat-rs becomes
lexically EDN-compliant.

## Scope

### Substrate edits

**Lexer change** (`src/lexer.rs`):
- Recognize `'` as a token in the position currently held by
  `,` for dispatch-suffix
- During transition (this arc): accept BOTH `'` and `,` in the
  dispatch suffix; emit a deprecation diagnostic for `,` use
- Arc closure: retire `,` acceptance in dispatch-suffix
  position (commas in dispatch fully gone)

**Parser change** (`src/parser.rs`):
- Update fixed-arity dispatch parsing to accept `'` (or both
  during transition)

**Consumer sweep** — one site:
- `wat/core.wat:96`: `(:wat::core::define-dispatch :wat::core::-,2 ...)`
  → `(:wat::core::define-dispatch :wat::core::-'2 ...)`

That's the entire substrate-level consumer sweep per
2026-05-10 grep (`grep -rEn ":[a-zA-Z][a-zA-Z_:\\-]*,[0-9]+\\b"`
across all wat/.rs sources). One site.

### What does NOT change

- Commas inside `<>` keyword bodies — unaffected (slice 1f-W
  already shipped that rule)
- Commas in quasiquote bodies (`,name` / `,@list`) — arc 172's
  domain
- Type declarations / typealiases / struct fields — comma not
  used in source for these
- Symbol names — symbols never contained commas

## Slicing plan

Three small slices. Total predicted: 60-120 min sonnet.

### Slice 1 — Lexer + parser accept `'` (transitional)

- Lexer recognizes `'` after a keyword body as dispatch-suffix
  separator
- Parser accepts `'N` arity suffix on dispatch-form keywords
- Tests verify the new shape parses; existing comma-suffix
  tests still pass

Predicted: 30-60 min sonnet.

### Slice 2 — Single-site consumer sweep

- Edit `wat/core.wat:96` from `,2` to `'2`
- Re-run cargo test; verify the dispatch works under the new
  shape

Predicted: 5-10 min sonnet.

### Slice 3 — Closure (retire `,` in dispatch position)

- Lexer rejects `,` in dispatch-suffix position with a clean
  diagnostic naming the new shape
- Memory + INSCRIPTION + USER-GUIDE row (the deprecation is
  complete; new docs lock in `'N` as canonical)

Predicted: 30-60 min sonnet.

## Dependencies

- Arc 170 slice 1f-W shipped (`4278c4d`) — wire encoding rule
  already locked; this arc doesn't change that.
- Independent of arc 172. Either can ship first.

## Risks

- **Apostrophe collision with quote in other lisps** — Scheme
  uses `'foo` for quote (equivalent to `(quote foo)`). Arc 172
  introduces Clojure `'foo` quote with the same meaning. The
  apostrophe in dispatch suffix is in a DIFFERENT position
  (after a keyword + arity digit) — no ambiguity per lexer's
  position-awareness, but worth verifying.
- **Workspace sweep larger than expected** — grep found one
  site, but the lab repo might have more once we look beyond
  `wat-rs/`. Verify with sonnet during slice 2.

## Ship criteria (whole-arc)

- Substrate lexer/parser accept `'N` dispatch suffix
- `wat/core.wat:96` updated
- Workspace cargo test green (no regressions)
- Final closure slice retires `,` acceptance with clean
  diagnostic if user code still uses `,N`

## Cross-references

- Arc 170 REALIZATIONS pass 14 (the decision that opened this
  arc)
- Arc 170 slice 1f-W (the FIRST thread of EDN-compliance to
  ship — commas inside `<>`)
- Arc 172 (sibling — the THIRD thread; macros)
- Arc 146 — dispatch mechanism this arc fine-tunes

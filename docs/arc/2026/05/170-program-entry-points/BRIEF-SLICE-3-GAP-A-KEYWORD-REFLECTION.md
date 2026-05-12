# Arc 170 slice 3 — Gap A BRIEF (keyword reflection primitives)

**Sonnet.** Close the missing-substrate-capability surfaced by Phase D: macros today can SUBSTITUTE pre-existing keyword AST nodes (via quasiquote/unquote) but cannot CONSTRUCT new keyword ASTs from constituent parts. The lexer tokenizes a keyword as ONE atomic unit; quasiquote substitution cannot reach inside.

This gap blocks not just Phase D's Layer 2 ergonomic shape (it forced full channel-type keywords `:Receiver<i64>` instead of inner element types `:i64`), but EVERY pattern needing keyword construction at macro-expansion time: parametric types, method-name attachment, namespace-path extension, generic mangling. The fix is the reflection primitives that map keywords ↔ strings, plus one sugar form for the most common case.

## What ships (three pieces; locked names per `/gaze` ward)

### 1. `:wat::core::keyword/to-string`  — runtime primitive

```
(:wat::core::keyword/to-string  (k :wat::core::keyword) -> :wat::core::String)
```

Extracts the keyword's text content **without the leading colon**. The colon is sigil; the text carries the name.

Examples:
- `(keyword/to-string :wat::core::i64)` → `"wat::core::i64"`
- `(keyword/to-string :Foo)` → `"Foo"`
- `(keyword/to-string :wat::core::Vector<wat::core::i64>)` → `"wat::core::Vector<wat::core::i64>"`

### 2. `:wat::core::keyword/from-string`  — runtime primitive

```
(:wat::core::keyword/from-string  (s :wat::core::String) -> :wat::core::keyword)
```

Constructs a keyword Value from its text. Inverse of `keyword/to-string` — clean round-trip: `(from-string (to-string k)) = k` for any keyword `k`.

The text MUST NOT have a leading colon (the colon is added by display, not stored). If the input string starts with `:`, that's a malformed-form runtime error (helpful diagnostic; don't silently strip).

### 3. `:wat::core::keyword/of`  — macro special-form (sugar built on the reflection primitives)

Recognized at macro-expansion time in `src/macros.rs::expand_form` alongside `quasiquote` / `quote`. Constructs a parametric type keyword AST from a head keyword + ≥ 1 argument keywords.

```
(:wat::core::keyword/of  head  arg1  [arg2 ...])
```

After argument substitution (so unquote sites `~name` have resolved), the form evaluates to a single Keyword AST whose text is `{head}<{arg1-stripped},{arg2-stripped},...>` (arg-stripped = leading `:` removed from each).

Examples (after argument substitution):
- `(keyword/of :wat::kernel::Receiver :wat::core::i64)` → AST: `:wat::kernel::Receiver<wat::core::i64>`
- `(keyword/of :wat::core::Result :wat::core::i64 :wat::core::String)` → AST: `:wat::core::Result<wat::core::i64,wat::core::String>`
- Nested: `(keyword/of :Vector (keyword/of :Option :i64))` → AST: `:Vector<Option<wat::core::i64>>` (inner form fires first)

**Sugar IMPLEMENTED IN TERMS of reflection primitives** — the macro engine internally uses the same text-extraction + concat + parse path that `keyword/to-string` and `keyword/from-string` use at runtime. This is a deliberate design: the sugar's existence demonstrates the foundation is composable.

## Naming rationale (from `/gaze` ward analysis)

The ward evaluated all candidates and chose:
- `keyword/to-string` — mirrors `Bytes/to-hex` exactly (Type/verb shape, `to-` prefix, `string` names result type)
- `keyword/from-string` — paired inverse; `from-` mirrors `to-`; the pair reads as a unit
- `keyword/of` — `of` reads as prose: `(keyword/of :Receiver :i64)` parses in reader's head as "Keyword of Receiver and i64"

Rejected:
- `parametric-keyword` (Level 2 mumble — names result not action; lexes as a noun phrase, not a constructor)
- `intern` (Level 1 lie — implies global intern table this substrate doesn't have)
- `keyword/text` (Level 2 mumble — sounds like a field access, not a conversion)
- `->string` / `string->` arrow shapes (taste, but breaks established `to-X` / `from-X` convention)

## Future rename note (out of scope here)

Per user direction 2026-05-11: when arc 109 (kill-std FQDN rename) circles back, `:wat::core::keyword` (lowercase, the Value type name) will be renamed to `:wat::core::Keyword` (PascalCase, consistent with `Bytes` / `Process` / `Option` / `Vector` / etc.). The three primitives in this slice will be swept along with that rename; for now they ship under the lowercase namespace.

## Required reading IN ORDER

1. **`src/macros.rs:479`** — `expand_form` handles quasiquote/quote special-cases; `keyword/of` lives at the same level
2. **`src/macros.rs:563`** — `expand_macro_call` does the unquote substitution
3. **`src/runtime.rs:2499`** — `parse_type_keyword` shows canonical format `{head}<{arg1},{arg2},...>` (args WITHOUT leading colon, comma-separated)
4. **`src/runtime.rs`** Value::wat__core__keyword variant — the runtime Value shape for keywords
5. **`src/edn_shim.rs:566`** — keyword EDN handling (for reference)
6. **`wat/test.wat:670+`** — current Layer 2 macro using the workaround (full channel types)
7. **`tests/wat_arc170_program_contracts.rs`** T18/T18b — currently take full channel types; migrate to simpler form
8. **`docs/arc/2026/05/170-program-entry-points/SCORE-SLICE-3-PHASE-D-LAYER2.md`** — Decision 1 documented the gap

## Implementation path (three phases, sequential)

### Phase 1 — Reflection primitives (Rust substrate)

Add two runtime forms to whichever module owns keyword/string primitives (likely `src/runtime.rs` near other string/keyword ops; verify by reading):

- `eval_keyword_to_string(args, env, sym) -> Result<Value, RuntimeError>`
- `eval_keyword_from_string(args, env, sym) -> Result<Value, RuntimeError>`

Register the dispatch arms (likely in `src/runtime.rs::eval_call` or wherever the kernel-verb dispatch table lives). Update type-check schemes in `src/check.rs` for the new keyword verbs.

Unit tests in the relevant Rust test module:
- `(keyword/to-string :foo)` → `"foo"`
- `(keyword/to-string :wat::core::i64)` → `"wat::core::i64"`
- `(keyword/to-string :Vector<i64>)` → `"Vector<i64>"`
- `(keyword/from-string "foo")` → `:foo`
- Round-trip: `(keyword/from-string (keyword/to-string k))` = `k` for several `k`
- Error case: `(keyword/from-string ":foo")` → MalformedForm error mentioning leading colon

### Phase 2 — Macro special form `keyword/of`

Add to `src/macros.rs::expand_form`, AFTER child-recursion has substituted unquotes, BEFORE generic List handling:

```rust
if let Some(WatAST::Keyword(head, _)) = expanded_children.first() {
    if head == ":wat::core::keyword/of" {
        return construct_keyword_of(&expanded_children, list_span);
    }
}
```

`construct_keyword_of` verifies arity ≥ 2 (head + ≥1 args), all children are Keywords, constructs `{head_text}<{arg1_text_stripped},{arg2_text_stripped},...>`, returns a single `WatAST::Keyword(constructed_text, list_span)`.

Errors:
- Arity 0 (just `(keyword/of)` with no args) → `MacroError`
- Non-keyword child → `MacroError` mentioning "keyword/of children must be keyword ASTs"

Add new `MacroError` variant or reuse existing one with prefix.

Macro-expansion-level unit test using a small defmacro that uses `keyword/of` + verify the constructed form's text.

### Phase 3 — Layer 2 migration

Update `wat/test.wat`'s `run-hermetic-with-io` macro to take INNER element types and use `keyword/of` to construct the channel types:

```
(:wat::core::defmacro
  (:wat::test::run-hermetic-with-io
    (input-type :AST<wat::core::nil>)
    (output-type :AST<wat::core::nil>)
    (inputs :AST<wat::core::nil>)
    (body :AST<wat::core::nil>)
    -> :AST<wat::core::nil>)
  `(:wat::test::run-hermetic-with-io-driver
     (:wat::kernel::spawn-process
       (:wat::core::fn
         [rx <- (:wat::core::keyword/of :wat::kernel::Receiver ~input-type)
          tx <- (:wat::core::keyword/of :wat::kernel::Sender ~output-type)]
         -> :wat::core::nil
         ~body))
     ~inputs))
```

Update T18 + T18b in `tests/wat_arc170_program_contracts.rs` to pass inner element types (`:wat::core::i64`) instead of full channel-type keywords. Verify the simplified surface works.

## Ship criteria (10 rows)

| Row | What | Pass criterion |
|-----|------|----------------|
| A | `:wat::core::keyword/to-string` registered + dispatched | grep + unit test |
| B | `:wat::core::keyword/from-string` registered + dispatched | grep + unit test |
| C | `keyword/to-string` returns text WITHOUT leading colon | unit test |
| D | Round-trip `(from-string (to-string k)) = k` works | unit test |
| E | `:wat::core::keyword/of` special-form handled in `expand_form` | grep |
| F | `keyword/of` constructs correct parametric text including comma-separated multi-arg | unit test |
| G | Layer 2 macro in `wat/test.wat` uses `keyword/of` for channel types | grep |
| H | T18 + T18b updated to pass inner element types (e.g., `:wat::core::i64` not `:Receiver<wat::core::i64>`); both still pass | cargo test |
| I | Workspace stays at 0 failed (count stays 2184) | full cargo test |
| J | `cargo check --release` green | clean |

**10 rows.** All must pass.

## Predicted runtime

**60-120 min sonnet.** Substrate primitives (Rust) + macro special-form (Rust) + Layer 2 migration (wat) + test updates (wat in Rust strings).

**Hard cap:** 240 min.

## Constraints (hard)

- DO NOT commit. Orchestrator atomic-commits after scoring verification.
- DO NOT rename `:wat::core::keyword` to `:wat::core::Keyword` — that's deferred to a future arc 109 follow-up; primitives ship under the current lowercase.
- DO NOT modify Layer 1 macro / driver (Layer 1 is unchanged).
- DO NOT touch `deftest` / `deftest-hermetic`.
- DO NOT touch BareLegacy* walker / spawn.rs / Process struct fields.
- DO NOT use deferral language in SCORE — per FM 11.
- If a Phase 1-2-3 ordering surfaces an architectural blocker (e.g., quasiquote evaluation order means `keyword/of` can't see substituted unquotes), STOP and report; do not workaround.
- Workspace must stay at 0 failed throughout.

## Honest delta categories (anticipated)

1. **Where the runtime primitives live** — which module owns them; rationale
2. **Macro form expansion order** — does `keyword/of` fire before or after enclosing quasiquote splices; how composition with `~unquote` resolves
3. **Phase 3 macro shape** — does `~input-type` substitute correctly INTO `(keyword/of :Receiver ~input-type)`; verify
4. **Anything unexpected** — surfaced during authorship

## Cross-references

- Phase D SCORE: [`SCORE-SLICE-3-PHASE-D-LAYER2.md`](./SCORE-SLICE-3-PHASE-D-LAYER2.md) — Decision 1 documents the gap
- Phase C SCORE: [`SCORE-SLICE-3-PHASE-C-LAYER1.md`](./SCORE-SLICE-3-PHASE-C-LAYER1.md) — Path A pattern
- `/gaze` ward (naming rationale): `holon-lab-trading/.claude/skills/gaze/SKILL.md`
- Future rename: `:wat::core::keyword` → `:wat::core::Keyword` queued for arc 109 follow-up; NOT this slice
- Gap B (next, after this ships): `Sender/close` for explicit EOF signaling on channel write side

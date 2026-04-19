# Colon-Quote Sweep — Backlog

**Created**: 2026-04-19
**Priority**: HIGH — blocks further slice work
**Scope**: wat + holon-lab-trading wat/ + 058 proposal corpus

## The insight

`:` is wat's **symbol-literal reader macro**. One leading `:` quotes the body as a symbol; the body is a **literal Rust path**. This is what `:` has always meant — we just didn't realize it until we tried to write `:crossbeam_channel::Sender<T>` and found our lexer rejecting internal colons.

Implication: **`::` is the canonical namespace separator.** `/` was a wat-local accommodation, not a design choice.

Implication: every wat keyword-path should be a literal Rust path (no translation). We do this because the language is strongly typed and the types should read as honestly as possible.

## What's dishonest today

| Current | Rust truth | Reason for change |
|---|---|---|
| `:wat::core::load!` | `:wat::core::load!` | `/` was never the Rust separator |
| `:wat::core::+` | `:wat::core::+` | same |
| `:wat::core::/` | `:wat::core::/` | reads unambiguous — `::` separator, `/` is the name |
| `:my/vocab/foo` | `:my::vocab::foo` | user paths too |
| `:Vec<T>` | `:Vec<T>` | Rust collection is Vec |
| `:wat::core::vec` | `:wat::core::vec` | Rust constructor is `vec!` / `Vec::new()` |
| `:Pair<T,U>` | `:(T,U)` | Rust has no `Pair`; it has tuples |
| `:Tuple<T,U,V>` | `:(T,U,V)` | same |
| `:Union<T,U,V>` | named enum required | Rust has no anonymous union; force named enum declaration |
| `:QueueSender<T>` | `:crossbeam_channel::Sender<T>` | use the crate path |
| `:QueueReceiver<T>` | `:crossbeam_channel::Receiver<T>` | same |

## What stays honest (no change)

| Type | Rust |
|---|---|
| `:i64`, `:i32`, `:usize`, `:f64`, `:bool`, `:String`, `:()` | 1:1 with Rust |
| `:Option<T>`, `:Result<T,E>` | 1:1 |
| `:HashMap<K,V>`, `:HashSet<T>` | 1:1 |
| `:fn(T,U)->R` | Rust `fn(T,U)->R` (function-pointer syntax) |
| `:holon::HolonAST`, `:wat::core::keyword`, `:AST<T>` | wat-originated (no Rust parallel; owned honestly) |

## Union — why named enum is the honest replacement

**Current (with `:Union<T,U,V>`):**
```scheme
(:wat::core::define (:my::handle (x :Union<i64,String,bool>) -> :holon::HolonAST)
  ...)

(my::handle 42)      ; implicit variant — which one is 42?
(my::handle "hi")
(my::handle true)
```

**Honest (named enum):**
```scheme
;; Declare the coproduct explicitly.
(:wat::core::enum :my::IntStringBool
  (IsInt    (n :i64))
  (IsString (s :String))
  (IsBool   (b :bool)))

;; Use it.
(:wat::core::define (:my::handle (x :my::IntStringBool) -> :holon::HolonAST)
  ...)

;; Callers tag the variant.
(my::handle (:my::IntStringBool::IsInt    42))
(my::handle (:my::IntStringBool::IsString "hi"))
(my::handle (:my::IntStringBool::IsBool   true))
```

The named-enum approach is objectively better: every coproduct has a discriminator, dispatch is explicit, and the Rust emit path is trivial (`enum IntStringBool { IsInt(i64), IsString(String), IsBool(bool) }`).

## Backlog (sequenced so tests stay green)

### Track A — Foundation (blocks everything else)

**A1: Lexer — remove `InternalColon` rule**
- Delete `LexError::InternalColon`.
- Lexer accepts internal `:` and `::` in keyword bodies.
- Module doc rewritten with the colon-quote framing.
- Tests: `:wat::core::load!`, `:crossbeam_channel::Sender<T>`, `:wat::core::/`, `:Vec<T>`, `:(T,U)`.

**A2: Parser — tuple-literal type syntax `:(T,U,...)`**
- Fourth shape in `parse_type_expr`: a keyword starting `:(` opens a tuple.
- `:()` stays (unit = 0-tuple).
- Grammar: the `(` must immediately follow the `:` (no whitespace).
- Tests: `:(i64,String)`, `:(Holon,holon::HolonAST,Holon)`, `:()`.

### Track B — Reserved-prefix migration

**B1+B2: Flip `RESERVED_PREFIXES` + sweep every Rust source string**
- `RESERVED_PREFIXES` becomes `[":wat::core::", ":wat::kernel::", ":wat::algebra::", ":wat::std::", ":wat::config::", ":wat::load::", ":wat::verify::"]`.
- Every Rust match arm, every symbol-table key, every scheme registration, every check-pass built-in key, every resolve-pass head, every lower-pass head, every runtime dispatch.
- Every test's wat-source literal.
- Division: `:wat::core::/` → `:wat::core::/`.
- Atomic commit — can't be split without breaking tests.

### Track C — Type system honesty

**C1: `:Vec<T>` → `:Vec<T>` and `:wat::core::vec` → `:wat::core::vec`**
- Rename in check.rs schemes (Bundle's input type, list constructor return type).
- Rename constructor path; `is_special_form` updates.
- Every test referring to `:Vec<...>` or `(:wat::core::vec ...)` updates.

**C2: Drop `:Pair<T,U>` / `:Tuple<T,U,V>` from docs**
- Requires A2.
- Docs-only — wat code doesn't emit these types directly.

**C3: Drop `:Union<T,U,V>`**
- `parse_type_expr` refuses `:Union<...>` at the same layer as `:Any` with a dedicated `TypeError::UnionRetired` variant, message pointing at "declare a named enum."
- `058-030` and FOUNDATION retire the form.

**C4: Channel types — `:crossbeam_channel::Sender<T>` / `:crossbeam_channel::Receiver<T>`**
- Replace `:QueueSender<T>` / `:QueueReceiver<T>` in proposal docs.
- Decision: `crossbeam_channel` (the actual dep in trading-lab) over `std::sync::mpsc`.
- Docs-only in wat — kernel isn't implemented yet.

### Track D — Downstream sweep

**D1: holon-lab-trading wat files (every `wat/*.wat`)**
- Every `:wat::...` → `:wat::...`.
- Every user path migrates to `::`.
- Blocks running under wat-vm until migrated.
- Ward pass after.

**D2: 058 proposal documents**
- `FOUNDATION.md` — every keyword-path example.
- `058-030-types/PROPOSAL.md`, `058-029-lambda/PROPOSAL.md`, `058-028-define/PROPOSAL.md`, `058-001-atom-typed-literals/PROPOSAL.md`.
- Every sub-proposal with wat code (30+ files under `058-ast-algebra-surface/`).
- `OPEN-QUESTIONS.md`, `INDEX.md`.

**D3: FOUNDATION-CHANGELOG**
Four entries (2026-04-19):
- Colon-quote model — `:` is the symbol-literal reader macro; `::` is the canonical separator.
- `:Vec<T>` → `:Vec<T>` and `:wat::core::vec`.
- `:Pair<T,U>` / `:Tuple<T,U,V>` retired for `:(T,U)` / `:(T,U,V)`.
- `:Union<T,U,V>` retired — heterogeneous types require named enums.
- Channel types use `crossbeam_channel` paths.

### Track E — Verification

**E1: Ward pass across the whole repo**
- `/ignorant` + `/scry` + `/gaze` over updated corpus.
- Find any `/`-separator path that slipped through.
- Find any leftover `:Vec<...>`, `:Pair<...>`, `:Tuple<...>`, `:Union<...>`, `:QueueSender<...>`, `:QueueReceiver<...>`.

**E2: End-to-end smoke**
- wat: `cargo test` + `cargo test --release` clean.
- holon-lab-trading: build + smoke (once wat files migrate).

## Commit plan

Each commit must leave tests green.

1. A1 — lexer allows internal `::`
2. A2 — tuple-literal parser
3. B1+B2 — atomic reserved-prefix flip + source sweep
4. C1 — `:List` → `:Vec`, `:wat::core::vec` → `:wat::core::vec`
5. C3 — drop `:Union` from parser
6. D1 — holon-lab-trading wat files
7. D2 + D3 — proposal docs + changelog entries
8. E1 — ward sweep; fix anything caught

Parallel: C2 and C4 are docs-only and land with D2.

## What this is not

This is **not a proposal** for review. The insight — `:` is the symbol-quote, `::` is the separator — is a correction of a mistake, not a design choice up for debate. The backlog executes the correction.

The 058 proposals were WRITTEN with the `/` convention. Updating them documents the correction; it doesn't re-open the decision.

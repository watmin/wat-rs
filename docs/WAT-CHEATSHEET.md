# Wat syntax cheatsheet

Single-page reference for writing wat. The substrate teaches you
when you break a rule (every constraint here surfaces as a parse
or type-check error with a concrete fix path); this doc is the
table you check BEFORE writing so the iteration loop is shorter.

For the deep tutorial / mental model see `USER-GUIDE.md`. For
naming + namespacing rules see `CONVENTIONS.md`. For the
concurrency architecture see `ZERO-MUTEX.md`. This cheatsheet is
*how to spell things*; those docs are *what the things mean*.

---

## 1. Colon rule

ONE colon per keyword-path token, always at the start. NEVER inside
`<>`, `(...)`, `:fn(...)`, or `:[...]`. Type expressions inside
those brackets are bare Rust symbols.

| Illegal | Canonical | Why |
|---|---|---|
| `:Vec<:String>` | `:Vec<String>` | inner colon — arc 115 |
| `:Result<:Option<i64>,:wat::kernel::ThreadDiedError>` | `:Result<Option<i64>,wat::kernel::ThreadDiedError>` | same |
| `:fn(:i64)->:bool` | `:fn(i64)->bool` | same |
| `:Vec<:wat::core::String>` | `:Vec<wat::core::String>` | same |

Arc 115's compile error names the rule and shows the canonical
form. See `arc/2026/04/115-no-inner-colon-in-parametric-args/`.

## 2. Whitespace rule

NO whitespace inside `<...>`, `:(...)`, `:fn(...)`, or `:[...]`.

| Illegal | Canonical |
|---|---|
| `:Vec<wat::core::i64, wat::core::String>` | `:Vec<wat::core::i64,wat::core::String>` |
| `:Result<(), Vec<wat::kernel::ThreadDiedError>>` | `:Result<(),Vec<wat::kernel::ThreadDiedError>>` |
| `:(A, B, C)` | `:(A,B,C)` |
| `:fn(A, B) -> C` | `:fn(A,B)->C` |

The lexer rejects whitespace inside an unclosed bracket.

## 3. FQDN namespace rule

Substrate-provided types use their full path. No bare aliases
like `:Sender<T>` or `:Receiver<T>` — those are not registered.

| Illegal / unregistered | Canonical |
|---|---|
| `:Sender<T>` | `:rust::crossbeam_channel::Sender<T>` |
| `:Receiver<T>` | `:rust::crossbeam_channel::Receiver<T>` |
| `:i64` | `:wat::core::i64` (in user code post-arc-109/1c) |
| `:String` | `:wat::core::String` (same) |

Type aliases CAN be defined in user code (`:wat::core::typealias`)
but are not auto-registered for substrate types. See arc 109's
J-PIPELINE.md for the FQDN sweep.

## 4. Comm-call position rule

`:wat::kernel::send` / `recv` / `try-recv` / `select` /
`process-send` / `process-recv` MUST appear ONLY as:

- the scrutinee of `:wat::core::match`, OR
- the value-position of `:wat::core::result::expect`, OR
- the value-position of `:wat::core::option::expect`.

Bare let* RHS, function-call argument positions, etc. are
illegal. Arc 110 enforces this — silent disconnect must be
handled at every comm site.

```wat
;; Illegal
((received :Result<...>) (:wat::kernel::recv rx))

;; Canonical
(:wat::core::match (:wat::kernel::recv rx)
  -> :T
  ((Ok (Some v)) ...)
  ((Ok :None)    ...)
  ((Err died)    ...))
```

## 5. Control-form shapes

| Form | Required shape |
|---|---|
| `:wat::core::if` | `(if cond -> :T then else)` — arc 108 made `-> :T` mandatory |
| `:wat::core::cond` | `(cond -> :T (test-1 result-1) (test-2 result-2) ... (else default))` |
| `:wat::core::let*` | `(let* ((name :T expr) ...) body)` |
| `:wat::core::match` | `(match scrutinee -> :T (pattern body) ...)` |
| `:wat::core::define` | `(define (:user::name (arg :T) -> :Ret) body)` |
| `:wat::core::lambda` | `(lambda ((arg :T) -> :Ret) body)` |

The `-> :T` is the result-type annotation; required on `if`,
`cond`, `match`, `define`, and `lambda`.

## 6. Special-form arg shapes

Forms that take ASTs (not strings):

| Form | Takes |
|---|---|
| `:wat::kernel::raise!` | `data: HolonAST`. Wrap a string with `(:wat::holon::leaf "msg")`. |
| `:wat::kernel::assertion-failed!` | `(message :String, actual :Option<String>, expected :Option<String>)` |
| `:wat::core::eval-ast!` | `:wat::WatAST` (the AST datatype itself) |

Forms that take string literals:

- `assertion-failed!`'s message field
- `:wat::kernel::run-sandboxed`'s src
- error-message slots on `result::expect` / `option::expect`

## 7. No-`:Any`, no-new-types

`:Any` is banned in wat source. Heterogeneous storage uses
`std::any::Any` on the Rust side; wat code uses concrete types
or generics.

Wat does NOT mint its own type system. `Atom<T>` uses real Rust
types — `Atom<wat::core::String>`, `Atom<wat::holon::HolonAST>`,
etc. No `AtomLiteral` enum or `AtomValue` trait. Rust types ARE
wat types.

## 8. Common verb signatures

| Verb | Returns |
|---|---|
| `:wat::kernel::send sender value` | `:Result<(),:Vec<wat::kernel::ThreadDiedError>>` |
| `:wat::kernel::recv receiver` | `:Result<Option<T>,:Vec<wat::kernel::ThreadDiedError>>` |
| `:wat::kernel::try-recv receiver` | `:Result<Option<T>,:Vec<wat::kernel::ThreadDiedError>>` |
| `:wat::kernel::select [(rx-1 ...) (rx-2 ...)]` | `:Result<Chosen<T>,:Vec<wat::kernel::ThreadDiedError>>` |
| `:wat::kernel::spawn-thread body` | `:wat::kernel::Thread<I,O>` (arc 114) |
| `:wat::kernel::Thread/join-result thr` | `:Result<:(),:Vec<wat::kernel::ThreadDiedError>>` |
| `:wat::kernel::spawn-program src scope` | `:Result<wat::kernel::Process<I,O>,wat::kernel::StartupError>` |
| `:wat::kernel::Process/join-result proc` | `:Result<:(),:Vec<wat::kernel::ProcessDiedError>>` |

Arc 113 widened every Err arm to `:Vec<*DiedError>` (chain).
Arc 114 retired `:wat::kernel::spawn` / `join` / `join-result`
in favor of `spawn-thread` + `Thread/join-result`.

## 9. Test verbs

Tests use `:wat::test::*`, NOT `:user::*`:

| Verb | Path |
|---|---|
| `assert-eq` | `:wat::test::assert-eq<T>` |
| `assert-substring` | `:wat::test::assert-substring` |
| `assert-coincident?` | `:wat::test::assert-coincident?` |
| `deftest` | `:wat::test::deftest` |

See USER-GUIDE.md § 13 "Testing".

## 10. Scope-deadlock rule

Outer scope holds the Thread; inner scope owns every Sender
clone. The compiler refuses programs where a `QueuePair` /
`QueueSender` lives at sibling scope to a Thread whose
`Thread/join-result` runs in the same `let*`.

```wat
;; Illegal — pair sibling to thr; pair's Sender outlives thr;
;; the worker's recv never sees EOF.
(:wat::core::let*
  (((pair :wat::kernel::QueuePair<i64>) (:wat::kernel::make-bounded-queue :wat::core::i64 1))
   ((thr  :wat::kernel::Thread<(),i64>) (:wat::kernel::spawn-thread ...))
   ...)
  (:wat::kernel::Thread/join-result thr))

;; Canonical — outer holds thr; inner owns pair + Sender;
;; inner returns thr; pair drops at inner-scope exit.
(:wat::core::let*
  (((thr :wat::kernel::Thread<(),i64>)
    (:wat::core::let*
      (((pair :wat::kernel::QueuePair<i64>) (:wat::kernel::make-bounded-queue :wat::core::i64 1))
       ((h    :wat::kernel::Thread<(),i64>) (:wat::kernel::spawn-thread ...))
       ...)
      h)))
  (:wat::kernel::Thread/join-result thr))
```

Same rule applies to `Process/join-result`. Arc 117 enforces it
at type-check time. See `SERVICE-PROGRAMS.md § "The lockstep"`
for the why.

## 11. Discovery loop

When you trip a rule:

1. Read the substrate's error message — it includes the rule + a
   concrete fix path (the substrate-as-teacher discipline; see
   `SUBSTRATE-AS-TEACHER.md`).
2. Re-check this cheatsheet for the rule's canonical form.
3. Find the arc that introduced the rule (the error message names
   it; e.g., "arc 115") and read its INSCRIPTION for the why.

The substrate is the most authoritative reference for its own
behavior — this cheatsheet aggregates the rules at a snapshot in
time. When this disagrees with the substrate, the substrate
wins. File a doc bug.

---

## Sources of truth

- **Active rules** — every entry above traces to an arc inscription
  in `docs/arc/2026/04/`. The arc is the authoritative why; this
  doc is the convenient how.
- **Living changelog** — `holon-lab-trading/docs/proposals/2026/04/058-ast-algebra-surface/FOUNDATION-CHANGELOG.md`
  records every shipped change. When a rule changes, the changelog
  records it; this cheatsheet updates from there.
- **The substrate's own error messages** — every rule above is
  enforced at parse / type-check time with a self-describing
  diagnostic. If the diagnostic is unclear, that's a substrate bug
  to file, not a doc-only fix.

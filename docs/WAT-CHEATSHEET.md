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
| `:wat::core::unit` | `:wat::core::nil` (arc 153 — same type, new name) |

Type aliases CAN be defined in user code (`:wat::core::typealias`)
but are not auto-registered for substrate types. See arc 109's
J-PIPELINE.md for the FQDN sweep.

### `:wat::core::nil` — the singleton (arc 153)

`:wat::core::nil` is wat's name for the unit type — the type
with one inhabitant, the role Rust spells `()`. Same name in
both positions:

- **Type position.** `(:my::probe -> :wat::core::nil)` declares
  "this function returns nothing meaningful." The empty-tuple
  spelling `:()` is bare and retires per arc 109 slice 1d; the
  legacy FQDN `:wat::core::unit` retired arc 153.
- **Value position.** `:wat::core::nil` evaluates to the nil
  singleton. The empty-list literal `()` continues to evaluate
  to the same singleton (transitional spelling kept for
  cross-form ergonomics).

The triplet `nil` / `Some(t)` / `None` reads cleanly and stays
orthogonal — `:wat::core::nil` is the unit type (singleton),
`:wat::core::None` is `Option<T>`'s absence variant,
`:wat::core::Some(t)` is the presence variant. The type system
enforces the split. No "null pointer exception" semantics; no
sentinel-value lies.

### `:wat::core::do` — sequential evaluation (arc 136)

`(:wat::core::do form_1 form_2 ... form_N)` evaluates each form
left-to-right; non-final results are discarded; the FINAL form's
value is returned and its inferred type IS the do form's type.
Clojure-faithful — non-finals' types are unconstrained.

```wat
;; The print-then-return idiom, daily verb of any Lisp:
(:wat::core::do
  (:wat::console::log "computing...")
  (:wat::core::i64::+ 1 1))                ;; → :i64

;; Replaces the let*-with-((_ :wat::core::unit) ...) crutch:
(:wat::core::do
  (:wat::test::assert-eq v1 e1)
  (:wat::test::assert-eq v2 e2)
  (:wat::test::assert-eq v3 e3))           ;; → :wat::core::nil
```

Empty `(:wat::core::do)` is a parse error. Single-form
`(:wat::core::do x) ≡ x`. Substrate infers from the final form;
recipient unification verifies.

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
| `:wat::core::let` | `(let ((name :T expr) ...) body)` — arc 154; sequential semantics (Clojure-faithful single-letform vocabulary; `let*` retired) |
| `:wat::core::do` | `(do form_1 form_2 ... form_N)` — arc 136; non-finals' types unconstrained, final form's type IS the do's type |
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
| `:wat::kernel::Thread/join-result thr` | `:Result<wat::core::nil,:Vec<wat::kernel::ThreadDiedError>>` |
| `:wat::kernel::spawn-program src scope` | `:Result<wat::kernel::Process<I,O>,wat::kernel::StartupError>` |
| `:wat::kernel::Process/join-result proc` | `:Result<wat::core::nil,:Vec<wat::kernel::ProcessDiedError>>` |

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
clone. The compiler refuses programs where a `Channel` /
`Sender` lives at sibling scope to a Thread whose
`Thread/join-result` runs in the same `let*`.

```wat
;; Illegal — pair sibling to thr; pair's Sender outlives thr;
;; the worker's recv never sees EOF.
(:wat::core::let*
  (((pair :wat::kernel::Channel<i64>) (:wat::kernel::make-bounded-channel :wat::core::i64 1))
   ((thr  :wat::kernel::Thread<(),i64>) (:wat::kernel::spawn-thread ...))
   ...)
  (:wat::kernel::Thread/join-result thr))

;; Canonical — outer holds thr; inner owns pair + Sender;
;; inner returns thr; pair drops at inner-scope exit.
(:wat::core::let*
  (((thr :wat::kernel::Thread<(),i64>)
    (:wat::core::let*
      (((pair :wat::kernel::Channel<i64>) (:wat::kernel::make-bounded-channel :wat::core::i64 1))
       ((h    :wat::kernel::Thread<(),i64>) (:wat::kernel::spawn-thread ...))
       ...)
      h)))
  (:wat::kernel::Thread/join-result thr))
```

Same rule applies to `Process/join-result`. Arc 117 enforces it
at type-check time. Arc 131 extended it to `HandlePool<T>` —
when T (after alias resolution) contains a Sender, a HandlePool
sibling to a Thread with `Thread/join-result` fires the same
diagnostic with `(a HandlePool)` as the offending kind. Arc 133
extended visibility to tuple-destructure bindings
`((pool driver) ...)` so the check sees them uniformly with
typed-name shapes. See `SERVICE-PROGRAMS.md § "The lockstep"`
for the why.

Arc 134 added two structural narrowings to reduce false positives
on canonical Thread<I,O> usage:

- **Origin-trace exemption.** A Sender whose binding RHS is
  `(:wat::kernel::Thread/input <_>)` or `Process/input` extracts
  the parent-side end of an internal pipe owned by the Thread
  struct. The pair-Receiver is the spawned function's `in`
  parameter — lifetime-coupled to the Thread. The rule does NOT
  fire on this shape, even when sibling to `Thread/join-result`.

- **Body-form exemption.** When the Thread's binding RHS is a
  spawn call whose function argument is an inline lambda whose
  body contains no `(:wat::kernel::recv ...)` / `try-recv` /
  `select` call, no recv-loop can exist; no Sender lifetime can
  deadlock the thread. The rule does NOT fire for any sibling
  Sender in that case.

Both narrowings are heuristic — a body that calls a helper
function which recvs, or a lambda body with an unbounded recv-
loop on its input pipe, can still deadlock at runtime; arc
134's narrowings prefer precision over conservative-fire and
accept the runtime hang as the cost. See arc 134's INSCRIPTION
for the full failure-engineering record.

## 11. Channel-pair-deadlock rule

A function call MUST NOT receive both halves of one
`make-bounded-channel` pair as arguments. Holding both ends
in one role deadlocks any recv — the caller's writer keeps
the channel alive even when the receiving thread dies.

```wat
;; Illegal — caller binds both `tx` and `rx` from one pair;
;; the helper-verb call passes both. Recv inside the helper
;; never sees EOF if the worker dies; caller's tx clone
;; keeps the channel open.
(:wat::core::let*
  (((pair :wat::kernel::Channel<wat::core::nil>)
    (:wat::kernel::make-bounded-channel :wat::core::nil 1))
   ((tx :wat::kernel::Sender<wat::core::nil>)   (:wat::core::first  pair))
   ((rx :wat::kernel::Receiver<wat::core::nil>) (:wat::core::second pair))
   ...
   ((_ :wat::core::nil) (:my::helper-verb tx rx ...)))
  ...)

;; Canonical — pair-by-index via HandlePool. Each producer
;; pops one Handle holding ONE end of EACH of two distinct
;; channels. The driver gets the corresponding (Rx, AckTx).
;; Distinct pair-anchors → distinct channels → no deadlock.
(:wat::core::let*
  (((handle :svc::Handle)                (:wat::kernel::HandlePool::pop pool))
   ((req-tx :svc::ReqTx<...>)            (:wat::core::first  handle))
   ((ack-rx :svc::AckRx<wat::core::nil>) (:wat::core::second handle))
   ...
   ((_ :wat::core::nil) (:my::helper-verb req-tx ack-rx ...)))
  ...)
```

Arc 126 enforces this at type-check time. The diagnostic names
the pair-anchor binding and points at `ZERO-MUTEX.md § "Routing
acks"` for the canonical fix patterns. Same trace machinery as
arc 117; different rule arm.

## 12. Discovery loop

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

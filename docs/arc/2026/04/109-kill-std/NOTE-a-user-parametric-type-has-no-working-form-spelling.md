# ⛔ NOTE (arc 109) — a USER parametric type has no working FORM spelling. It BLOCKS β-ii-c.

**Filed 2026-08-21 while grounding β-ii-c. MEASURED at `8e6e83618`.**

β-ii-c's entire job is to make `defservice` emit its generated type references as FORMS —
`(:lru-svc::State :- [K V])` instead of the keyword `:lru-svc::State<K,V>`. **That form does not
work for a user-declared type**, so the stone cannot land as drawn.

## The measurement

```
(:wat::core::Vector :- [:wat::core::i64])                     core parametric      → clean
(:wat::kernel::Peer [:wat::core::i64 :wat::core::i64])        stdlib parametric    → clean
:user::Box<wat::core::i64>            (Box declared in-file)  legacy keyword       → clean
(:user::Box :- [:wat::core::i64])     (Box declared in-file)  FORM, marked         → ERROR
(:user::Box [:wat::core::i64])        (Box declared in-file)  FORM, unmarked       → ERROR
```

The error is the same in both form spellings, and it is not about `:-`:

> *"invalid type keyword: malformed type expression `[…]`: function-type bracket needs a `:->`
> arrow: `[arg… :-> ret]`"*

— i.e. the bracket is being parsed as a STANDALONE function type, so the list never reaches
`parse_type_form`'s bracketed-args arm.

## ★ The trigger is DECLARING the type — which is the opposite of the obvious guess

```
(:user::Nope [:wat::core::i64])    type NOT declared anywhere   → clean
(:user::Box  [:wat::core::i64])    type declared in-file        → ERROR
(:user::Plain [])                  declared, ZERO params        → ERROR
```

An **undeclared** type passes (the filed "type annotation names are unchecked in annotation
position" gap). A **declared** one fails. Prefix is irrelevant — `:user::` and `:my::` behave
identically. Arity is irrelevant — the zero-param case fails too.

## ★ THE MECHANISM — MEASURED, not guessed

**A `defrecord` registers `:user::Box` as a CONSTRUCTOR function. Once a function of that name
exists, the checker resolves the list `(:user::Box …)` as a CALL rather than a type — and the
leftover bracket is then parsed as a standalone type, which is why the message is about
function-type brackets.**

Four independent measurements, and the last is the one that makes it certain:

| probe | result | what it rules out |
|---|---|---|
| the AST of `(:user::Box [i64])` vs `(:wat::core::Vector [i64])` | **IDENTICAL** — `list(keyword, vector)` in both | a syntax/shape difference |
| `typealias :user::Alias<T>` — mints NO ctor | **works** | "user namespaces are broken" |
| `defenum :user::E<T>` — no ctor under its OWN name | **works** | "declared types are broken" |
| `defrecord`, referenced BEFORE its own declaration | **works** | everything except ctor registration |

The order test is decisive: byte-identical source, and the only variable is whether the constructor
exists yet when the annotation is read. **A type annotation's meaning currently depends on
declaration order.**

Core containers escape it because the checker special-cases them. `typealias` and `defenum` escape
it because neither mints a function under its own name.

★ This is stone 251.8's subject — *"one node, two unrelated jobs — a value and a reference — with
the node itself unable to say which"* — surfacing in the TYPE-FORM surface rather than at call
heads. There the tiebreaker was context; here it is "does a function of this name happen to be
registered yet."

## Why this blocks β-ii-c and not the stones before it

β-ii-a′ and β-ii-b never emitted a form — a′ made the binder the source of truth while the emitted
spelling stayed a `<K,V>` keyword, and b deleted a suffix from function names. β-ii-c is the first
stone that must emit `(Head :- [args])` for a **user** type, and `lru-svc::State`, `::Record`,
`::Handle`, `::Admin`, `::Op` are all user types with constructors.

## What β-ii-c would have been

Measured while drawing it, and worth keeping because it is still the shape once this is unblocked:

- **35 splice sites across 6 names** — `record-ty` 13 · `state-ty` 8 · `admin-ty` 4 · `status-ty` 4 ·
  `handle-name` 4 · `service-op-decl-kw` 2.
- **6 of the 35 are DECLARATION slots**, not references: `defrecord ~record-ty` (`:642`),
  `defstruct ~state-ty` (`:666`), `defenum ~admin-ty` (`:992`), `defenum ~status-ty` (`:1002`),
  `defenum ~service-op-decl-kw` (`:1164`), `defstruct ~handle-name` (`:2660`).
  ★ **A declaration takes `Head :- [K V]` as SIBLINGS; a reference takes `(Head :- [K V])` as a
  FORM.** Emitting a form in a name slot is precisely the codemod's binder bug — **the third
  appearance of the same trap this session.**
- The remaining 29 are type references and would become forms.

## Suggested sequencing

β-ii-c is blocked. β-ii-d (the substring transport-param test) is NOT — it reads `fqdn-tp-syms`,
which exists, and emits nothing new. It can proceed while this is decided.

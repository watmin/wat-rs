# NOTE — a `where` body is never macro-expanded (2026-08-05)

Grounded twice, by direct read, while landing `BRIEF-rete-cond-is-its-own-macro.md`:

- `wat/rete.wat:2315` — `defrule` quotes the conditions verbatim:
  `(:wat::core::quote ~when-vec)`. The `:when` vector never passes through the macro expander;
  it is captured as raw AST inside a `quote` form.
- `src/rete/matcher.rs:1237` — `eval_test_core` evaluates that raw AST by calling
  `runtime::eval_inner` directly on it. There is no expansion pass on this path at all.
- `src/macros/expand.rs:441` — the expander does **not** descend into `:wat::core::quote`. So
  even if a `where` form happened to be visited during a later expansion pass, the quote wrapper
  would shield it.

**Consequence:** a `cond` written literally inside a `where` clause — core-spelled
(`:wat::core::cond`) **or** rete-spelled (`:wat::rete::core::cond`) — still raises
`UnknownFunction` at fire time, not compile time, because neither spelling has ever been
expanded away by the time `eval_test_core` runs it. `dispatch_rete_op`'s runtime re-invoke has
an arm for genuine runtime special forms and `Alias`/`Redispatch` rows, but `cond` has none — it
only exists as a macro template.

Proven both ways on the current tree:

- `wat-scripts/scratch-pad/probe-cond-in-where-baseline.wat` — a bare
  `(:wat::rete::where (:wat::core::cond (?a true) (:else false)))` raises
  `#wat.runtime/UnknownFunction {:message "unknown function: :wat::core::cond" ...}`. This
  predates `BRIEF-rete-cond-is-its-own-macro.md` and is unaffected by it — the core spelling was
  never expandable inside a `where` and still isn't.
- `wat-scripts/scratch-pad/probe-cond-rete-where.wat` — the same shape with the rete spelling,
  `(:wat::rete::where (:wat::rete::core::cond (?a true) (:else false)))`, raises the equivalent
  `UnknownFunction` on `:wat::rete::core::cond`. Minting `cond` as its own rete-spelled
  `defmacro` (`wat/rete.wat`) does not change this: the macro is real and expands correctly
  everywhere macro expansion actually runs, but a `where` body never reaches the expander to
  begin with.

## Why this is out of scope here (STOP-2)

Closing this gap means making `where`/`:test` bodies pass through macro expansion before they
reach `eval_test_core` — a change to `defrule`'s quoting strategy and/or the matcher's
evaluation path, not to any macro's template. That is a separate, larger mechanism change, and
it is the builder's ruling to make, not this strike's.

`BRIEF-rete-cond-is-its-own-macro.md` is the correct and necessary prerequisite for whatever
later change closes this gap: once `where` bodies *are* expanded, a `cond` inside one needs to
expand to rete-spelled `if`, not core-spelled `if`, to stay law-A-clean. Landing that macro now
means the day this gap closes, `cond` already emits the right spelling — no follow-up rewrite of
the macro itself will be needed.

The two probes above are **RED by design** and measure exactly this gap. Do not "fix" them by
routing `where` through the expander as a side effect of this brief — that STOP is deliberate.

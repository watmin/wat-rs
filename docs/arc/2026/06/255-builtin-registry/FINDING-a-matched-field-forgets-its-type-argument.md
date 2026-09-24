# FINDING — a field bound by matching a parametric enum forgets its type argument (D2)

**Measured 2026-09-24** by the message-alias probe executor, and re-run by the orchestrator, against a
snapshot binary built from `main` @ `58d7e0ad3`. Probe files are in the session scratchpad under
`alias-probe/`. The reproducer is reproduced verbatim below so it outlives the scratchpad.

## The lie — `--check` rc=0, runtime rc=1

```wat
(:wat::core::defenum :probe::E :- [X] :wat::enum::Pure
  :A [v <- :X]
  :B [])
(:wat::core::defn :probe::k :- [T] [e <- (:probe::E :- [T])] -> :wat::core::i64
  (:wat::core::match e
    [:probe::E.A {:v v} v]
    [_ (:wat::kernel::assertion-failed! :message "x")]))
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:wat::i64::+ 1 (:probe::k (:probe::E.A {:v "hello"})))))
```

- `--check`: **rc=0**. A `T`-typed field is returned as `i64` inside a generic defn.
- run: **rc=1**, `":wat::i64::+: expected i64, got wat::core::String \"hello\""`.
- Control (the same shape at concrete `String`): `--check` **rc=1**, *"body produces :wat::core::String;
  signature declares :wat::core::i64"*. The check exists. It is lost only in the generic context.
- Control (plain type variables stay rigid): `:K` returned as `:V` → rc=1.

## Mechanism (as measured, not yet located in code)

Inside a generic defn, a field bound by a `match` arm on a parametric enum is typed with a hole
(`(GetResponse :- [_])`). That hole then unifies with any argument list of the right-ish shape. `[A]`,
`[B]`, `[A B]` and `[i64]` all pass; `[A B A]` is refused. The constructor path does not share the
hole: `[K V]` is refused there. **The room is unlocated.** Suspect the match-pattern binding of fields
against a parametric scrutinee in `src/check.rs`.

## Why it matters here

- **The alias probe could not say both words because of it.** Row 3 (the negative control) passed. The
  probe's controls ctlA/ctlB/ctlC and the concrete-arg pair `reply-S-concrete-*` are the discriminating
  instruments that remain.
- **Every generated service client is a generic defn that matches a parametric reply enum.** Any
  type-level projection for the message name (candidate 2 of the alias design) would be checked
  exactly where this hole is. Its negative controls would pass vacuously.
- Also measured: **D1** — a type annotation's argument count is not checked against the type's
  parameter count (`(GetResponse :- [i64 String])` on a one-parameter type, rc=0). It is caught only
  later, at unification with concrete args.

## Alias-probe outcome, recorded

- **Candidate 1 (omit the annotation) is dead.** `defn` requires `-> :Ret`. Leaving it out is
  `MalformedForm … got 2 element(s)` at the syntax layer.
- The next candidate under the spec's own table is 2 (a type-level projection). **D2 must be closed
  first**, or its controls must sit at concrete type args.
- Spec defects the executor named: the emitted return is wrapped in `RecvOutcome`, not bare; row 3
  depended on D1/D2; row 4 cannot discriminate once row 1 fails on syntax.

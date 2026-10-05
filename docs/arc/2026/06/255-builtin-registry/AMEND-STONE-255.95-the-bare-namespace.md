# AMEND — STONE 255.95: the bare namespace

**Drawn 2026-10-04.** **Executor: grok via pulsare, working solo.** Applies to 255.95 in flight; nothing else changes.

**Ruled (builder, 2026-10-04): an unqualified keyword's namespace is `$bare`.** `:k` is `{$bare, k}`: total, like
every symbol (`Identifier::namespace`'s contract: "never an absence"), and never `$bound` (a keyword is not a binding).

- Add the constant beside `BOUND_NAMESPACE` (`crates/wat-reader/src/identifier.rs:64`): `BARE_NAMESPACE = "$bare"`.
- `:k` reads as `{$bare, k}` and prints as `:k`. A namespaced keyword prints `:{namespace}/{name}`.
- No `Option` for a keyword's namespace.
- **Nobody writes `$bare` in source:** the reader refuses `$bare/x` and `:$bare/x` as it refuses `$bound/x`. One test
  holds that refusal.
- The keyword-equality gate gains a row: `:k` and `:$bare/k` cannot both be read (the second is refused), and `:k` is
  not equal to the symbol `k`.

Append to the SCORE, commit, **do not push**.

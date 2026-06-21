# NOTE — a "function not used" (dead-code) lint, NON-autofixable (2026-06-21)

Builder ask: implement, in wat's self-hosted linter, something like rust/cargo/clippy's
**`dead_code` ("function is never used")** — a name that is defined/registered but never
referenced anywhere in the corpus.

## The autofix axis
The linter carries autofix as a **per-rule flag** (`fix: Some(FixEdit)` = auto-applicable;
`None` = report-only — see `wat/lint.wat:49`, the concat→format fix is the first autofixable
rule). **This dead-code lint is explicitly NON-autofixable** (`fix: None`). You cannot blindly
auto-remove a possibly-dead function: it may be public API, a not-yet-wired hook, a
test-only helper, or reflectively/dynamically reached. Removal is a human-judgment call. So
the rule REPORTS ("`:my::ns/foo` is defined but never referenced") and stops there — never
edits.

## Why it becomes feasible (the arc-255 synergy)
This lint needs to answer "is this name referenced anywhere?" — which needs (a) the full set
of defined/registered names and (b) reference-tracking across the corpus. **Arc 255 (builtin
registry) delivers (a):** once builtins AND user forms are registered + reflectable in `sym`
(with `child-namespaces`/`names` enumeration + `metadata-of`), the linter can enumerate every
callable and cross-check it against the call-graph the rete/resolve passes already build. The
dead-code lint is a natural consumer of the 255 registry — it queries the registry for the
universe of names, the resolver/call-graph for the referenced set, and reports the difference.

## Scope marker
Not built yet — a queued lint rule for the 277 linter, unblocked by 255's registry +
reference enumeration. NON-autofixable (`fix: None`, report-only). Pairs with the registry's
reflection surface (255.2) and the resolver's reference tracking.

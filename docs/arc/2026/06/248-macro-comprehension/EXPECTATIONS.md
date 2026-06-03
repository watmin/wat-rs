# EXPECTATIONS — Arc 248 Stone 248.1 — the `for` template comprehension

Verified against an independent orchestrator re-run, not the agent's self-report.

## Gates (raw commands)

1. `cargo test --release --test probe_arc248_macro_for_comprehension` → **3 passed / 0 failed / 0 ignored** (zero `#[ignore]` left).
2. `cargo test --release --lib -p wat` → **895 passed / 0 failed / 1 ignored** (unchanged; no regression in existing macros / `,@` splice).
3. `cargo build --release --tests --workspace` → clean.

## The 2 un-ignored mints — what each proves

- `mint_for_yields_elements` — `(for [x items] ~x)` ≡ `,@items` — the `for` *iterates* and yields each element. `(:my::vof 10 20 30)` → `[10 20 30]`, first → `10`.
- `mint_for_transforms_per_element` — `(for [x items] (i64::+ ~x 1))` — the `for` *instantiates a template* per element (the generative power). → `[11 21 31]`, first → `11`.

## Hygiene + boundedness (the load-bearing correctness)

- **No leak:** the `for` binder (`x`) does not bind at the call site or persist across iterations. Confirm: a `for` binder named the same as a call-site symbol does not capture it (the existing sets-of-scopes hygiene covers this — verify it's reused, not bypassed).
- **Bounded:** `for` only iterates a finite list + instantiates a template. No recursion, no conditionals, no arbitrary computation added to the macro body. Confirm the implementation adds *only* the iterate-and-splice path — grep the diff for any new branching/recursion machinery beyond the comprehension.
- **Reuses existing hygiene:** the diff extends `walk_template`/`splice_argument`; it does NOT introduce a parallel scope/hygiene mechanism.

## Scope guard

- `for` is the ONLY new template form. No `if`/`when`/`cond` in templates, no nested-list flattening beyond the single splice, no expansion-time function calls.
- Existing `quasiquote`/`unquote`/`unquote-splicing` behavior unchanged (the regression gate + lib 895/0/1 guard this).
- No `holon-rs`.

## Hand-off

Leave all changes uncommitted. Do not commit/tag/push — the orchestrator scores against an independent re-run and commits.

# BRIEF — Arc 249 Stone 249.1 — threading desugar `->` / `->>`

**Mission.** Add Clojure's threading forms `->` (thread-first) and `->>` (thread-last) to wat as
a **macro-expansion-time desugar** in `src/macros.rs`. They rewrite to ordinary nested calls
*before* type-check; the checker and runtime never see them and need **no changes**. Full
rationale + verdict in **`DESIGN.md`** (same dir) — read it first; this is the strike order.

**The contract is the probe** `tests/probe_arc249_threading.rs` (committed, currently
`1 passed / 5 ignored`). **Done when all 6 pass with zero `#[ignore]`** and zero `-- --ignored`
needed. Do not edit the probe's assertions (FM-2-bis: the source forms are correct; make the
substrate satisfy them).

## Where — the one hook

`src/macros.rs`, function `expand_form` (line 507), the `WatAST::List` arm. Mirror the existing
**`:wat::core::keyword/of` built-in** (macros.rs:548–569): the recognition happens **after**
`expanded_children` is computed (children already recursively expanded) and **before** the
generic registered-macro dispatch (the `registry.get(head)` block at ~572).

The difference from `keyword/of`: the threading head is a bare **`WatAST::Symbol`** whose
`.as_str()` is `"->"` or `"->>"` (not a `Keyword`). So:

```text
if let Some(WatAST::Symbol(head, _)) = expanded_children.first() {
    match head.as_str() {
        "->"  => return <build thread-first nesting from expanded_children[1..]>,
        "->>" => return <build thread-last  nesting from expanded_children[1..]>,
        _ => {}
    }
}
```

Add a small helper (e.g. `fn thread(acc_and_steps: &[WatAST], last: bool, list_span: Span) ->
Result<WatAST, MacroError>`) that performs the left fold.

## The rewrite (the fold)

`(-> x s1 s2 … sN)` folds an accumulator `acc` (initially `x` = the first element after the
head) left through each step `s`:

| step `s` shape | `->` (first) | `->>` (last) |
|---|---|---|
| non-list (Symbol/Keyword `f`) | `(f acc)` | `(f acc)` |
| list `(f a b …)` | `(f acc a b …)` | `(f a b … acc)` |

- `(-> x)` / `(->> x)` (no steps) → just `x` (identity).
- `(->)` / `(->>)` (no accumulator at all) → return a `MacroError` (arity: needs ≥1 form). Use
  `MacroErrorKind` — pick the closest existing variant; if none fits cleanly, STOP and surface it
  (do not invent a kind without checking what's there).
- Each emitted call is a `WatAST::List`; carry `list_span` (the outer `->`/`->>` form's span) on
  the constructed nodes, matching how `keyword/of` / the macro-call path inherit the call-site
  span.
- The steps in `expanded_children[1..]` are **already child-expanded**; build the nesting from
  them directly and return. (Returning through `expand_form`'s normal path / a fixpoint
  re-expand is acceptable but not required — function-headed steps introduce no new macro heads.)

Worked expansions (these are the probe rows):
- `(->> [1 2 3] (:wat::core::map F))` → `(:wat::core::map F [1 2 3])`
- `(->> [1 2 3] (:wat::core::map F) (:wat::core::filter P))` → `(:wat::core::filter P (:wat::core::map F [1 2 3]))`
- `(-> 5 (:wat::core::i64::- 3))` → `(:wat::core::i64::- 5 3)`  (= 2)
- `(->> 5 (:wat::core::i64::- 3))` → `(:wat::core::i64::- 3 5)`  (= -2)
- `(-> 3 :my::inc)` → `(:my::inc 3)`  (= 4)

## Disambiguation (already free — do NOT add parser/lexer logic)

- `->>` already lexes as a single bare symbol (`is_symbol_break` in `src/lexer.rs:428` excludes
  `>`); `->` lexes today as the return arrow. **No lexer change.**
- The infix return-arrow `->` in a signature (`[a <- :T] -> :Ret`) is **never** a list *head* —
  it is a middle element of a `defn`/sig form. You recognize threading **only** when `->`/`->>`
  is `expanded_children.first()`. The return arrow is untouched. (Probe row
  `mint_thread_first_injects_first` proves both coexist: `->` is the sig arrow AND the thread
  head in the same `:user::compute` form.)

## Constraints (hard)

- **Edit `src/macros.rs` ONLY.** No edits to `src/check.rs`, `src/runtime.rs`,
  `src/special_forms.rs`, `src/lexer.rs`, `src/parser.rs`, or any `wat/*.wat`. If you believe one
  is needed, **STOP and report** — threading is pure desugar; that need signals a design miss,
  not a coding task.
- **Un-ignore all 5 mint tests** in `tests/probe_arc249_threading.rs` (delete the `#[ignore = …]`
  lines) once they pass. Leave the regression test as-is. Do not touch any other assertion.
- **HARD CUT:** threading desugars and vanishes — no runtime entity, no `special_forms.rs`
  registry entry, no `Display`. Nothing to alias or shim.
- No `holon-rs`. No new dependencies.

## Verification (you run these to self-check; the orchestrator re-runs independently)

- `cargo build --release --tests` — compiles.
- `cargo test --release --test probe_arc249_threading` — **6 passed; 0 ignored**.
- `cargo test --release --lib -p wat` — unchanged baseline (**895 passed; 0 failed; 1 ignored**).

Plain single commands, one per line. **Do not run any `./scripts/*` wrapper** and do not treat a
shell hiccup as a blocker — if a command seems unavailable, just run the vanilla `cargo`/`grep`
form. **You do not need to commit, push, or run any git command** — the orchestrator owns commits
and the gate. Report what you changed, the test output, and any STOP.

## Refs

- `docs/arc/2026/06/249-threading-macro-verdict/DESIGN.md` — verdict + mechanism.
- `src/macros.rs:548–569` (`keyword/of` precedent), `expand_form` (:507).
- `tests/probe_arc249_threading.rs` — the contract.

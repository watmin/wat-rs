# BRIEF — arc 109 γ-i: `fn` takes the `:- [T …]` binder

**Ruled:** D1 (γ-i first) · G3 (`fn` carries the binder; `def` derives; `defn` forwards into the
emitted `fn`). DESIGN: `DESIGN-STONE-gamma-i-defn-takes-the-binder.md`.

## The work, in one paragraph

`(:wat::core::fn :- [T U] [x <- :T] -> :U …)` and `(:wat::core::defn :name :- [T U] [x <- :T] -> :U …)`
must both check, and the type params must be real VARIABLES — an anonymous binder-carrying fn, bound
in a `let`, must apply at two different types. Today the binder is rejected outright, and even
without it an anonymous fn's `:T` is a rigid concrete type. `def` is NOT touched: its
`(name [meta] expr)` arity stays exactly 3-or-4.

## Read in order

| where | why you are being sent there |
|---|---|
| `wat-scripts/scratch-pad/arc109-gamma-i-anon-fn-is-rigid.wat` | **Start here.** Four rungs recording measured behaviour, with the two failing forms commented out beside their verbatim errors. Rungs 1 and 4 pass today and must keep passing. |
| `src/types.rs:4390` `parse_declared_name` + `take_declared_binder` (7 callers) | **The shape to copy.** A name-spelling reader paired with a binder consumer that ERRORS when both are present. Mirror this pairing; do not invent a second one. |
| `src/function/metadata.rs:20` `peel_metadata_preamble` | The hook. A binder peel is its sibling — same file, same shape, same two callers. |
| `src/function/eval.rs:42-66` | Peel site 1, then `type_params: Vec::new()` at `:66` — the runtime-side rigidity. |
| `src/function/infer.rs:105-140` | Peel site 2, and **the hard part**: `infer_fn` binds params into `body_locals` and checks the body with **no generalization step**. This is why `:T` is rigid. |
| `src/runtime.rs:3395-3527` `try_parse_fn_shape_def` | Reads params off the def NAME, then **Stone 251.7 (`:3499`) unions the signature's free type-vars**. The fn's binder unions in exactly there. |
| `src/runtime.rs:3551`, `:3671` | The two variadic `def`-fn recognizers. Same union, same place. |
| `wat/core.wat:673` the `defn` macro | Takes `[name & rest]`. The binder currently rides `rest` into the emitted `fn` **as a stray**, which is where the error comes from. It must be recognised and re-emitted as the fn's binder. |

## Implementation sketch

```
metadata.rs   peel_type_binder(args) -> (Option<Vec<String>>, &[WatAST])
              `:- ` is a KEYWORD (measured — not a Symbol), followed by a Vector of bare Symbols.
              Absent → (None, args) unchanged.

eval.rs:42    let (binder, sig_args) = peel_type_binder(peel_metadata_preamble(args));
   :66        type_params: binder.unwrap_or_default(),

infer.rs:105  same peel; then bind each binder name as a fresh type VARIABLE for the body,
              and generalize at the binding site.

runtime.rs    try_parse_fn_shape_def + the two variadic recognizers: read the fn's binder and
   :3499      union it where 251.7 already unions collect_free_type_vars.

core.wat:673  defn: if `rest` starts with `:-` + a Vector, take both and splice them into the
              emitted `(:wat::core::fn …)` immediately after the head.
```

## Blast radius

`wat/core.wat` · `src/function/{metadata,eval,infer}.rs` · `src/runtime.rs`.
**NOT `src/check.rs`. NOT `parse_fn_signature_prefix`'s `&[WatAST; 3]`** — that array is a
deliberate wall (*"arity is type-guaranteed"*, Stone 243.4.1); the binder is peeled BEFORE the slice,
exactly as metadata already is. **No `.wat` corpus migration** — the 40 parametric sites keep their
`<T,U>` spelling.

## What "done" looks like

1. `(:wat::core::defn :user::f :- [T] [x <- :T] -> :T x)` checks.
2. ★ An anonymous binder fn applies at TWO types:
   `(:wat::core::let [f (:wat::core::fn :- [T] [x <- :T] -> :T x) _ (f 1) __ (f "s")] nil)`.
   **One instantiation proves nothing** — a rigid `:T` passes a single application.
3. A declaration carrying BOTH `<T>` and `:- [T]` is an ERROR naming the contradiction.
4. Probe rungs 1 and 4 still pass (a no-param-list `defn` stays generic; the concrete-type HOF
   control is undisturbed).
5. A parametric **kwargs** `defn` in binder spelling mints `Kwargs<T,U>`, not a monomorphic bundle —
   `defn` derives `{b}::Kwargs{p}` from `name-tp`, the string suffix off the NAME, which the binder
   spelling leaves empty. Zero instances in `wat/`; this is a rule, not a census.
6. A **variadic** `defn` in binder spelling registers.
7. `def` of a non-fn value still registers and `def`'s arity is unchanged — the negative control.

## How to check your own work

`target/release/wat --check <file>` (~0.2s) against files you write under
`wat-scripts/scratch-pad/`, and `cargo nextest run --release -E 'binary_id(wat::function)'` for a
scoped run. Both are fast and local to your change.

⚠ **`wat/core.wat` is the stdlib and is baked in by `include_str!` at RUST-compile time.** Your edit
there is invisible to `--check` until a full rebuild, so **do not try to verify the macro half** —
edit it, describe precisely what you changed, and say so in your report. The orchestrator rebuilds
and runs the floor centrally.

## STOP triggers — ship nothing and report

- **STOP-1.** If generalizing in `infer_fn` requires changing `parse_fn_signature_prefix`'s
  `&[WatAST; 3]`, STOP and report. That array is a deliberate wall and the peel is supposed to make
  touching it unnecessary; needing to means the peel landed in the wrong place.
- **STOP-2.** If making the anonymous case generalize requires `src/check.rs`, STOP and report. G3
  was ruled specifically because `check.rs` leaves the blast radius; if it does not, the ruling
  rested on a wrong premise and the builder re-decides.
- **STOP-3.** If acceptance row 2 cannot be made to pass at TWO types, STOP and report with the
  verbatim error. Do not narrow the row to one instantiation — a single application passes while
  `:T` is still rigid, which is exactly the failure this row exists to catch.

## Your report

The diff you made, per file. Which acceptance rows you ran and their verbatim output. What surprised
you. Any site you inspected and deliberately did not change, with the reason.

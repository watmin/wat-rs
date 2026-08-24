# DESIGN — STONE: the call-site binder must be UNIVERSAL, or it is a lie

> **The defect, measured 2026-08-24 while probing an unrelated design.** Arc 109 shipped the
> call-site binder — `(:ns/f :- [:i64] 7)`, the fourth position, `69933d362` *"FINISH `:-` — four
> positions, one door; the call site now BINDS."* **Ten root-level substrate forms never learned it.**
> The CHECKER accepts the binder; the RUNTIME handler refuses it.

## Reproduction — both halves, run this session

```
(:wat::eval-ast! :- [(:wat::core::PersistentVector :- [:wat::rete::Rule])] expr)
  → #wat.runtime/MalformedForm
    "(:wat::eval-ast! <ast-value>) takes exactly 1 argument; got 3"
```

The handler counts `:-`, `[…]`, and the real argument as three. **It type-checked clean** — this is a
runtime refusal of a form the checker blessed.

★ **And the scope is NOT "generic verbs".** The obvious reading is that only generic forms can take a
binder, so only `eval-ast!` and `eval-with-defs!` (the two with `type_params`) are affected. **Measured
otherwise:**

```
(:wat::eval-edn! :- [] "42")   → takes exactly 1 argument; got 3     ← NON-generic
(:wat::eval-ast! :- [] …)      → takes exactly 1 argument; got 3     ← generic
```

**`:- []` ≡ absent, and macros emit it unconditionally** — that is arc 109's own ruling, in the seam.
So a binder can appear on ANY call, and the rule is not about genericity at all:

> **Any handler that counts its own `args.len()` must peel the param-spec FIRST.**

## Why this is a wall, not a papercut

`peel_param_spec` (`src/types.rs:4793`) is THE ONE DOOR and it has **27 callers** — `check.rs` 5,
`runtime.rs` 7, `types.rs` 9, `macros/expand.rs` 2, `types/surface.rs` 3, `function/metadata.rs` 1.
The root-level substrate forms bypass all of it: their dispatch arms
(`src/runtime.rs:6835-6845`) are thin, and the arity check lives in each helper.

This is [[feedback_a_slot_with_two_implementations_is_two_slots]] in a slot **arc 109 itself
shipped** — the checker learned the fourth position, one family of handlers did not. A binder that
works everywhere except ten forms is not a language feature; it is a trap that fires only when
someone reaches for those ten.

⚠ **And it blocks real work.** The wat-grep rete-processor design needs
`(:wat::eval-ast! :- [T] expr)` to type the chamber's return — `eval-ast!` is registered
`∀T. WatAST -> (Result :- [T EvalError])`, so **the caller MUST bind `T`**, and today it cannot. The
chamber is otherwise proven: a query file read at runtime, eval'd in the frozen world, compiled into
a rete session. Only the binder is missing.

## The rooms

1. **`src/runtime.rs:6835-6845`** — the dispatch cluster. Ten arms, each `":wat::eval-…!" =>
   eval_form_…(args, env, sym, list_span)`. The `args` handed down still carry the param-spec.
2. **The ten helpers**, each of which counts for itself. Five carry a `takes exactly N argument`
   message and are findable by it:
   `eval_form_ast` (`:28841`) · `eval_form_with_defs` (`:29053`) · `eval_form_step` (`:29360`) ·
   `eval_form_edn` (`:30891`) · `eval_form_file` (`:30920`).
   **The other five — `eval_form_digest`, `eval_form_digest_string`, `eval_form_signed`,
   `eval_form_signed_string`, `eval_walk` — were NOT surfaced by that message grep.** Their arity
   handling must be read, not assumed; the message text was one instrument and it saw five of ten.
3. **`src/types.rs:4793`** — `peel_param_spec(args) -> (Option<&[WatAST]>, &[WatAST])`. `:- []` peels
   to `Some(&[])`, never `None` — so "was a binder written" and "was it empty" stay distinguishable.

## The contract decision, pinned

**Peel ONCE, at the dispatch cluster — not ten times in ten helpers.**

The arms at `6835-6845` are the single place every one of these forms passes through. Peeling there
hands each helper a clean `args`, and a new eval form added later inherits the fix instead of
re-earning the bug. Peeling per-helper is ten edits, ten chances to miss one, and no structural
guarantee for the eleventh.

**What to do with the peeled binder:** for the two generic forms it is the caller's type argument and
must reach the checker's existing binding path — which already works, since the call type-checked.
For the eight non-generic forms an EMPTY binder is `≡ absent` and is simply discarded; a NON-empty
binder on a non-generic form should be **refused with a diagnostic that names the verb**, not
silently ignored.

## Out of scope — affirmatively cut

- **`load-file!` / `signed-load!` and the load family.** They have no runtime arm (resolved at FREEZE
  time, `freeze.rs:1906`) so they cannot receive a runtime binder. Verified.
- **Auditing every other self-counting handler in the substrate.** The general call path peels; this
  stone is the root-level substrate cluster that bypasses it. A wider audit is its own census and
  should be imposed as a check, not grepped.

## Acceptance

- `(:wat::eval-ast! :- [T] expr)` binds `T` and returns `(Result :- [T EvalError])` — the wat-grep
  chamber probe runs end to end.
- `(:wat::eval-edn! :- [] "42")` behaves EXACTLY as `(:wat::eval-edn! "42")` — the empty binder is
  absent, per arc 109's ruling.
- A non-empty binder on a non-generic eval form is refused, and the error NAMES the verb.
- All ten forms covered — including the five the message-grep could not see. **The acceptance row is
  the classification of all ten, not the five that were easy to find.**
- Floor green with every move accounted by name; clippy 0.

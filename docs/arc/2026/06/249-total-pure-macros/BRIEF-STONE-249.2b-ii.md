# BRIEF — Arc 249 Stone 249.2b-ii — the body-model (program bodies via `macro_eval`)

**Mission.** Let a macro body be a **combinator program** (not just a quasiquote template), evaluated
at expand time by `macro_eval`. Turns probe gates **C + D green** (an `if`-body and a `foldl`-body),
adds **gate E** (a name-introducing program body is REFUSED — the hygiene bound). The arc's most
intricate stone — but the design floor (below) makes it un-break-able for existing macros.

## CRITICAL — no regression (read first)

Every existing stdlib macro body is a **bare quasiquote** and gets sets-of-scopes hygiene from
`walk_template`. **DO NOT route bare-quasiquote bodies through `macro_eval`** — that would strip
their hygiene and break the whole stdlib. The dispatch is the *existing* `expand_template` check
(expand.rs:443, "is the body a 2-elem `(quasiquote X)` list?"):
- **bare quasiquote body → `walk_template` (the EXISTING path, UNCHANGED).** Hygiene preserved.
- **non-quasiquote (program) body → the NEW `macro_eval` path** (this stone).

The 898 lib tests + every existing macro test MUST stay green — proof the existing path is untouched.

## The new program path

When `expand_template` sees a body that is NOT a bare quasiquote, instead of `UnsupportedBody`:

1. **Bind the macro params as QUOTED FORM-VALUES** in an `Environment` for the body eval:
   - each fixed param: its arg-FORM wrapped as a `Value::wat__WatAST(arg_form)` (a quoted form).
   - the variadic (`& rest`): a `Value::Vec` of the rest arg-forms, each as `wat__WatAST` — a
     **Vector value** the HOFs can iterate (NOT the current `List`-form binding at expand.rs:430).
   This is what lets `(foldl … nums)` fold over the arg-*forms*: each `n` is a `wat__WatAST` value;
   `` `(:wat::core::i64::+ ~acc ~n) `` → `eval_quasiquote` evaluates `~n` → `value_to_watast` →
   the arg-form spliced in. Correct for literal args (`1`) AND expression args (`(f x)`).
2. **`macro_eval(body, &body_env, sym)`** — the fenced evaluator (validate_pure_total + eval). The
   body is a program (`foldl`/`if`/`fn`/quasiquote); its result is a `wat__WatAST` (or a value
   `value_to_watast` converts) → the expansion form.
3. **`eval_quasiquote`** (runtime.rs:10357, already handles `~`/`~@` against the env) is the
   form-builder for the program's inner `` `(…) `` — no new quasiquote machinery.

The exact composition (env construction, how `macro_eval`'s result becomes the expansion form) is
discovered against the probe — **substrate-as-teacher: iterate until gates C/D green.** The probe
defines correct behavior; do not edit its assertions.

## Hygiene bound — gate E (refuse name-introducing program bodies)

`eval_quasiquote` does NOT add hygiene scopes. A program-body quasiquote that **introduces a name**
(a `:wat::core::let` / `:wat::core::fn` binder with a *literal*, non-unquoted name) could capture.
Per the four-questions verdict (DESIGN § 2b-ii): **default-deny — REFUSE such a body** (a clear
error; reuse `RefusedInMacro` or add a sibling variant via the existing pattern), rather than ship a
silent capture bug. The idiomatic non-introducing family (threading/`cond->`/`when`) is unaffected;
full eval-time-hygienic quasiquote is a named follow-on.
Detection: walk the program body's quasiquote templates; if a binder position holds a literal name
(not a `~`-unquote), refuse. (A bare-quasiquote body is NOT affected — it uses hygienic
`walk_template`.)

## Probe (extend `tests/probe_arc249_macro_engine.rs`)

- **Un-ignore C** (`mint_program_body_if`) and **D** (`mint_program_body_fold`) — they must pass.
- **Add gate E** — a program body whose quasiquote introduces a literal name (e.g.
  `` (:wat::core::if (:wat::core::= 1 1) `(:wat::core::let [tmp ~x] tmp) `~x) ``) → `startup_ok`
  must be FALSE (refused). (At HEAD it's `UnsupportedBody`; after, it's the hygiene refusal — either
  way refused, but the gate locks the BOUND so a future "allow program bodies" can't silently admit
  the capturing case.)
- **A + B stay green** (F5 closure intact); the regression macro tests stay green.

## Constraints (hard)
- Edit: `src/macros/expand.rs` (the `expand_template` program path + the param-value binding),
  possibly `src/macros/eval.rs` (if `macro_eval` needs a body-entry variant), `src/macros/error.rs`
  (if a new refusal variant), `tests/probe_arc249_macro_engine.rs`. No other file — STOP if it seems
  needed.
- **Do NOT touch `walk_template` or the bare-quasiquote path.** Existing hygiene is sacred.
- No new deps. No `holon-rs`.

## Verify (plain single commands; vanilla cargo — no `./scripts/*` wrapper)
- `cargo build --release --tests`
- `cargo test --release --lib -p wat`  → **898 passed; 0 failed; 1 ignored** (UNCHANGED — proves no
  regression on existing macros)
- `cargo test --release --test probe_arc249_macro_engine`  → **5 passed; 0 failed; 0 ignored**
  (A/B/C/D + E all green)

Do NOT commit, push, or run git — the orchestrator owns commits + the gate. Report: the diff stat,
the program-path code (paste it), the param-value binding, the command outputs, and any STOP.

## Refs
- DESIGN-STONE-249.2b.md § "2b-ii — the body-model build" + § "The body model — ONE kind".
- `expand_template` (expand.rs:443, the dispatch); `walk_template` (the hygienic path — untouched);
  `eval_quasiquote` (runtime.rs:10357); `value_to_watast` (:10458); `macro_eval` (src/macros/eval.rs).
- The contract: `tests/probe_arc249_macro_engine.rs` gates C/D (+ new E).

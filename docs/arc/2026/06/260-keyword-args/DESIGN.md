# Arc 260 — keyword / named arguments (the form should say what it means)

> **STATUS: STUB — queued, non-blocking.** Surfaced 2026-06-12 by the builder during the
> `deftest'` build (arc 259 S3.5a). Not started. This file is the context breadcrumb so the
> arc can be picked up cold.

## The trigger

Writing a probe, the form `(:wat::kernel::assertion-failed! "msg" :wat::core::None :wat::core::None)`
stopped the builder cold:

> *"i've been staring at the none/none and some/some without any sigils as to wtf they are…
> can we make those :kwargs or something? :reason :expected :actual — or whatever, i legit
> don't know the ordering."*

That last clause is the whole bug: **a competent reader cannot tell, from the form, what the
trailing positional `:None :None` mean — nor in which order.** The form refuses to carry its own
meaning. That is the exact failure the substrate exists to abolish (legibility-by-design; the
form and the prose on one channel — see the recenter interlude / #93).

## Grounded facts

`assertion.rs:54-57`:

```rust
pub struct AssertionPayload {
    pub message:  String,
    pub actual:   Option<String>,
    pub expected: Option<String>,
    // … location, frames, chain (internal)
}
```

So `(:wat::kernel::assertion-failed! <message> <arg2> <arg3>)` is **`(message, actual, expected)`**
— and `actual`-before-`expected` is precisely the order a writer flips by accident. Every
`… :None :None` / `… (:Some a) (:Some b)` call site is illegible and order-fragile.

Call sites today: `assertion-failed!` is called from `wat/test.wat` (the assert helpers) and is
the panic primitive every `assert-*` lands on. The opacity is widespread but low-stakes
(non-blocking).

## The reach, and the deeper question

The builder reached for **keyword args** — `:actual …`, `:expected …`. The load-bearing
question this arc must answer first:

> **Does wat have keyword / named arguments? If not, that is the real reach-stumble** — the
> substrate is missing a tool an LLM (and a human) instinctively reaches for, and the fix is to
> make it, not to nest a workaround.

Grep `keyword`/`kwarg`/named-arg support in the parser + checker before designing; do NOT assume.

## GROUNDING (2026-06-16, orchestrator crawl — kwargs ARE a real new feature)

The question is answered: **wat has NO call-site keyword/named arguments. This is the real
reach-stumble** — a genuinely missing tool, not a workaround-able gap.

- **Everything is positional, end to end.** Call args parse to a positional `Vec<WatAST>`
  (`parse_list_body`); the checker matches by arity + positional index; eval binds via
  `func.params.iter().zip(args.iter())` (`runtime.rs:23080`, `:9156`; defclause `:6166`/`:5035`).
- **No existing named-arg machinery.** Every `keyword arg` hit in `src/` is something else: TYPE-keyword
  args (`:S :R` type params), `use!`'s single path keyword, or namespace keywords (`string_ops`). None is
  general named call args.
- **BUT the foundation exists: fn signatures retain param NAMES** — `Function.params: Vec<String>` +
  `param_types` + `rest_param` (`runtime.rs:886-901`; `function/parse.rs:54`). So matching `:name val`
  call-args to param names and reordering to positional is BUILDABLE on existing data.
- **Architectural constraint (the load-bearing design fact):** reordering `:name val` → positional needs
  the **callee's param names AT the call site**. The PARSER does not know the callee's signature, so kwargs
  CANNOT be a pure-parse desugar. It must happen **after the callee resolves** — at the call chokepoint
  (`apply_function`, `runtime.rs:17990`) for eval, and in call-inference for check.
- **OPEN sub-question to pin FIRST in the design:** does the call-site CHECKER carry the callee's param
  NAMES, or only `param_types` (via the derived scheme, for unification)? `param_names` is present at fn
  DEFINITION check (`function/infer.rs:115`); whether the CALL-site scheme retains names determines how
  option 1 reorders before unification. Crawl this before drawing the stone.

**Implication for the design space below:** option 1 (real kwargs) is a genuine language feature touching
the call chokepoint in both check + eval, with a type story — feasible (names exist) but not free. Option 2
(record-arg) needs zero new call machinery (records already carry field-name labels) and fixes
`assertion-failed!` immediately, but doesn't generalize to every call site. Four-questions them with this
grounding; a disconfirming probe (does a `:name val` reach `apply_function` with the param names available
to reorder?) settles option 1's feasibility before any build.

## PROBE FINDINGS (2026-06-16) — the trigger is the hardest case; option 1 decomposes

Four-questions gauge (builder + orchestrator): hard constraint = **generality** (label call sites across
the language — the arc's thesis). Option 2 (record-arg) FAILS it (one verb, doesn't generalize) → it's a
local patch, not the feature. Option 1 (real kwargs): Obvious YES, Honest YES, UX YES; **Simple was the sole
open axis** → settled by the probe below. Builder's pick: **Option 1.**

Disconfirming probe `tests/probe_arc260_keyword_args.rs` (#[ignore], RED at HEAD): `(:user::sub :b 3 :a 10)`
must reorder by param name to `(sub 10 3) = 7`; at HEAD startup fails (positional only). The probe + the
call-path crawl found the **decisive split**:

| | USER fns (`defn`) | INTRINSICS (`assertion-failed!` — the trigger) |
|---|---|---|
| eval | `func.params` names ✓ (`apply_function`) | Rust fn, positional, **NO names** |
| check | names via `sym.functions[path].params` ✓ (not in the scheme) | scheme is `params: Vec<TypeExpr>` — types only, **NO names anywhere** (check.rs:79-81, `assertion-failed!` :14391) |

**`TypeScheme` carries param TYPES, not names.** So the arc's own TRIGGER (`assertion-failed!`, an intrinsic)
is the hardest case — kwargs for intrinsics needs param-name infrastructure ADDED. **Simple = NO at full
scope → DECOMPOSE** (per the parity lesson: a hard-constraint-forced path with Simple=NO means decompose,
not abandon):

- **260.1 — user-fn keyword args (the mechanism).** Reorder `:name val` → positional at the call chokepoint:
  check via `sym.functions[path].params`, eval via `apply_function`/`func.params`. Names already exist; this
  proves the reorder + the type story (reorder before unification) on the data we have. The committed probe
  is its gate. Probe-first per sub-stone.
- **260.2 — intrinsic keyword args (reach the trigger).** Give schemes param names (a `TypeScheme.param_names`
  field; populate `derive_scheme_from_function` from `func.params` + the kernel-verb `env.register` sites) +
  the intrinsic-call eval reorder. THIS is what lets `assertion-failed!` (and every kernel verb) take kwargs.
- **260.3 — migrate `assertion-failed!`** (the original trigger) to kwargs + its call sites; close the arc.

Each stone is flat-Simple; the bundle was not. Build 260.1 first (it's the mechanism on existing data; 260.2
extends it to the nameless intrinsics).

## Design space (to weigh with the four questions when the arc opens)

1. **Keyword arguments** — `(assertion-failed! :message "…" :actual … :expected …)`. The general
   capability; clojure-idiomatic (`& {:keys […]}` / maps-as-kwargs). Biggest surface, widest payoff
   (labels every call site in the language, not just this one). If wat lacks them, this is a real
   language feature with its own type story (typed kwargs are non-trivial).
2. **A record argument** — `(assertion-failed! "msg" (AssertionDetail :actual … :expected …))`.
   Reuses the EDN-native record surface (arc 257); no new call-arg machinery. The labels live in the
   record's field names. Narrower; doesn't generalize to other verbs.
3. **Named-positional via a labeled enum / tuple** — weakest; still positional under the hood.

Lean (un-grounded, for the future self to re-decide): if wat genuinely lacks keyword args, the
record-arg (option 2) is the cheap honest fix for `assertion-failed!` *now*, and "keyword args in
wat" is its own larger arc to weigh on its own merits. Don't conflate the two unless grounding
shows kwargs are cheap.

## Scope / discipline

- **Non-blocking.** It breaks nothing and fixes nothing functional; it is a legibility/ergonomics
  debt. Do it when the velocity allows, not as a dependency.
- When opened: ground the kwargs question FIRST (parser/checker), then four-questions the design
  space, then a disconfirming probe, then build through a shadowdancer + weigh (the arc-259 rhythm).
- Pairs the prose-and-form thesis: a call form that needs a comment to decode its own arguments is
  the thing the comm channel exists to kill.

## RELATED — the macro-time sibling (the kwargs pattern already exists, inlined)

This arc is RUNTIME call-site keyword args. Its sibling is **macro-time options parsing**, and that
pattern already shipped inlined in `defservice` (2026-06-16, arc-272 rs-1, `wat/service.wat` ~line 67):
a variadic defmacro takes trailing `[:key val :key val …]`, folds them into a HashMap in one pass
against a `known-opts` set (rejecting any unknown key with a named macro-error), then reads each option
as `(HashMap/get opts-map "<key>")` with a default. Adding an option = one `known-opts` entry + one
`get`. This IS the canonical "macro with optional keyword-options" shape (Clojure `& {:keys}`, CL
`&key`, Python `**kwargs`).

**Why it's inlined, not extracted (yet):** the macro-eval fence CANNOT call user-defined wat fns
(`feedback_does_a_macro_need_it_intrinsic_boundary`), so a shared `parse-macro-opts` helper would have to
be a **Rust intrinsic** on the macro-eval allow-list (the "does a macro need it → intrinsic" boundary),
or this arc's facility generalized to expand-time. Per rule-of-three: defservice is the ONLY consumer
today; extract when a SECOND option-taking macro appears (generalizing from N=1 risks baking in
defservice's specifics). When this arc is built, weigh folding the macro-time pattern in (or minting the
intrinsic) so the two layers share one kwargs story.

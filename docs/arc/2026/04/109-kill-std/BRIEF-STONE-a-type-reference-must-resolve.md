# BRIEF — a type reference must RESOLVE

DESIGN: `DESIGN-STONE-a-type-reference-must-resolve.md`. **RULED: D1-A · D2-A · D3-B.**
⛔ **Read the DESIGN's "GROUNDED AFTER THE RULING" section first.** Three of its four findings change
the shape of the work, and two of them produce a failure that reads as success.

## The work in one paragraph

A declaration may name a type that does not exist and nothing says so. Add a pass, in the RESOLVER
(step 7), that sweeps the registered declarations, walks each declared type expression, and reports
any name that is neither a registered type nor a type variable bound by the enclosing declaration.
Every namespace, `:wat::*` included. Declared positions only.

## Read in order

1. `src/freeze.rs:886-894` — the eleven-step order. **Step 5 registers types; step 7 resolves.** That
   ordering is the whole reason this pass is cheap: the `TypeEnv` is fully populated when you run.
2. `src/freeze.rs:1279-1315` — the resolve/check precedence. **This is STOP-2. Read every line.**
3. `src/resolve/walk.rs:29` — `resolve_references(forms, sym, macros)`. Your entry point. It needs a
   `&TypeEnv` parameter; `bundle.types` is in scope at the call site.
4. `src/resolve/walk.rs:259` — `is_resolvable_call_head`. **The shape to mirror**, including its
   `UnresolvedReference { path, context }` payload. Note its reserved-prefix exemption and note that
   D3-B does NOT copy it — the DESIGN's D3 table says why.
5. `src/types.rs:522` — `TypeEnv::contains(&str) -> bool`. The door. Already covers builtins
   (`with_builtins` → `register_builtin_types`).
6. `src/types.rs:60-100` — `TypeExpr`. The tree you walk: `Path` · `Parametric{head,args}` ·
   `Fn{args,ret}` · `Tuple` · `Var`. ⛔ Read the `Path` doc comment — it is the constraint below.
7. `src/check.rs:80` — `TypeScheme { type_params, params, ret, rest_param_type }`. The function-side
   record: scope and payload on the same struct.
8. `src/types.rs:3341` — `register_types_impl`. Proof that type declaration forms are CONSUMED and
   never reach step 7. This is why the pass is a registry sweep.

## ⛔ The constraint that decides the implementation

**Type variables parse as `TypeExpr::Path`.** `T`, `K`, `V` and `NoSuchType` are the same node kind;
`TypeExpr::Var(u64)` is synthetic and never comes from parsing. A walk that asks `contains` on every
`Path` will flag every type variable in the corpus — hundreds of them.

The walk MUST carry the enclosing declaration's `type_params` as a bound set. Both carriers already
exist: `TypeScheme.type_params` for functions, and a `type_params: Vec<String>` on each of `TypeDef`'s
six variants for types.

## The sweep

Iterate the registries, not the forms:

- **Types** — every `TypeDef` in the `TypeEnv`. Walk field types (Aggregate), variant payloads
  (Enum), the aliased expression (Alias), the newtype's inner, union members, surface method
  signatures. Bound set = that def's own `type_params`.
- **Functions** — every registered function's `TypeScheme`. Walk `params`, `ret`, and
  `rest_param_type`. Bound set = that scheme's `type_params`.

For each `TypeExpr`, recurse structurally and check every `Path` and every `Parametric.head`:
resolved if `TypeEnv::contains(name)` OR the name is in the bound set. Otherwise it is an
`UnresolvedReference` whose `context` names the slot — e.g. `type in the signature of :user::f,
parameter #1` / `field type of :user::R.n`.

⚠ `SymbolTable.functions` is private (`src/value/symbol_table.rs:33`). Adding a read-only iterator
accessor is in scope; changing its representation is not.

## STOP triggers — ship nothing further and report

- **STOP-1 — the D4 scope question. Do NOT invent an answer.** Whether a nested `fn`'s binder extends
  or shadows its enclosing `defn`'s scope is NOT probeable with today's binary (both spellings exit 0,
  and exit 0 proves nothing while type names go unresolved). READ how the checker builds scope
  (`check.rs:65` documents `type_params` as the ∀-bound list) and mirror it exactly. **If the checker
  has no nested-scope concept, STOP and report** — the pass must not mint a scoping rule the language
  does not have.
- **STOP-2 — the precedence rule.** `freeze.rs:1308` re-raises the resolve error only when EVERY check
  error is an `UnknownCallee`. A phantom type WITH a caller produces a `TypeMismatch`, which is not an
  `UnknownCallee`, so **your new diagnostic will be discarded in exactly the case that motivates this
  stone.** Row 2 below is the row that catches it. Fixing the precedence is IN scope — but the rule is
  load-bearing for three named contracts (`resolve_error_bubbles_up`, the REPL's bad-line arm,
  `unknown-call-head-panics`, all cited in the comment there). **If you cannot make row 2 pass without
  disturbing those, STOP and report what the precedence needs.** Do not weaken row 2 to fit.
- **STOP-3 — if the corpus screams.** D3-B exempts nothing, so the stdlib is in scope and may hold
  existing violations. That is the point, not an obstacle. But if the count is large or any violation
  looks like a real bug rather than a phantom name, STOP and report the list before fixing anything.
  **Do not "fix" a stdlib type name to make your pass green** — a genuine unresolvable type in `wat/`
  is a finding that outranks this stone.

## Acceptance — every row's CURRENT output is recorded, measured today

⛔ Two shapes of a probe I ran this session returned five greens while measuring nothing. Every row
below names a command whose behaviour TODAY I have captured verbatim, so "it passes" cannot be
confused with "nothing looked."

| # | case | command | TODAY | REQUIRED |
|---|---|---|---|---|
| 1★ | phantom in an **UNCALLED** declaration | `--check` on `(defn :user::f [x <- :user::NoSuchType] -> :i64 0)` | **EXIT 0** | EXIT 1, naming `:user::NoSuchType` at the declaration |
| 2★★ | phantom **WITH a caller** | same, plus `(:user::f 5)` in a `main` | EXIT 1, `TypeMismatch: "parameter #1 expects :user::NoSuchType; got :wat::core::i64"` | EXIT 1, naming the unknown TYPE — **not** a mismatch blaming the caller |
| 3 | phantom in a **return** slot | `-> :user::NoSuchType` | EXIT 1, `ReturnTypeMismatch: "body produces :wat::core::i64; signature declares :user::NoSuchType"` | EXIT 1, naming the unknown type |
| 4 | phantom as a **parametric form** | `(:wat::cache::NoSuchType :- [:i64])` in a param | **EXIT 0** | EXIT 1 |
| 5 | phantom in a **field** type | a `defrecord` field typed `:user::NoSuchType` | measure it yourself first, record it | EXIT 1 |
| 6✅ | **forward references still legal** | `defn` naming a type declared LATER in the file | EXIT 0, and it genuinely resolves | **still EXIT 0** |
| 7✅ | **type variables still legal** | `scripts/floor.sh` | 4859/4859 | **4859/4859** |

**Row 2 is the stone.** Row 1 alone can pass while every called phantom keeps its old message — see
STOP-2. A stone that ships row 1 without row 2 has fixed the case nobody hits.

**Row 6 is the negative-space control.** Forward references ARE legal — proven with a control, since
exit 0 alone means nothing here: passing a real `:user::Later` checks clean, passing an `i64` fails
with *"parameter #1 expects :user::Later; got :wat::core::i64"*. A pass that validates in
registration order would break this. If row 6 goes red you have built option D1-D, which was rejected.

**Row 7 is the mass control.** `wat/` is full of parametric declarations. A pass that mishandles the
bound set lights up in the hundreds. A green floor here is meaningful precisely because the corpus is
large — it is the only row that proves the `type_params` scoping works at scale.

Rows 1-5 become permanent tests in the same commit as the fix, under `tests/` — **not** under
`wat-scripts/`, which is loader-gated by `tests/lint/wat_scripts_fixes_load.rs` and where a must-fail
fixture is a red floor. Copy `tests/macros/probe_arc279_format.rs` for the shape
(`startup_from_file` → assert `is_err()` → `wat::assert_edn_matches_file!` against a golden).

## Boundaries

- `src/resolve/`, `src/freeze.rs` (the one call site, plus the precedence if STOP-2 allows),
  `src/value/symbol_table.rs` (a read-only iterator accessor only), and new tests under `tests/`.
- Do NOT touch `is_resolvable_call_head`'s reserved-prefix exemption for CALL heads. That half is
  late-but-honest — its deferral target genuinely fires (`UnknownFunction` at runtime, verified) —
  and it is a separate question nobody has asked.
- Do NOT check inline `let`/`match` ascriptions inside function bodies. D2-B was rejected; those are
  checked at use by the existing unifier, and that position works.
- Do NOT run `scripts/floor.sh` or a full `cargo nextest`. The orchestrator measures centrally. Your
  own checks: `target/release/wat --check` on your fixtures, and scoped
  `cargo nextest run --release -E 'binary_id(wat::types)'` / `'binary_id(wat::resolve)'`.
  ⚠ A scoped run is not the floor — a recent stone was 133/133 green on its own binary while the
  floor was red in `wat::lint`.
- Do NOT commit, push, stash, revert or amend. Leave everything in the working tree.

Prefix long commands with `systemd-run --user --scope -q -p MemoryMax=16G -p MemorySwapMax=0 timeout 900`.
Read exit codes DIRECTLY — never through a pipe, and never after a trailing `; echo` (that reports the
echo's status; it masked a red floor for me this session).

## Your report

Every acceptance row with verbatim output, rows 2 and 7 especially. What the checker's scope
construction actually says (STOP-1) and what you mirrored. Whether the precedence needed changing and
what you did about the three contracts it carries. The full list of any stdlib violations the sweep
found, BEFORE you touch any of them. What surprised you. Anything you inspected and left alone, with
the reason.

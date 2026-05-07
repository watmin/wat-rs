# Arc 157 — INSCRIPTION

**Inscribed 2026-05-07 by orchestrator.** All slices shipped.

## What shipped

`:wat::core::def` minted as the foundational top-level value-binding
special form. Module env IS the ambient monad; `def` is the
state-modifying operation on it. Type inferred from the expression;
no annotation on the form. Strict-default redef-error; opt-in
relaxation via `:wat::config::set-redef!` with mandatory type-
stability check.

End-to-end working: `(:wat::core::def :pi 3.14159)` registers
`:pi` as `:wat::core::f64` at type-check AND binds `:pi` to value
`3.14159` at runtime; subsequent expressions resolve `:pi` to the
bound value (including inside nested let bodies that were
themselves the def's home).

## Slices

| Slice | Commit | What landed |
|---|---|---|
| 1a-i | `b10e998` | def special form + position predicate + `defined_values` carrier (CheckEnv) + runtime binding via `register_runtime_defs` + 14 tests |
| 1a-ii | `28afc5b` | `:wat::config::set-redef!` + `:wat::config::set-eval-redef!` + opt-in gating in `infer_def` + type-stability check + `DefRedefTypeChange` variant + 5 tests |
| 1b | (empty) | Consumer migration: zero call sites; `def` is a brand-new form; no workarounds to migrate |
| 2 | (this commit) | Closure paperwork |

## Settled design

**Top-level position rule (Q1)** — Clojure splice semantics:
- `def` is legal at top-level position only
- Top-level position = file form list, OR direct child of a top-
  level `do`, OR direct child of a top-level `let` body
  (recursive)
- Both `do` and `let` splice; `let` body's locals can be captured
  as closures by the def's expression (the load-bearing case for
  shared module-state setup)
- Conditional (`if`/`when`/`cond`/`match`/`Result/try`/`Option/try`)
  + function bodies + iteration constructs all reject `def` —
  they'd violate the once-per-load-time-execution contract `def`
  needs

**Re-binding discipline (Q2)** — opinionated opt-in:
- Default: every redef is an error (`DefRedefForbidden`)
- `(:wat::config::set-redef! :wat::core::true)` permits redef
  WHEN type stays stable
- `(:wat::config::set-redef! :wat::core::false)` re-asserts
  strict default
- `(:wat::config::set-eval-redef! ...)` mirrors for eval-time
  flow (carrier present; gating inert because eval-time def-
  binding is not wired in arc 157 — see § Out of scope below)
- Type-stability mandatory whenever redef happens regardless of
  flag (`DefRedefTypeChange` rejects type drift even with
  set-redef! on)
- Config-preamble semantics: setters are file-level, processed
  before any program form. Mid-program toggle is not
  expressible in this convention — matches existing
  `:wat::config::set-*!` precedents (`set-capacity-mode!`,
  `set-presence-sigma!`, etc.)

**Namespacing (Q3)** — substrate has no namespace concept:
- `def` accepts any keyword as the name — bare (`:pi`) or
  FQDN (`:my::app::pi`)
- Wat-provided forms ship at FQDN paths (`:wat::core::def`,
  `:wat::config::set-redef!`); user code chooses freely;
  substrate doesn't enforce

## Implementation surface

`src/special_forms.rs`:
- `:wat::core::def` registered with sketch `["<name>", "<expr>"]`
- (Two `:wat::config::set-*!` setters parsed in `src/config.rs`
  per the existing config-preamble convention)

`src/runtime.rs`:
- `SymbolTable.defined_values: HashMap<String, (TypeExpr, Span)>`
  — declared in 1a-i for shape; populated at check time
- `SymbolTable.runtime_def_values: HashMap<String, Value>` —
  populated at freeze step 9.5 by `register_runtime_defs` /
  `register_runtime_defs_form`
- `SymbolTable.redef_allowed: bool` (default false) +
  `SymbolTable.eval_redef_allowed: bool` (default false) —
  the opt-in flags
- Eval's keyword arm consults `runtime_def_values.get(k)` AFTER
  `unit_variants`, BEFORE function lookup
- `dispatch_keyword_head` checks `runtime_def_values` for callable
  values when the symbol isn't a registered function

`src/check.rs`:
- `infer_def` runs the position check (recursive walker
  `validate_def_position_with_wrapper`), inference, and
  three-way redef gating
- `CheckEnv.defined_values + defined_value_spans` populated
  sequentially as forms are processed
- `CheckEnv.redef_allowed` mirrored from `SymbolTable.redef_allowed`
  in `from_symbols`
- `DefNotTopLevel`, `DefRedefForbidden`, `DefRedefTypeChange`
  CheckError variants

`src/freeze.rs`:
- `:wat::core::def` recognized as top-level form alongside
  `define`, `defmacro`, `define-dispatch`, `struct/enum/newtype/
  typealias`
- Step 7.5: `symbols.redef_allowed = config.redef_allowed`
  BEFORE `check_program` runs at step 8 (timing-critical)
- Step 9.5: `register_runtime_defs` walks top-level defs
  (including via `do` / `let` splice) and populates
  `runtime_def_values`

`src/config.rs`:
- `Config.redef_allowed + Config.eval_redef_allowed` (default
  false) + `parse_bool` helper + parsing arms for the two
  setters (mirror `set-capacity-mode!` / `set-presence-sigma!`
  shape)

## Tests

`tests/wat_arc157_def.rs` — 19 tests, all green:

- 4 basic binding (literal/computed/type-mismatch-via-registered/
  expr-error)
- 4 legal positions (top-level / do-splice / let-splice-with-
  closure / recursive let-do)
- 2 illegal positions (if / define-body)
- 1 strict-default redef collision
- 3 runtime resolution (pi resolves to f64, user's exact let+pi
  example, let-splice closure capture of let-local through def-
  bound fn)
- 5 redef-discipline (default-off / set-redef!-true-same-type /
  set-redef!-true-diff-type / set-redef!-false-explicit /
  set-eval-redef!-surface)

Workspace: 2010 baseline + 19 = 2029 tests; 0 failed; 0
warnings.

## Out of scope

**Eval-time `def` binding + gating.** The `set-eval-redef!`
flag is functional on the SymbolTable + Config carriers but the
gate is inert at runtime because eval-time `def` binding is not
wired (the eval arm at `dispatch_keyword_head` returns
`Value::Unit`; `register_runtime_defs` handles freeze-time
only). A new arc opens IFF a caller surfaces wanting eval-time
`def` redef (e.g., an interactive `eval-ast!` flow that needs
to mutate module state per evaluation).

This is an affirmative scope cut, not a deferral: the foundation
honestly carries the surface for future wiring; no caller today
needs eval-time redef; when one does, the work is targeted and
small (extending the eval arm + adding the runtime gating
parallel to the freeze-time gating).

**`define` retirement.** User direction 2026-05-07: *"define
will be swapped to a wrapper on (def :name (fn ...)) - don't
worry about this for now. let's get (def ...) as a foundational
form first."* Arc 157 ships ONLY the foundation per that
direction; the `define` retirement is explicitly its own work,
opened separately when the user directs.

## Slicing rationale

Recovery doc § 5 + memory `feedback_stepping_stones_proactive.md`
guided the in-arc re-slicing:

- Original BRIEF bundled def + redef gating in one ~90-min slice
- User direction surfaced runtime-binding requirement mid-flight
  ("(def pi 3.14) -> pi is now an f64; (let [x 2] (+ x pi)) ->
  f64 as 5.14 — this is what we want to enable")
- First sonnet sweep deferred runtime binding silently;
  orchestrator pushed back rather than accepting deferral
- Re-spawned focused on runtime binding; sonnet shipped + made a
  root-cause fix (function-body checks running before def-
  registration; reordered)
- 1a-ii (gating) shipped on the settled foundation 1a-i
  established — cleaner per-piece verification, smaller per-piece
  cognitive surface, opt-in default catches typos by default
  while permitting hot-reload-style flow on opt-in

The discipline held: each shipped slice is a complete, useful
piece on its own.

## Cross-references

- **Arc 145** (typed-let) — back-out lesson cited in DESIGN:
  don't add type annotations when substrate inference suffices.
  `def` carries no `-> :T`.
- **Arc 154** (kill let*) + **Arc 155** (fn rename) — closest
  precedents for adding/modifying top-level forms via the
  substrate-as-teacher recipe.
- **Memory `feedback_substrate_already_typed.md`** — paid-for
  lesson from arc 145.
- **Memory `feedback_stepping_stones_proactive.md`** — proactive
  slicing framework codified during arc 157.
- **Memory `feedback_paperwork_orchestrator_side.md`** — this
  INSCRIPTION written orchestrator-side.
- **Recovery doc § 5 (Proactive slicing — stepping stones)** —
  the framework itself.

## Commit chain

- `443deaa` arc 157 opens (DESIGN + BRIEF-1a-i + EXPECTATIONS-1a-i
  + recovery doc § 5 extension)
- `b10e998` arc 157 slice 1a-i: def form end-to-end (substrate +
  runtime binding via two coordinated sonnet sweeps)
- `0a9d0a3` arc 157 slice 1a-ii BRIEF + EXPECTATIONS
- `28afc5b` arc 157 slice 1a-ii: redef config + gating + type-
  stability
- `ecd219a` arc 155 test cleanup (dead `startup_err`; pristine
  warnings)
- (this commit) arc 157 closure paperwork

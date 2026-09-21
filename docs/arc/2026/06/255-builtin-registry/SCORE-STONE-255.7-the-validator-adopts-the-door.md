# SCORE — STONE 255.7: the rete validator adopts the door

Branch: `main`. **Committed, not pushed.** Drawn against `f9914bf70` (draw `a828a5b85`).
Parent: `BRIEF-STONE-255.7-the-validator-adopts-the-door.md`.
Floor / workspace clippy / census: orchestrator. Crate clippy + lint suite run here.
**No corpus `.wat` converted.** New fixtures only. 8d-ii not started.

## Gate: the 179-file delta — ONE tree

Same spread: every 12th of the 8d-i census `files.txt` (**179**).
Originals: **live tree**. Converted: `/tmp/255-7/conv` — **re-converted with the CURRENT
codemod.** Timed one file first: pprintln copy **0.31 s**; batch 179 **30.81 s**.

| | this tree at 255.6 | this stone |
|---|---|---|
| originals clean | **161 / 179** | **161 / 179** |
| still clean after conversion | 100 | **112** |
| ⛔ **REGRESSIONS** | **61** | **49** |

**12 closed. 0 newly_broken. Net −12.**

### Classification (49)

| cause | n | vs this tree at 255.6 (61) |
|---|---|---|
| `UnresolvedReference` | **23** | same |
| `ReteCheckErrors` | **0** | **16 → 0** |
| `CheckErrors` | 11 | 7 → 11 *(four left the rete wall and hit type-check)* |
| `MalformedDecl` | 9 | |
| `ProgramBodyEvalFailed` / OTHER | 6 | |

⭐ **`ReteCheckErrors` as a regression class is gone.** Twelve of the sixteen freeze clean.
Four now fail as `TypeMismatch` (`foldl` PersistentVector; `vrm/ins` expects Record) — the
**checker**, not the rete validator. Named, not forced.

## How the set was derived (not the brief's 14-line grep)

Walked `src/rete/validate/` for every `WatAST::Keyword` **and** every rust-scheme
string compare / `rete_op_for` / `quote_boundary` / `type_env_name` consult. The
brief's grep missed `typing.rs`'s rete-op list head (`rete_op_for` on a Keyword)
and treated `:135` as "a Keyword match" when it is a **literal-name** via
`quote_boundary`.

### Class A — type extraction → `canonical_identity`

| site | what |
|---|---|
| `validate_then_form` fact head | `(weather/ColdAndWindy …)` is the same type as `:weather::ColdAndWindy` |
| `then_operand_declared_type` list head + kwargs type slot | nested constructors |
| `walk_nested_constructors` type slot | aggregate / enum-variant at `type_idx` |
| `typing.rs` `resolve_operand_type` call head | `wat.rete.i64/+` hits the same `RETE_OPS` row |

`type_env_name` was a **second** slash-to-`::` parser. Replaced with
`canonical_identity` (trim colon). ⭐ **Thirteenth correction:** the brief's grep
could not see a helper that already reconstructed identity by another door.

### Class B — literal-name → canonicalize both sides

| site | literal |
|---|---|
| `walk_for_make_rule` / `walk_for_make_query` | `:wat::rete::make-rule` / `make-query` |
| `rete_walk_skips_children` | `quote_boundary` after identity |
| `walk_nested_constructors` `match` | `resolve_core_name` after identity |
| `kwargs_construct_head` | `:wat::core::kwargs-construct` |
| test helper `find_make_rule` | same as the walk |

### Class C — legitimate Keyword-only (unchanged)

| site | why |
|---|---|
| kwargs field keys (`:1084` / `:1151` / `:1336`) | converter preserves `:field`; `rete_is_kwargs` is the kwargs shape |
| `check_field_kw` | takes the field-naming **node** for the caret |
| `check_operand_field_ref` / `is_non_field_keyword` / field arm of `resolve_operand_type` | field refs are keywords |
| `collect_var_occurrences` Keyword arm | scalars have no `?var`; Symbol already handled |
| `check_match_pattern_for_shadow` | already lists Symbol **and** Keyword |
| `describe_operand` | already has a Symbol arm |

A widening of Class C would admit a Symbol where a Symbol is a **value**, not a
field name. Asserted by the unknown-field fixture (`:nope` stays `UnknownField`).

## Non-vacuity both ways

- `(weather/ColdAndWindy :location ?loc)` as a `:then` insert **freezes**.
- Converted `:then` + unknown field `:nope` is still **`UnknownField`** (and
  `RhsMissingFields`) — **diagnosed, not skipped.**
- `:then [42]` is still **`MalformedClause`**.
- `(?k <- :k)` is still **`MalformedClause`**.
- Live rust-scheme northstar still `--check` rc=0.

Stone B's skip for an **unknown** constructor head is unchanged and now spelling-
independent (Symbol unknown joins Keyword unknown). The errors that fire **after**
a known type still fire.

## What the door cannot express

- **`eval_insert::build_insert_fact`** still demands a Keyword fact-form head at
  **fire** time. Freeze (`--check`) does not fire. 8d-ii load/run will. Not this
  stone (not the validator; not a `.wat` we convert).
- **`quote_boundary` itself** still matches rust-scheme literals. The rete walk
  canonicalizes **before** calling it. Resolve's own consult is out of scope.
- The four `TypeMismatch` files above — `foldl` collection param; Record vs
  user type. Registry/checker residue, listed out of scope.

## Test count

Predicted **+4**. `cargo nextest list --release -p wat`: **5335** (was 5331).

## Walls I ran

- crate clippy `-p wat --all-targets --release -D warnings` — **0**
- `one_param_spec` / `rete_bind_generators` / `one_variant_separator` — pass
- `rete::validate` lib tests — 14 passed
- `probe_arc255_7_the_validator_adopts_the_door` (accept + unknown-field + non-list + `<-`) — pass

Floor / workspace clippy / census: orchestrator. Do not push. Do not start 8d-ii.

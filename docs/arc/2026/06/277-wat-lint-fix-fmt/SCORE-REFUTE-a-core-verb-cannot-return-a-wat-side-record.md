# SCORE — REFUTE: a core verb cannot return a wat-side record

No commit. The two floor reds were not re-run as the original failure; after the fix they pass.

## The type

`:wat::core::read-string-with-comments` now returns `:wat::core::ReadWithCommentsOutcome`, registered in `TypeEnv` beside `ReadOutcome`:

```
Forms     [forms comments]   comments = (PersistentVector :- [:wat::fmt::Comment])
Malformed [cause]
```

TypeScheme in `register_builtins` next to `read-string`. `check_env.get` answers. The debt ledger is unchanged.

`@see :wat::fmt::emit` is gone — that is a wat `defn`, not an intrinsic. `@see :wat::core::read-string` remains.

`wat/fmt.wat` no longer defines `Parsed`. It matches the core enum. Comment records stay wat-side.

## The two arms, after

```
intrinsic::tests::checker_skip_debt_is_named_and_frozen          ok
intrinsic::tests::all_see_fqdns_resolve_to_registered_intrinsics ok
intrinsic::tests::doc_arg_ret_types_match_checker_scheme         ok
```

## R11 is a default, not a hand-list

`Claim {form}` is asserted by R1 (`defn-claim` in `defn.wat`). R11 fires under `(:wat::rete::not (:wat::fmt::Claim (?p <- :form)))`. The `not= ?hn ":wat::core::defn"` line is gone.

A later `let`/`match` rule asserts its own Claim; R11 is not edited.

## Substance still holds

R1 multi-arg and empty `[]` unchanged. Half-broken match: R1 leaves arms on one line; R11 (new file only) breaks every child. `wat/io.wat`: FORMS=3 COMMENTS=28 IDEMPOTENT=true.

## Commands

| command | result |
|---|---|
| the two floor arms + doc/scheme | 3 passed |
| R1 / R11 / io.wat drivers | layout + counts as before |
| `every_wat_scripts_file_loads` | **1 passed** |

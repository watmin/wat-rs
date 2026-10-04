# WEIGH — STONE 255.90: measure — the name is the pair — ACCEPTED, with three corrections

**Executor: grok via pulsare.** Census `35cf2d158`, re-applied on `main` as `61d54d1bf` after the builder's ruling B
(2026-10-04): 255.89 is parked on `origin/cutover-5c-iii` at `ddb1261b5` and `main` returns to green ground at
`839e8fbbf`. The census ran on the 255.89 tree; its `src/` counts include 255.89's cures (the door in `wat-reader`,
`fold_member_twin`), which are not on `main`. Weighed by the orchestrator on 2026-10-04.

## What it settles

- **The cost has three addresses** (one fuzz deftest: 90.6M door calls, 157.6M `as_str`, 97.1M allocations):
  `canonical_type_key` (`src/types.rs:374`, 71.1M), `env_key` borrowing `flat` (`src/scope/resolution.rs:81`, 91.6M),
  and `ident.as_str() == "nil"` (`src/runtime.rs:1785`, 54.5M), a string compare deciding a symbol.
- **Every door result is identity:** 247 of 247 calls; 273 of 276 `as_str`/`leaf`/`path` calls. `flat` serves string
  identity, which the ruling deletes.
- **The literals by role:** REG 1954, DISPATCH 683, CMP 2286, BUILD 398, WAT 241, MSG 1417; OTHER 15,135 (8,371 doc
  comments; 3,158 + 130 + 2,610 unclassified residue).
- **Two parsers disagree today:** `:a::b/c` reads as `{":a::b", c}`, `a.b/c` as `{a.b, c}`.
- **Equality reliance:** `substitute` (`src/runtime.rs:14217`) needs a binder and its body reference equal; under
  pair-equality it must resolve instead of compare.

## Corrections

1. **`syn` moved into the shipped crate's `[dependencies]`** so the census could be a bin. A measuring tool does not
   change what every `wat` build links. It moves to `examples/` (examples link dev-dependencies), and `syn` returns to
   `[dev-dependencies]`. Owed by 255.91.
2. **The map classifier mostly failed:** 355 of 427 rows `STOP`, and it filed the four core registries as `OTHER`
   because their key parameter is named `path`. **Ruled by the orchestrator:** `SymbolTable.functions`,
   `unit_variants`, `runtime_def_values` and `UseDeclarations.declared` are name registries. The remaining maps are
   found by the compiler when the registry key type changes, not by a second classifier.
3. **The interner:** the ruling's "interned" meets `docs/ZERO-MUTEX.md`, and a global interner behind a lock is the
   situation that document refuses. `Name` is two `Arc<str>` with equality and hashing over their contents (a
   pointer-equality fast path is allowed); interning, if a measurement ever asks for it, is a frozen-world table, not a
   lock.

## Verdict

Accepted as a measurement. Next: 255.91, the `Name` type and pair equality, on green `main`.

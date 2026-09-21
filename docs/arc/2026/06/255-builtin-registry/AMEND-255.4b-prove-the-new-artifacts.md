# AMEND — STONE 255.4b: 28 → 3. Three walls asking the new artifacts to prove themselves.

**Orchestrator's verification row, `82575be23` (not pushed).**

## ⭐ THE FOLD WORKED — 28 → 3, and the diagnosis was better than the amend's guess

```
     Summary [ 287.864s] 5939 tests run: 5936 passed (8 slow), 3 failed, 22 skipped
```

The amend guessed the third join was *"a `push_str`, a `join`, an identity step, or the surface-member
resolution."* ⭐ **It was neither — it was TWO CONSUMERS of the emitter you had already fixed:**

1. **`UseDeclarations::covers`** — a `use!` covered a head only when the rest began `::`, so a live
   `:rust::test::MathUtils/add` was *"not covered by any `use!`"*. **Coverage is prefix + member
   join**, and the join moved. 25 of the 28.
2. **`check.rs`'s surface-method arm** — `UnknownCallee` immediately where the runtime already fell
   through to `sym.get`. A non-member `/name` under a surface type is an ordinary function.

⭐ **And you found two more EMITTERS the brief never named** — `format!("{}::surface-forms")` in
`types.rs` and `"{proto-base}::surface-forms"` in `wat/service.wat`. Same class as `method_wat_path`.

⭐ **`.wat.bad` — the glob gap.** `git ls-files '*.wat'` misses `*.wat.bad`. ⚠ **That is the same
defect that cost this campaign 33 of 64 stdlib files** when the orchestrator wrote
`git ls-files 'wat/**/*.wat'`. **Third glob miss in this arc; you caught this one.**

## ⛔ THE 3 REDS — every one is a wall asking a NEW artifact to prove itself

### 1. `every_discovering_gate_declares_how_it_knows_it_reached_something`

> *"1 gate(s) in `tests/lint/` DISCOVER their subject set and never say how they know it was not
> empty. A walk that comes back empty asserts nothing over nothing and reports PASS."*

Your `one_member_join.rs` now derives its corpus from `git ls-files` — **correct, and it created
this** — but it does not declare a floor. ⛔ **This is the SAME hazard the amend flagged about
`bootstrap/`, arriving from the other side:** a gate that reads nothing passes.

**Drive it and read the real number first.** A floor picked by guess is worse than none.

### 2. `every_recorded_migration_is_fixtured_or_runed`

```
recorded-migration coverage failed (1):
  type-member-colon-to-slash: no fixture, no rune:replay(unreadable-preimage)
```

⭐ **The builder's merge constraint is the reason this wall exists.** A codemod that is merge
infrastructure must be **provable on a pre-image**, or the next branch merge cannot trust it.
**Give it a fixture.**

### 3. ⛔ `retirement_table_is_fully_reachable` — **17 rows fail to TEACH**

> *"17 row(s) are diagnosed only at RUNTIME — door 1 (`check.rs`'s retirement consult) is not firing
> for them, so the error arrives after the program starts instead of before"*
>
> `:wat::kernel::HandlePool::new` → `UnresolvedReferences` at startup

⛔⛔ **This is the brief's own requirement, unmet:** *"A RETIREMENT LEDGER ENTRY for each… **an old
spelling must TEACH its replacement, not merely fail.**"* Today an author writing the old
`HandlePool::new` gets a generic `UnresolvedReference` **after startup** instead of
*"retired; use `HandlePool/new`"* **at check time.**

⚠ **The rows exist; door 1 does not consult them for this shape.** The ledger is there and silent —
which is worse than absent, because it looks done. **Make door 1 fire, or report why it cannot.**

## The fold

⛔ **Fold into `82575be23`.** Not a repair commit.

## Verified good, do not redo

| row | result |
|---|---|
| clippy workspace | ✅ **0** |
| delta | ✅ **held at 81** — nothing regressed while 28 reds closed |
| `Type::member` call heads in `.wat` | ✅ **zero** |
| the 2 `Bytes/to-hex` regressions | ✅ still closed |
| the final derived set | ✅ stated from the registries and both consumers, not grep |

## After the fold

State crate clippy + the three named tests. I re-run floor, workspace clippy, census and the delta.
**Do not push. Do not start 255.5 (the position grammar).**

# SCORE — STONE N RELAND 8: the rename corrupted Option's own declaration

No commit. Floor not re-run. Lands on DRAW N RELAND-8 (`617a444bf`).

Builder-ruled order held: **guard → repair `:None []` → revert keyfn → re-measure**.

## 1. Guard — position rule, not a name list (STOP-3)

`wat/fix.wat`: `binder-marker?`, `enum-purity-marker?`, `defenum-variant-start`,
`rename-exact-edits-defenum-variants`. `rename-exact-edits` special-cases a `defenum`
list: prefix (head / type name / `:-` / params / purity / optional meta) is walked
normally; the variant body is walked as **name-slot / field-vector pairs**. A
variant-name keyword is not a use site — it is not renamed, whether followed by
`[]` or bare. Field vectors still recurse (a `:None` *inside* a field type would
still move).

`wat-scripts/fixes/positional-ctor-to-map.wat`: same door. `walk-edits` does not
`ctor-edits` a `defenum` list; `walk-defenum-variants` skips the name, walks the
field vector.

Landmine while wiring the prefix walk: `take`/`drop` return Stream, not Vector.
`rename-exact-edits` / wrap now `(:wat::core::into [] (take/drop …))`. Without that
the guard itself CheckErrors at `fix.wat:953`.

Measured on a `/tmp` copy of the **repaired** `wat/core.wat`:

| pass | EXIT | `diff -q` vs repaired source |
|---|---|---|
| `bare-variant-to-qualified.wat` | 0 | **identical** — `:None []` survived |
| `positional-ctor-to-map.wat` | 0 | **identical** — variant-name slot not wrapped |

`--check` wrap + rename scripts: **EXIT=0**.

## 2. Repair — `wat/core.wat:2127` is `:None []`

HEAD (`617a444bf`) still has the corruption:

```
(:wat::core::defenum :wat::core::Option :- [T] :wat::enum::Pure
  :Some [value <- :T]
  :wat::core::Option::None)
```

Now:

```
(:wat::core::defenum :wat::core::Option :- [T] :wat::enum::Pure
  :Some [value <- :T]
  :None [])
```

`type-of` (the instrument, not a grep):

```
:variants [ { :name :Some :fields [ {:name :value :type T} ] }
            { :name :None :fields [] } ]
```

Result unchanged (`:Ok` / `:Err`). `include_str` — binary rebuilt after the edit.

### The synthesized ctor wall (necessary companion, not a wall-weaken)

`:None []` made register.rs emit `(:wat::core::variant :Option :None)` as the unit
ctor body. `infer_list` treated `:None` as a **value**, so `retired_bare_variant`
fired: freeze `malformed :None form`. First attempt (reverted): emit the FQDN as
the variant tag. `eval_variant` wants the **short** name — runtime then said
`enum Status has no variant named …::Started`. Right fix: `src/check.rs` infer_list
arm for `":wat::core::variant"` — args[0]/args[1] are NAME LITERALS; infer only
field args[2..]. `register.rs` still `format!(":{}", variant_name)`. The wall on
expression-position `:None` is unchanged.

## 3. Revert the keyfn workaround — it was compensating (STOP-2)

`wat/query/mem.wat` was never committed with the explicit-fn workaround (RELAND 7
not committed). Working tree is the keyword form:

```
(:wat::core::sort-by :wat::query::Row/sk matches)
(:wat::core::sort-by :wat::query::IndexRow/isk matches)
```

Probe `wat-scripts/scratch-pad/probe-reland8-sort-by-row-sk.wat` (empty Row vector):

```
"[]"
EXIT=0
```

Before the declaration repair this is the purity refuse that killed the store.
Arc 255 A-2-ii-b-0's property is restored. **Not a remaining classifier gap at
this call site.** The workaround is not kept.

## 4. STOP-4 — type-of every stdlib enum the campaign could have poisoned

`wat-scripts/scratch-pad/probe-reland8-type-of-campaign-enums.wat`. 58 enums, 214
variants. **0** variant `:name`s are FQDN-corrupted (no `/`, no `::` in the
short name). Option was the only live instance of the class — Result escaped
because `:Ok`/`:Err` carry field vectors the rename's pattern did not match.

Spot-check (short names, unit variants report `:fields []`):

| enum | variants |
|---|---|
| Option | Some, **None** |
| Result | Ok, Err |
| RecvOutcome | Message, Closed, Stopped, Lost |
| BreakKind | Block, Align |
| NodeKind | IntLit … Map (all short) |
| WalkStep | Continue, Skip |
| Numeric | I64, F64 |

## 5. Re-measure the 34

The `-11` cluster is **gone**. Journal query returns 21. Keyword keyfn.

| cluster | after guard+repair+revert |
|---|---|
| journal query / logs / metrics / span / tagged_keys / mem_store / backend / log_captures (the `-11` / Lost-on-scan set) | **PASS** |
| rete smem_roundtrip + sqlite_store_differential | **11 passed** |
| `probe_arc296_enum_map_ctor` | **5 passed** |
| sift_rules ×2 + sift_rules_arena ×2 (counts-exact) | still `disconnected` — rete-sift path, **not** mem-store scan. Named remainder. Fail-closed unknown-message arms PASS. |
| call_context ×5 | freeze: positional `Op::-Mark`. Not a wrong answer. |
| journal_surface `wrong_response_type` | compile-error fixture; positional-ctor wall + `:wat::core::None` wall fire **before** the intended TypeMismatch WriteMetrics vs PutResponse. STOP-5: **not recaptured**. |
| rs1 `IncrementResponse::Ok {:value}` | wrap named the field `:value`; declared is `:count`. Wrap field-name, not scan. |

STOP-5 held: no assertion updated to accept a wrong value. `journal_surface` stays red.

## 6. The rest of the RELAND-6 45 — named, not closed

`.wat.bad` try/zombie (5): PASS on this tree because RELAND 7's incidental Ok/Err wrap
+ col `20→36` goldens are still uncommitted leftover. This strike did not recapture
them and did not recapture `journal_surface`.

| test | arm |
|---|---|
| hashmap p6 | golden `1 type-check error` vs **4**. Intended TypeMismatch still fires; extra 3 are incidental bare `:Some`/`:None` match arms in the `.wat.bad` + fallout non-exhaustive. Fixture stays wrong. |
| diagnostics c3 | freeze: positional `:demo::Op::Go`. |
| lint `every_wat_scripts_file_loads` | **2 of 694** do not load: `wat-scripts/fixes/positional-to-kwargs.wat:27` positional `(:wat::core::Option::None :wat::WatAST)` (skip-path for wrap; loader still type-checks); `wat-scripts/probes/arc-170/probe-m1-phantom-d.wat:38` wrap field-name `:s` vs declared `:pair`. |
| reflection metadata_of | golden mismatch on `#wat.doc/Row` (examples still carry retired spellings). |
| mcp counter ×2 | positional `GetResponse::Ok` / `IncrementResponse::Ok` / `Outcome::Reply` in the `<mcp>` source. Same class as call_context. |

## STOP rows

| STOP | result |
|---|---|
| STOP-1 declaration repaired before the guard | **held.** Guard in `fix.wat` + wrap first; repair is one token after the skip is live. Re-running rename/wrap on the repaired file is a 0-byte no-op. |
| STOP-2 keyfn workaround kept without measuring | **held.** Restored (was never committed). `sort-by Row/sk` EXIT=0 `"[]"`. Compensating. |
| STOP-3 guard is a name blacklist | **held.** Position rule: inside a `defenum`, the variant-name slot is not a use site. No `:None`/`:Some` list. |
| STOP-4 other declarations assumed clean | **held.** 58 enums / 214 variants type-of'd. 0 FQDN-as-variant-name besides the Option we repaired. |
| STOP-5 expectation updated to match a wrong value | **held.** journal_surface, hashmap p6, mcp, call_context stay red. |

## Working tree (this strike)

```
wat/fix.wat                                         guard (rename)
wat-scripts/fixes/positional-ctor-to-map.wat        guard (wrap)
wat/core.wat                                        :None []
src/check.rs                                        :wat::core::variant name literals
wat-scripts/scratch-pad/probe-reland8-*.wat         measurement
```

`wat/query/mem.wat`: no diff (keyword keyfn is HEAD). RELAND 7 leftover `.wat.bad`
col-shift goldens still sit uncommitted; not this stone.

Do not commit unless a later brief says to.

# WEIGH — STONE 255.57 (and 255.56's finish): `nil` is a proper type; the operators ask the classes — ACCEPTED

**Executor: grok via pulsare, commits `782f5fcda` (255.57, K1) and `4bcf0dfbd` (255.56 finish, W1 + housekeeping), on
top of `6225697a9` (255.56's switch).** Weighed by the orchestrator on 2026-09-27. All four are pushed together, now
that the floor is green.

## Rulings built

- **K1** (builder: *"rip the bandaid off … finally kill it"*): `:wat::core::nil` is a builtin leaf path, not an alias of
  `Tuple([])`. `Value::Unit` → `Value::Nil`. A tuple type needs at least one slot. This delivers the earlier
  `nil != ()` ruling the old scratch comment cited.
- **W1** (builder: *"record is the holder for data and only data; struct is only for things who must hold onto
  resources and non-data fields"*): `:test::Wrapper` is a `defrecord`.
- **255.56:** `<`/`=` ask `wat/class.wat`'s classes; Refuse; E-a; Z1; `is_type_orderable`/`is_type_equatable` deleted
  (ledger 198 → 195); bounds on `assert-eq`/`dedupe-walk`/`dedupe`; doctest through `matches?`.

## Re-run by the orchestrator

| row | result |
|---|---|
| release floor | **6193 passed / 23 skipped** at `4bcf0dfbd` |
| `grep 'Tuple(vec!\[\])\|Tuple(Vec::new())' src` | 0 |
| `grep 'Value::Unit' src tests` | 0 |
| `grep 'is_type_orderable\|is_type_equatable' src` outside comments | 0 |
| `wat-tests/edn/roundtrip.wat` | `defstruct` → `defrecord`, one line |
| census / delta / clippy | grok's: three flips only, the expected contract goldens (0→1); NEW 2 / RECOVERY 0 same files; clippy rc 0. Not re-run |

The first floor went red on seven `nil` spellings (`:()` in goldens and tests; `wat.type/nil` as a separate path; an
empty `Tuple :- []` still built). They were kept, named and fixed, not re-run away.

## Carried

- The three `fix_source_local_rules__contract-0{6a,6b,7}` goldens show census rc 1 until the goldens-as-data stone.
- `:()` survives only as the retired spelling a refusal names (`BareLegacyUnitType`, a parse-refusal test) and two
  unrelated empty-list messages (`"()"`).

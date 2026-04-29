# Arc 086 — EDN round-trip + natural lossy formats

**Status:** SHIPPED 2026-04-29.

**Predecessors:**
- Arc 079 — `:wat::edn::*` shims (write surface). Slice 1 left struct
  field names rendering as positional `:field-N` keys; round-trip
  not yet shipped at the wat-Value layer.
- Arc 048 — user-defined enum value support. Tags carry variant
  identity; needed for read-side reconstruction.
- Arc 085 — `SymbolTable.types` capability carrier. The type
  registry that read + named-field render both consult.

**Surfaced by:** the user's iteration on the `ConsoleLogger` UX
(arc 087, same session). Three consecutive observations forced
the substrate work:

> "{:field-0 ... :field-1 ...}" — that's not what I want, fields
> should have real names

> "wait... the wat-edn side can't consume its own edn to reconstruct
> its own items?..."

> "we should show {:time ... :level ... :caller ... :data ...} —
> i don't know why we need this leading tag"

> "{:_type :Buy ...} this isn't honest... Buy isn't a fqdn...
> demo.Event/Buy should be the _type"

Each comment surfaced a distinct gap. The arc bundles all four —
named-field render, read-side round-trip, tagless natural EDN,
ingestion-friendly natural JSON with FQDN discriminator — because
they're load-bearing for one another.

---

## What shipped

### `value_to_edn_with(v, types)` — named-field struct render

`src/edn_shim.rs`:
- New `value_to_edn_with(v, Option<&TypeEnv>)` walker. When the
  registry contains the struct's `StructDef`, fields render as a
  Map keyed by declared field names. Fallback to positional
  `:field-N` keys when the registry isn't reachable (older
  test harnesses).
- `value_to_edn(v)` becomes a back-compat thin shim that calls
  `value_to_edn_with(v, None)`.
- The three existing eval entry points (`eval_edn_write` /
  `_pretty` / `_json`) now thread `sym.types()` through.

Output before:
```
#wat.std.telemetry/LogLine {:field-0 #inst "..." :field-1 :info :field-2 :market.observer :field-3 #demo.Event/Buy [...]}
```
Output after:
```
#wat.std.telemetry/LogLine {:time #inst "..." :level :info :caller :market.observer :data #demo.Event/Buy [...]}
```

### `:wat::edn::read s` — round-trip

`src/edn_shim.rs`:
- New `eval_edn_read(args, env, sym)` entry point.
- New `edn_to_value(edn, types)` walker — the inverse of
  `value_to_edn_with`. Maps `OwnedValue` variants back to wat
  `Value`s.
- New `tagged_to_value(tag, body, types)` — body shape disambiguates:
  Map → struct lookup, Vector → enum tagged variant, Nil → enum
  unit variant.
- New `EdnReadError` enum with descriptive variants (UnknownTag,
  UnsupportedTag, NoTypeRegistry, UnknownStructField,
  EnumVariantNotFound, Other).

`src/runtime.rs`:
- Op-dispatch table extended.

`src/check.rs`:
- Polymorphic type scheme `:fn(String) -> :T` (fresh-var return).

Tag dispatch:
- `#wat.std.telemetry/LogLine {:time ...}` → look up
  `:wat::std::telemetry::LogLine` as Struct; reconstruct
  `Value::Struct` with declared field names + recursively walked
  field values.
- `#demo.Event/Buy [100.5 7]` → look up `:demo::Event` as Enum;
  find variant `Buy`; reconstruct `Value::Enum` with positional
  fields.
- `#demo.Event/None` (nil body) → unit variant lookup.
- `#inst "..."` → `Value::Instant` (handled by parser).
- `#wat-edn.opaque/...` / `#wat-edn.holon/...` → unsupported (no
  reconstruction path; opaques can't round-trip).

7 round-trip deftests at `wat-tests/edn/roundtrip.wat` covering
primitives, Vec, enum tagged variants, structs with named fields,
and nested (struct holding an enum). All green.

### `:wat::edn::write-notag` — tagless EDN

`src/edn_shim.rs`:
- New `eval_edn_write_notag` entry point.
- New `value_to_edn_notag(v, types)` walker. Drops the `#tag`
  wrapper from struct + enum-variant renders. Struct → flat Map.
  Enum tagged variant → Map with `:_type` discriminator + named
  fields. Enum unit variant → bare keyword.
- Discriminator shape: `_type` → namespaced keyword
  `:<dotted-namespace>/<Variant>` (e.g. `:demo.Event/Buy`).
  Fully-qualified per the user's correction: bare variant names
  collide across enums; the FQDN is the honest identity.

Output:
```
{:time #inst "..." :level :info :caller :market.observer :data {:_type :demo.Event/Buy :price 100.5 :qty 7}}
```

Lossy. EDN tagless renders cannot be `read` back into the original
wat values (no tags ⇒ no struct/enum reconstruction signal). For
round-trip use the tagged form.

### `:wat::edn::write-json-natural` — ELK/DataDog-friendly JSON

`src/edn_shim.rs`:
- New `eval_edn_write_json_natural` entry point.
- New `value_to_json_natural(v, types)` walker.
  - Tags dropped (same as notag).
  - Keywords → plain strings (no `:` prefix, `::` → `.` for
    namespace clarity).
  - `Inst` → bare ISO-8601 string with millisecond precision.
  - Enum tagged variant → Map with string `"_type"` discriminator
    + named-field keys + values; `_type` value is FQDN
    `"demo.Event/Buy"`.
  - Enum unit variant → bare string `"demo.Event/Variant"`.

Output:
```json
{"caller":"market.observer","data":{"_type":"demo.Event/Buy","price":100.5,"qty":7},"level":"info","time":"2026-04-29T..."}
```

Same FQDN discriminator as the EDN form. Suitable for streaming
into ingestion tooling (ELK / DataDog / CloudWatch Logs) that
expects naturally-shaped JSON without sentinel-tagged objects.

Lossy. Use `:wat::edn::write-json` for round-trip-safe JSON.

---

## Slice plan

Single session. Shipped iteratively as the user surfaced each gap:

1. Field-name rendering (named fields via `sym.types`).
2. `:wat::edn::read` + 7 deftests verifying round-trip.
3. `:wat::edn::write-notag` + tagless walker.
4. `:wat::edn::write-json-natural` + naturalized walker.
5. FQDN discriminator correction (`Buy` → `demo.Event/Buy`).

Each step verified end-to-end via `cargo run -p console-demo`.

---

## Open follow-ups

- **HolonAST round-trip.** Read-side returns
  `UnsupportedTag(wat-edn.holon/...)` for HolonAST values today.
  Future arc when a consumer wants to rehydrate stored HolonASTs
  from EDN logs.
- **Type-coercion at read boundary.** Today's read walks fields
  positionally and trusts the EDN matched a `write` of a value
  with the expected schema. Mismatched shapes (wrong type on a
  field) get caught downstream when accessors run; no early
  coercion at read time. Refinement when a consumer wants
  early-fail diagnostics.
- **Pretty-print variant of natural formats.** `:NoTagEdn` and
  `:NoTagJson` are compact single-line. A multi-line indented
  natural form would need its own writer pass; substrate
  doesn't ship it yet.
- **Discriminator key configurability.** `_type` is hardcoded.
  Some consumers prefer `@type` / `kind` / `variant`. A render
  option to pick the discriminator key is a future arc when a
  real consumer surfaces a need.

---

## Test strategy

- 7 round-trip deftests at `wat-tests/edn/roundtrip.wat` covering
  primitives, Vec, enum tagged variants, structs with named
  fields, and nested (struct holding an enum).
- `examples/console-demo/` exercises all 5 formats end-to-end
  (Edn / NoTagEdn / Json / NoTagJson / Pretty) with both stdout
  and stderr level routing.
- Workspace-wide regression: 728 substrate Rust tests + every
  wat-suite green; zero failures.

---

## Dependencies

**Upstream:** Arc 085 (`SymbolTable.types`) — without the type
registry, named-field render and tagged-value reconstruction
both fall back to positional/erroring shapes.

**Downstream this arc unblocks:**
- Arc 087 — `ConsoleLogger` ships with all 5 formats reachable.
- Future cross-process IPC via EDN strings (forks passing typed
  values through pipe-serialized EDN).
- Replay tooling — emit wat values as EDN, replay through
  different consumer for testing.
- Production telemetry into ELK / DataDog / CloudWatch Logs via
  `:NoTagJson` format.

PERSEVERARE.

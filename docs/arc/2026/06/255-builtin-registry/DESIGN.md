# Arc 255 — Rust builtins as first-class, registered, reflectable entities

**The defect (an asymmetry, which `feedback_asymmetries_meet_high_bar` says is a
defect, not a quirk).** There are two classes of callable in wat, and they are not
equal citizens:

| | registered in | resolve asks | reflectable | carries metadata |
|---|---|---|---|---|
| **user forms** (`defn`, `defclause`, `defenum`…) | `sym` (SymbolTable) | `sym.get(name)` | yes (`metadata-of`, arc 241.7) | yes (def-metadata-map, arc 241.6) |
| **Rust builtins** (`i64::+`, `length`, `send`…) | **nowhere** — a 454-arm compile-time `match` | **(can't)** → reserved-prefix blanket-accept | **no** | **no** |

Builtins are the one callable-kind that is *opaque*: not registered, not queryable,
not reflectable, no metadata. That asymmetry is the direct cause of the
undefined-func class (`+'2`, `make-*-queue`, `Bogus`): with no registry to ask "is
this defined?", `resolve` falls back to `if is_reserved_prefix(head) { return true }`
— blanket-accepting any leaf, deferring wrong names to a runtime "unknown function"
(the 30-minute wat-lru crawl). The builtins predate the reflection model and were
never re-lifted to it. Same debt shape as everything else this session: a layer
that worked, never migrated to the doctrine that grew up after it.

**This arc eliminates the asymmetry: builtins become first-class registered
entities, queryable and reflectable through the *same* path as user forms.** The
undefined-func class dies as a *side effect* of fixing the real defect.

## The registry

A `BuiltinRegistry` — `name → BuiltinEntry` — populated once, queried uniformly:
- **resolve**: membership → "is this defined?" (the `+'2` bug, gone). The
  reserved-prefix blanket-accept hack is **deleted**; builtins resolve through the
  same path as user forms (registry/`sym` membership). One resolution path for
  everything.
- **check**: `entry.arity` (and later `entry.type_sig`) → arity/type validation
  at check time.
- **reflection**: `(:wat::core::metadata-of :wat::core::i64::+)` answers for a
  builtin exactly as it does for a user form.
- **errors-as-curriculum**: the registry enables near-match suggestions
  ("unknown `:wat::core::i64::+'2` — did you mean `:wat::core::i64::+`?").
- **runtime**: dispatches via the registry (or a generated fast path — see Perf).

## The entry-shape (DAY ONE) — *this is what we shape together before code*

Proposed starting shape, deliberately extensible (the metadata map grows later):

```rust
struct BuiltinEntry {
    name:    &'static str,          // FQDN keyword, e.g. ":wat::core::i64::+"
    handler: BuiltinHandler,        // the dispatch (see Mechanism)
    arity:   Arity,                 // Exact(n) | AtLeast(n) | Range(lo,hi) | Variadic
    meta:    BuiltinMeta,           // EXTENSIBLE — day-one: { category, doc }
}
```

**Open shaping questions (decide before code moves):**
1. **Align `BuiltinMeta` with the existing user-form metadata** (def-metadata-map,
   arc 241.6 / `metadata-of`, arc 241.7) so `metadata-of` is *uniform* across
   builtin + user. Do we reuse that exact shape, or a superset? (Builtins add
   `arity` + `handler`; do user forms already carry arity we can mirror?)
2. **Day-one metadata fields.** Minimum to fix the bug + enable reflection:
   `arity` + `category` + `doc`. Is `type_sig` day-one (big — the checker's
   per-builtin inference lives in `infer_list`), or a later metadata extension?
   Recommendation: defer `type_sig`; ship `arity`/`category`/`doc` first, grow in.
3. **`Arity` granularity** — is `Exact|AtLeast|Range|Variadic` enough, or do some
   builtins need richer (e.g. keyword-arg shapes)?

## Mechanism — one source, hot path preserved

The registry is the **single source of truth**; the runtime fast path is
*generated from it* so we keep both first-class-ness and speed:
- A declaration site lists each builtin once: `name => handler, arity, meta`.
- From that one list: (a) the registry (`phf::Map` — compile-time perfect hash →
  near-`match` lookup speed, carries the metadata, used by resolve/check/
  reflection); and, if benchmarks demand for the hot arithmetic path, (b) a
  generated dispatch `match`. Both derive from the same declaration → no second
  copy, no drift, no asymmetry. (Supersedes 254.R's hand-maintained `matches!`
  mirror — that was two-copies-with-a-gate; this is one source.)

## Perf (the one real risk)

`i64::+` runs millions of times in the trading substrate; a naive `HashMap` lookup
per op would regress the hot path. Mitigation: `phf` (perfect hash, compile-time,
no collisions) for near-`match` speed, and/or keep a generated `match` for the hot
arithmetic ops — both emitted from the one source. **Gate the arc on a benchmark:
no regression on the arithmetic hot path.**

## Migration

454 dispatch arms (`dispatch_keyword_head` + `dispatch_keyword_head_value`, plus
the prefix-guards like `:wat::config::set-*`) become registry declarations. The
arm *bodies* are unchanged (handlers); the outer shell becomes the declaration.
Large but mechanical. Substrate-as-teacher drives completeness: any builtin
missing from the registry → resolve rejects real corpus code → add it.

## Gates (reuse the 254.R probe)

- `tests/nursery/probe_undefined_builtin_resolves.rs` (already committed,
  RED-verified): `+'2` and `Bogus` become resolve-time errors; the real `+` stays.
- reflection: `metadata-of` answers for a builtin (new probe).
- arity: a wrong-arity builtin call is a check-time error (new probe).
- lib green; full corpus green (no over-rejection beyond the pre-existing 5
  `sqlite_Db`); **benchmark: no hot-path regression.**

## Relationship to 254.R

254.R correctly *named the class* and shipped the *probe* (the gate). Its
*mechanism* (a hand-transcribed `matches!`) was a half-measure — two copies, drift
caught not prevented, builtins still opaque. Arc 255 supersedes that mechanism
with the registry (one source, builtins first-class). The 254.R probe is arc 255's
gate. No code from 254.R's hand-list shipped (reverted).

---

## REFRAME (builder ask: "parity with user forms — the reflection is seamless")

The ask is sharper than a parallel registry: a reflection consumer must not be
able to *tell a builtin from a user form*. A separate `BuiltinRegistry` that
`resolve`/`metadata-of` *also* consult is the opposite of seamless (two tables,
two code paths). **The registry IS `sym`.** Builtins register into the *same*
`SymbolTable` structures user forms do:

- **`sym.functions`** (`HashMap<String, Arc<Function>>`) — so `sym.get(name)` finds
  a builtin exactly as it finds a user fn. `resolve` works unchanged; the
  reserved-prefix blanket-accept is **deleted**.
- **`sym.binding_metadata`** — so `eval_metadata_of` (UNCHANGED — it reads
  `sym.binding_metadata.get(name)`) returns a builtin's metadata exactly as a user
  form's. Seamless reflection, zero special-casing.

**`Function` already carries the entire parity surface** — `params` (arity),
`param_types` + `ret_type` (type-sig), `rest_param`. So a builtin registered as a
`Function` gets resolve + arity-check + **type-check** + reflection **for free,
identical to a user form**, because the checker's existing call-site machinery
reads those fields. This *upgrades* the day-one four-questions trim: type-sig is
not a separate deferred system — it is `Function.param_types`/`ret_type`,
populated from each builtin's signature.

**The one gap: `Function.body: Arc<WatAST>` is mandatory; builtins have no wat
body.** Represent the native handler by either (a) a synthetic sentinel `body`
that is never evaluated because the runtime dispatch `match` intercepts builtin
names before fn-apply (lean — dead field), or (b) a `Native` body variant
(`enum { Wat(Arc<WatAST>), Native(handler) }`, cleaner, more touch sites). Pick at
build time; (a) is the smaller change.

**One source still holds:** the builtin declaration generates BOTH the
`sym`-registration (the `Function` entry + `binding_metadata`) AND the dispatch
arm. No second copy.

**Revised entry-shape:** there is no bespoke `BuiltinEntry` — the entry IS
`Function` (+ its `binding_metadata`). Day-one populated: `name`, `params` (arity),
`binding_metadata` (`:doc`/`:category`). `param_types`/`ret_type` (full type
parity) populate from the existing per-builtin knowledge in `infer_list` — a slice
(the heavy part), incremental; the fields exist day-one so it grows in seamlessly.

**Slices:**
- **255.1** — `Function` native representation + register builtins into
  `sym.functions` + `sym.binding_metadata` (name + arity + doc); delete the
  reserved-prefix hack. Gates: the 254.R undefined-func probe goes green;
  `metadata-of` answers for a builtin (new probe); resolve seamless; lib + corpus
  green; **bench: no hot-path regression** (phf / generated dispatch from the one
  source).
- **255.2** — populate `param_types`/`ret_type` from `infer_list`'s per-builtin
  knowledge → full type parity + arity/type check on builtin call sites.
- **255.N** — INSCRIPTION; the asymmetry annihilated, builtins first-class.

---

## METADATA CONTRACT (builder co-design — "parity, but you can tell rust from wat by looking")

Seamless *query*, honest *content*: the reflection mechanism is identical for
builtin + user (same `metadata-of`, same map shape, always a map), but the map's
*content* honestly declares provenance. Not hidden — labeled, uniformly.

**Decisions (four-questions, all PASS):**

1. **`Function.body` becomes `FunctionBody { Wat(Arc<WatAST>), Native(NativeHandler) }`**
   — Ruby's C-form model (a C method reflects as a method; its body is native).
   Chosen over a sentinel dead-body because the sentinel would *lie* that a wat
   body exists; `Native` is honest AND makes provenance derivable from the variant
   itself (`Native ⟹ :defined-in :rust`). More touch sites; worth it.
2. **Implicit auto-tagging** — provenance is *derived at the registration site*
   (freeze-loaded wat vs startup-registered rust), never hand-decorated. A wat form
   cannot claim `:rust`; the tag can't lie. (The SSOT/automagic stance: derive,
   don't maintain.)
3. **Guaranteed minimum baseline** — `metadata-of` returns `Some(baseline)` for any
   *registered* form (never `None`). `None` keeps its meaning "binding doesn't
   exist." A registered form always reflects its identity. (Contract upgrade.)
4. **Two provenance axes, both baseline + auto-derived:**
   - `:defined-in :wat | :rust` (language — Ruby-C-form vs wat-source)
   - `:layer :substrate | :userland` (a stdlib wat fn is `:wat`+`:substrate`; a rust
     builtin is `:rust`+`:substrate`; a user fn is `:wat`+`:userland`)

**The universal baseline every callable carries:** `:defined-in` · `:layer` ·
`:name` · `:arity`. So `(metadata-of <anything>)` instantly answers "rust substrate"
/ "wat userland." Optional richer rust-reflection (`:rust-handler "eval_i64_arith"`,
source location — Ruby returns nil for C `source_location`; we can do better) grows
in later; the map is open.

This makes the baseline-metadata change touch BOTH classes: even a bare user `defn`
(today → `metadata-of` None) now carries the baseline (`:wat`/`:userland`/name/
arity). Parity is completed from both ends — builtins gain reflection, user forms
gain the guaranteed baseline — and they meet in one honest, uniform map.

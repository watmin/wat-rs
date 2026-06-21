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

## THE METADATA MAP AS CLASSIFICATION SUBSTRATE (builder co-design)

The metadata map is not only for reflection/provenance — it is the **symbol-
classification single source of truth.** Today the codebase classifies symbols by
*scattered exact-string-matching* (`name == ":wat::…"`, `is_reserved_prefix`,
`starts_with`, the verb-list `matches!`es) — duplicated, drift-prone logic that
re-derives a symbol's *kind* from its name *shape*. That string-shape-as-truth is
exactly how `+'2` and the `make-*-queue` phantoms hid.

**The model (Lisp symbol-plist / Clojure metadata):** declare a classification tag
*at the symbol's definition*; query the tag wherever that classification is needed.
The provenance tags (`:defined-in`, `:layer`) are instance #1; the pattern extends
to any classification currently done by string-matching (e.g. `:tier :kernel`,
`:kind :arithmetic`, `:service? true`).

**Discriminant (refined, four-questions):** tags carry **classification** ("is this
a *kind* of thing"); **identity dispatch** (exact name → handler) stays name-based —
that is not classification. `feedback_mark_the_source_not_memory`: the tag is the
declared truth at the source; checks read it instead of re-deriving kind by shape.

**Scope guard:** 255 ships the *mechanism* (tags declared at definition, queryable
via the metadata map) + the provenance tags. *Migrating* existing string-match
classification sites to tag-queries is a capability 255 unlocks, **harvested
incrementally** (each its own small stone), NOT an all-at-once sweep inside 255.
Mechanism now; harvest forever.

---

## ARC 255 PROMOTED TO ACTIVE — the catastrophic instance, grounded (2026-06-21)

Building rete + the collection campaign re-surfaced this as a **live catastrophic
hole**, not a latent asymmetry. Grounded this session (no speculation):

- `(:wat::core::nonexistent-xyz? 5)` in a typed body **type-checks clean** and only
  fails at runtime *if that branch executes*. A typo'd / retired / nonexistent
  builtin head escapes BOTH static layers.
- **The double-punt** (both layers say "not my job"):
  - resolve — `is_resolvable_call_head` → `is_reserved_prefix(head)` returns true for
    ANY `:wat::*` leaf (walk.rs:189 / reserved.rs:34). Comment admits it:
    *"leaf-level validation is the type checker's concern."*
  - check — no scheme for the unregistered head → permissive `Infer` fallback
    (check.rs:9923 *"may be a primitive / future slice / driver"*).
- Scope: every `:wat::*` builtin namespace (confirmed `:wat::io::bogus` passes too),
  NOT core-only. Unknown **user** fns ARE caught (resolve), so the hole is the
  builtin open-set specifically.
- Exit codes are correct (valid→0, runtime panic→1, MainSig→4) — an earlier
  "exit 0" reading was a measurement error (`$?` through a pipe). No exit-code bug.

**Builder verdict: annihilation. Any flaw is catastrophic; the forward-compat
justification is rejected outright (never agreed to).**

### Expressing "correct": list now, query later (builder)

> *"how we express correct is an impl detail — right now it's a list, later it's a query."*

The invariant is the same in both: **every `:wat::*` call head must resolve to a real
builtin (or a registered user form / macro / protocol method); unknown → a resolve-time
error carrying retirement + near-match remedies.** The blanket-accept is DELETED.

- **Runtime reflection cannot derive the set today** — every reflection verb
  (`metadata-of`, `signature-of-fn`, `lookup-define`, …) queries a GIVEN name; none
  enumerates the builtins, because builtins aren't registered as data. That
  enumeration capability IS this arc's payload. So the *query* expression must wait
  on registration; the *list* expression ships now.

### Decision (2026-06-21): go STRAIGHT to 255.1 — no throwaway 255.0 list

A standalone hand-list (255.0) was considered and **rejected**: it touches the exact
resolver/dispatch seam 255.1 rebuilds, then gets deleted — doing the dangerous part twice.
The hole is dev-time (corpus green, nothing in production), so there is no pressure for an
interim band-aid; 255.1 *is* the safe close. **278 is PARKED; 255 is active and unlocks
278's continuity** (the `List?`→`ast-list?`/`list?` split, retirement-loud-at-resolve, the
container-predicate family, the collection HOF fills all fall out of a sound registry).

### 255 IS ALSO THE MEGAFILE CARVE (builder, 2026-06-21)

> *"when we build 255 we rip out as much shit from the megafiles as we can to
> `src/<namespace>/<scope>.rs` — we've been attacking those huge files strategically to
> make the migration more tractable later."*

255 dissolves `runtime.rs`'s central dispatch (`dispatch_keyword_head` +
`dispatch_keyword_head_value`, ~483 arms). The "one source" declaration for each builtin
lives in its **namespaced home**; each home exposes `register_builtins(&mut …)` that
declares its builtins (name → handler, arity, meta) into `sym`; `runtime.rs` becomes an
assembler that calls each home's registration. The central `match` shrinks toward nothing.
Most homes exist (`collection/`, `channel/`, `process/`, `check/`, `types/`, `comms/`,
`function/`, `services/`) — builtins rejoin them; scalar/arith families (`core::i64`,
`core::f64`, `core::Bytes`, `core::String`, …) get new homes. Soundness fix + carve, one motion.

### THE REFLECTION SURFACE (builder co-design, 2026-06-21) — nothing like it exists today

Confirmed by grounding: there is **zero namespace introspection** today (no `names-in`/
`ns-names` verb, no rust fn lists names under a prefix, `sym` exposes no iteration). 255
builds it from the registry. The model is `ls` + `stat`:

- **A namespace has TWO discrete, non-recursive observables** (dir = subdirs + files):
  1. **child namespaces** — `(child-namespaces :wat::core)` → next-segment children, deduped (`i64 f64 Bytes …`)
  2. **names** — `(names :wat::core)` → leaf callables directly at this level (`map first conj …`)
  (Non-recursive by contract; the caller recurses if it wants. Verb names TBD at build.)
- **Interrogate a name** — `(metadata-of :wat::core::map)` → the per-name map (below). "Is it
  a func / macro / pure / …" is reading `:kind`/`:pure`/etc. off that map.

### THE PER-NAME OBSERVABLE MAP (retained from the METADATA CONTRACT + purity NOTE above)

`(metadata-of <fqdn>)` — same query from wat AND rust, always a map, never `None` for a
registered name, all **auto-derived at registration** (can't lie):
- baseline: `:defined-in` (`:wat`|`:rust`) · `:layer` (`:substrate`|`:userland`) · `:name` · `:arity`
- callable-class (TWO orthogonal purity bits, per the NOTE): `:kind` (`macro`|`fn`|`intrinsic`)
  · `:pure` · `:deterministic` · `:expand-time-legal`
- extensible classification tags (Lisp plist): `:tier`, `:kind :arithmetic`, `:service?`, …
- optional richer rust reflection: `:rust-handler`, source location.

**Absorbs the live stopgap:** `src/rete/purity.rs` (111-line hand-maintained
`{pure,deterministic}` map exposing `:wat::rete::pure?`/`deterministic?`) is the "parallel
hand-list" the NOTE warns of; when 255 lands it becomes a *projection* of the registry —
delete the hand list, the rete predicates query `metadata-of`.

### Build shape

- **Hand-authored seam + first home (reference template):** `FunctionBody::{Wat,Native}`;
  the one-source per-home registration declaration; the resolver rewrite (delete
  `is_reserved_prefix → true`; resolve through `sym`/registry membership + retirement/
  near-match remedy); the perf path (phf / generated dispatch from the one source).
- **Delegated per-home carving repeats:** one home per strike, sonnet under the template,
  each weighed against the full corpus.
- **Completeness gate = the full wat corpus + test cascade** (substrate-as-teacher): any real
  head missing the registry → resolve rejects real code → red → register it. Plus the 254.R
  undefined-builtin probe, the `metadata-of`-answers-for-a-builtin probe, the namespace-query
  probes, and the hot-path benchmark (no regression).

---

## THE RECORD ARCHITECTURE (builder co-design, 2026-06-21) — forced minimum ⊕ adjacent per-def-kind

The question that pins the contract: *"what's the record who describes the feature set
some name must satisfy — a name cannot be registered if it doesn't have what it must."*

Answer: **a forced minimum baseline ⊕ an adjacent per-def-kind record** (wat's own ADT —
sum + product — applied to wat's own symbol table; "like program-envs": a required core
with adjacent typed extensions).

**Grounding (2026-06-21):** the per-def-kind records ALREADY exist — `Function`
(env.rs:35), `StructDef`/`EnumDef`/`RecordDef` (types.rs), `ProtocolDef` (value.rs:461),
`MacroDef` (macros/registry.rs:9). `FunctionBody::{Wat,Native}` ALREADY exists
(env.rs:22, "255.1a") — `Native` is a unit marker, never yet constructed (255.1b begins
construction + registration). What is MISSING is the forced minimum that binds them
(`Function.name` is `Option`; no arity/kind/pure/determinism/expand-time contract).

### 1. The minimum baseline — the contract (can't register without it)

Every registered name carries, REQUIRED: `:name` · `:arity` · `:defined-in` · `:layer` ·
`:kind` · `:pure` · `:deterministic` · `:expand-time-legal`. Forced by the type:
- required fields (non-`Option`), **enum-typed not bool** (`Purity::Pure|Effectful`, not a
  fat-fingerable `true`), **no `Default`** → struct-literal completeness makes "register
  without answering each" a COMPILE ERROR (the exact forcing as an exhaustive match).
- provenance (`:defined-in`/`:layer`) is AUTO-DERIVED at the registration site (can't lie);
  the per-name facts (arity/kind/pure/determinism/expand-time) MUST be supplied.

### 2. The per-def-kind record — adjacent, the form's user-expressable surface

A SUM over def-kinds, each variant its existing record: `Function` (defn/fn) · `StructDef`
· `EnumDef` · `RecordDef` · `ProtocolDef` · `MacroDef` · **`NativeBuiltin`** (NEW, rust
builtins). Each carries what THAT def form lets a user express (`defstruct` →
`:restricted-to`/`:field-metadata`; `defprotocol` → method sigs; native → handler).
Exhaustive sum → a new def form = a new variant = compile error until handled. The
baseline is the shared product; the kind-record is the discriminated payload.

### 3. Query surface = baseline ⊕ kind-record projection

`(metadata-of <name>)` = baseline (always) ⊕ the kind's reflectable fields. Namespace
introspection (`child-namespaces` + `names`, non-recursive) walks the registry; per-name
interrogation reads the map.

### 4. rete / macro-gate / checker → CONSUMERS (builder: "rete just calls this instead")

Because `:pure`/`:deterministic`/`:expand-time-legal` live in the baseline, the three
scattered hand-lists collapse into ONE truth:
- `src/rete/purity.rs` (379 lines, `:wat::rete::pure?`/`deterministic?`) — DELETES; the
  rete predicates query the baseline.
- `macros::is_pure_total` (eval.rs:344, the macro-expand allow-list) — DELETES; queries
  `:expand-time-legal`.
- `runtime::is_effectful_op` (runtime.rs:22731) — becomes the registration-time DERIVER
  that POPULATES `:pure` (then the field is the single truth).
Any rete check that is really "a property of a name" relocates into the baseline; rete
reads it. (NOTE-purity already prescribed the projection; this extends it to all three.)

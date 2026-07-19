# DESIGN — dynamic EDN decode + the opaque telemetry sink (arc 278)

> **Origin (builder, 2026-07-19):** the telemetry service must accept logs from **arbitrary callers** and let
> arbitrary callers **process** them — the rete stack measures itself via telemetry (R25 `MACHINA CHAOS DOMAT`);
> it emits logs + metrics with arbitrary payloads that must **store + query without fault**. **Grounded blocker:**
> `tests/services/probe_arc278_journal_logs_on_process` is `#[ignore]`'d — a forked `journal'` child faults
> `UnknownTag` (`edn_shim.rs:907`) decoding a user `LogMessage` record whose type isn't in the child's baked
> registry, dies, closes the channel. Metrics cross fine (all-stdlib fields). This is the "tagged edn literals
> that failed on the receiver."

## The decomposition (builder's ruling — the load-bearing correction)
A sink/store service is a **DUMB OPAQUE store** — it must NOT decode arbitrary caller types (trying to be
omnipotent about all types = a **DoS surface**). Decoding foreign data is the **CONSUMER's** problem.
See `[[feedback_sink_is_opaque_store_consumer_decodes]]`. *(I first drifted toward a type-aware sink; the builder
cut it: "the telemetry service … is just a store to be queried … should not expose itself to a DoS vuln trying
to be omnipotent about all types.")*

- **Sink (telemetry):** carries `Log.message` **OPAQUE** (EDN-text `String`). Writes / forks / stores / returns
  it verbatim, **never decodes** → no `UnknownTag` across the fork, no DoS surface. rete self-measurement works
  because producer == consumer holds its own types and `edn::read`s its own messages back.
- **Consumer:** owns the decode. Holds the symbols → typed value. Lacks them → **dynamic value** (below).

## The capability — dynamic EDN decode (the keystone, general substrate)
`edn::read` gains a **DATA MODE**. Grounded on the **wire-purity invariant**: anything that crossed the wire as a
tagged value is a **pure record OR pure enum** (records-are-EDN + the **293.W purity wall** — a struct/impure
value cannot be typed onto a wire peer). So an unknown tag is *always* reconstructable as a self-describing
dynamic value, dispatched by the **body shape the decoder already uses** (`edn_shim.rs:172-178`, `:2331-2332`):

| body shape | today (strict, unchanged) | data-mode (new) |
|---|---|---|
| map `{…}` | record; `UnknownTag` on unregistered (`:2424`) | **dynamic record** `{class, name-keyed fields}` |
| vector `[…]` | enum tagged-variant; `UnknownTag` (`:2651`) | **dynamic enum-variant** `{enum-class, variant, positional fields}` |
| nil / bare | enum unit-variant | **dynamic enum unit-variant** `{enum-class, variant}` |

- **Self-describing:** the tag is fully qualified — `#ns/Type` (record) / `#<enum-path>/<Variant>` (enum,
  `edn_shim.rs:1784`) — so the type/variant + fields are all present; no registry lookup, no registry mutation.
- **Recursive:** a dynamic value's fields decode the same way — nested unknowns → nested dynamic values. A
  foreign record *containing an unknown enum field* (or an enum carrying an unknown record) decodes all the way
  down. *(This is the enum fault, resolved: enums make it symmetric, not infeasible.)*
- **Re-serializes faithfully** back to the same tag + body (store / forward / re-query round-trip).
- **One shape per fully-qualified tag, contradiction = exception:** same tag → same shape expected; drift →
  exception (honest). Enum variants are distinct tags (`/A` vs `/B`), so a multi-variant enum does **not**
  false-contradict — each `(enum, variant)` is its own tag, its own one-shape.

## The fault resolved up front — no-hidden-failures (R41 `EGO SVM LEX`)
`edn::read` **STAYS STRICT BY DEFAULT** — an unknown tag → error (catches typos; holds the no-hidden-failures
floor). The dynamic-value reconstruction is an **explicit consumer-opted MODE** ("read foreign data; dynamic
values expected"). A silent global default would turn a typo'd tag (`#typo/Naem {…}`) into a silent dynamic
value — a masked failure, forbidden. Strict-typed vs data-dynamic **is** the builder's "querier decides."

## Scope
- **IN:** unknown *user* tags (record map / enum vector / enum nil), **data-mode only**.
- **OUT (unchanged, both modes):** stdlib tags, `Option`/`Result`, `#inst`, all known/registered types, the
  sink (never decodes — opaque), and strict mode (errors on unknown — the floor).

## The wire-convention refinement (deduced 2026-07-19, ratified) — the floor under the decode
Materializing `read-foreign` surfaced a latent flaw: `nil` was **three-way overloaded** — the unit value
(`Value::Unit`), a user enum **unit-variant** (`#ns/Variant nil`, `edn_shim.rs:1395/2332`), AND `Option::None`
(`#wat.core.Option/None nil`, `:34`). And `nil`-body is **arity-ambiguous**: `#X nil` can't distinguish a
zero-field variant from a one-field variant holding `nil` (`Some(())` = `#Option/Some nil`) — a crutch the typed
registry papers over but `read-foreign` (unknown tags) cannot. **Ratified fix:** a variant's body is its
field-vector — **unit variant → `[]`** (zero fields), tagged → `[items]`; `nil` = the unit value ONLY.
`Option::None` → `#wat.core.Option/None []` (None *is* a unit variant; the honesty that "None ≠ bare nil" lives
in the TAG, not the body, so it's preserved). **FULL UNIFORMITY (builder ruling 2026-07-19, correcting an earlier
half-measure):** `#Option/Some nil` is illogical — a variant holding `nil` is a one-field vector `[nil]`, not a
bare `nil`. So **every variant, including Option/Result, is vector-bodied** — `None → []`, `Some(v) → [v]`,
`Some(nil) → [nil]`, `Ok(v) → [v]`, `Err(e) → [e]` — and the **arc-298.1 direct-body special-case
(`#Option/Some v`) is RETIRED entirely.** One rule, no exceptions: **record → `{field-map}` (map); enum variant
→ `[field-vec]` (vector, any arity); `nil` → the unit value only.** Body-shape is now a *perfect* discriminator
(map=record, vector=variant, nil=unit). Blast radius is **NOT small** (the direct-body form is pervasive):
**~52 `.edn` goldens** (51 with `#Option/Some`/`Result/Ok`/`Err` direct-body + 8 unit/`None nil`), **three
encoders** (`value_to_edn_with` `:1992-2005`+`:2984`, `value_to_edn_notag`, the JSON writer `:2125-2137`), the
**decoder** (`:1588-1626` direct-body → `[v]`; `:1819-1822` typed-coerce), and the **clj bridge** (`wat-edn-clj/
src/wat_edn/core.clj:40-49` readers + `:217-222` writers → vector bodies; `produce_shapes.clj`, `prove.clj`,
parity; **reconcile the spelling** — clj writes short `#wat.core/Some`, rust writes `#wat.core.Option/Some`).

## STATUS (2026-07-19+) — the campaign FLOOR is complete; the SIFT TIER is the live target
- **A.0 ✓** (`c9bfa8fd`) — uniform variant encoding (the floor).
- **A ✓** (`b68a130a`) — `read-foreign` / `ForeignRecord` / `ForeignVariant`; strict `read` untouched. (gate 2/2; floor 4207/0.)
- **B ✓** (`dc5427a4`) — opaque `Log.message`/`Span::LogRequest.message` (String); the sink never decodes;
  `dead_child_speaks` re-pointed (a); the `#probe/Note` fork blocker GONE. (floor 4208/0.)
- **C ✓** (`COMPONENDO DELEO`): **C1** (`27737ca9`, −6557 — the 3 legacy crates + interrogate + STOP-2 probe + the
  legacy check-lint), **C2** (`3266e363`, −376 — Tagged/NoTag + write-notag; kept `value_to_json_natural`), **C3**
  (`f11e64db` — the wat-fix de-prime). `:wat::telemetry::` / `:wat::sqlite::` are OURS. (floor 4150/0.)
- **THE SIFT TIER (T2 → R0) — the LIVE TARGET** (`DESIGN-sift-server-side-filter.md`, incl. a grounded
  "Predicate-form strike — GROUNDED (scout)" section): the chaos engine's first (paged) form, what A/B/C were the
  floor for. `sift-logs`/`sift-metrics` (Journal ops) taking `Sieve = Predicate | Rules` (`wat.query`); one seed per
  fire (alpha-only structural); a `runner-count 1` throwaway worker. **Pinned contract:** the `Predicate` field is a
  `:wat::core::String` of EDN source (NOT `:wat::WatAST` — nulls across a fork); server does `read-string → verify
  (:wat::rete::pure? ∧ deterministic?) → eval-ast! → apply` per seed. Task #5 Predicate → #6 Rules → #7 R0 streaming.
- **Deferred (NOT this campaign):** clippy's ~1000+ warnings are the pre-existing known-cruft pile (CI marks clippy
  "informational" — no zero-floor); a dedicated cleanup, arc-109-style. C was clippy-neutral.

## The campaign — all committed, no deferral (builder: "i do not wish to defer this")
- **Stone A.0 — uniform variant encoding: every variant is vector-bodied (the floor).** `None → []`,
  `Some(v) → [v]`, `Ok(v) → [v]`, `Err(e) → [e]`, user unit → `[]`, user tagged → `[items]`; **arc-298.1
  direct-body RETIRED**; `nil` = the unit value only. Touches THREE encoders (`value_to_edn_with`
  `:1992-2005`+`:2984`, `value_to_edn_notag`, the JSON writer `:2125-2137`), the decoder (`:1588-1626` direct-body
  Option/Result → `[v]`; `:2332` `Edn::Nil => reconstruct_enum_unit` → `Edn::Vector([])`→unit, `nil`→`Value::Unit`;
  `reconstruct_enum_unit` `:2689` merges into the tagged path; `:1819-1822` typed-coerce), **~52 `.edn` goldens**
  (51 direct-body Option/Result + 8 unit/`None nil`), and the **clj bridge** (`wat-edn-clj/src/wat_edn/core.clj`
  readers `:40-49` + writers `:217-222`, `produce_shapes.clj`, `prove.clj`, parity; reconcile the short-vs-Option
  spelling). RED gate: `None`/user-unit round-trip `[]`; `Some(v)` round-trips `[v]`; `Some(nil)` → `[nil]`; bare
  `nil` → `Unit` (never a variant). **MUST land first** — A's dispatch + B's telemetry goldens depend on it. The
  golden churn is mechanical (cascade names each); the code change is a few focused arms.
  **A.0 spans TWO surfaces** (the wat↔clj bridge is in scope — builder flag, grounded): (1) **wat side** —
  `edn_shim` encoder/decoder + the ~8 `.edn` goldens above. (2) **clj bridge** (`crates/wat-edn`) — the interop
  fixtures hardcode nil-body None: `interop-tests/clj/produce_shapes.clj:25,43` (`(tagged-literal 'wat.core/None
  nil)` → `[]`), `clj/prove.clj:41-42` (`"#wat.core.Option/None nil"` → `[]`); the clj reader
  `wat-edn-clj/src/wat_edn/core.clj:42-43` keys on the *tag* (body-agnostic — verify, likely no change); the
  parity tests (`tests/clj_oracle_parity.rs`, `token_unicode_parity.rs`) realign (wat emits `[]` ⟺ clj expects
  `[]`). The `wat-edn` CODEC itself is unchanged (structural, tag-agnostic). Also reconcile the two None spellings
  found in the bridge (`wat.core/None` vs `wat.core.Option/None`).
- **Stone A — dynamic EDN decode** (the keystone; general substrate). The dynamic-value family (dynamic record +
  dynamic enum-variant) + `edn::read` data-mode + one-shape-per-tag + recursive. Naming/shape of the dynamic
  value → **intueri cast** when drawn (Clojure's cousin is `tagged-literal`; ours is aggregate-aware). Benefits
  everything that reads possibly-foreign EDN, not just telemetry.
- **Stone B — telemetry applies it.** `:wat::telemetry'::Log.message` → opaque `String`; the producer
  (`span.wat`'s `log` op) `edn::write`s the message at the call site; the sink stays a dumb store; un-`#[ignore]`
  `probe_arc278_journal_logs_on_process` (write **and** query an arbitrary-payload log across a process fork).
- **Stone C — annihilate + fold + de-prime the whole family** (`COMPONENDO DELEO`; scope widened by builder
  2026-07-19: *"this can include sqlite and then we can reclaim that name in core along with de-priming mem-store
  too"*). Fold **three legacy crates into core** — `crates/wat-telemetry` (legacy `Event`/`WorkUnit`),
  `crates/wat-telemetry-sqlite`, `crates/wat-sqlite` (all currently workspace members, `Cargo.toml`) — kill
  `Tagged`/`NoTag` (`wat/edn.wat:32-33`) + their `write-notag` / sqlite auto-dispatch machinery
  (`crates/wat-telemetry-sqlite/src/auto.rs:497-504`), and **de-prime the family**. Two kinds of de-prime,
  grounded (`wat/sqlite.wat:34-35`, `stdlib.rs:358` — the in-core store family is prime-on-last-segment and its
  namespaces are described "net-new, no battery collides"):
    - `:wat::telemetry'::*` → `:wat::telemetry::*` — a **true reclaim**: requires the legacy `wat-telemetry`
      crate (which occupies `:wat::telemetry::Event` etc.) annihilated first.
    - `:wat::sqlite'` / `:wat::query::mem-store'` / `:wat::query::sqlite-store'` → bare — **possibly just a
      rename** (net-new namespaces). **GROUND FIRST:** what does each prime guard? (does `crates/wat-sqlite`
      define a non-prime `:wat::sqlite::*` holding the name, or is the `'` transitional style only?) +
      whole-tree consumer grep per crate before deletion (never "nothing uses X" from a subset).
  Plus: **delete** `probe_arc278_process_crash_reason_carried` — it tests the **STOP-2 non-goal** (crash reasons
  are admin-channel-only, never broadcast to clients — `[[feedback_ask_who_already_receives_it_before_building_delivery]]`;
  ratified 2026-07-19).

## Grounded citations (verified this session)
- `src/edn_shim.rs`: `:172-178` (body-shape dispatch — map=struct/record, vector=enum, nil=unit), `:861/907`
  (`UnknownTag`), `:2331-2332` (live dispatch arms), `:2411/2424` (`reconstruct_struct`→UnknownTag map),
  `:2469` (`reconstruct_record`), `:2639/2651` (`reconstruct_enum_tagged`→UnknownTag vector), `:1784`
  (enum tag = `<enum-path>/<Variant>`), `:43` (opaque-handle convention).
- `src/runtime.rs`: `:5700-5730` (record value = `class_fqdn` + positional `fields` + `RecordDef.field_names`
  in the TypeEnv), `:469` / `EnumValue` (`{enum_name, variant_name, fields}`).
- `wat/telemetry.wat`: `:82` (`LogMessage` open surface), `:97-101` (`Log`), `:118-160` (`Journal` write/query).
  `wat/telemetry/journal.wat`: `:45/58` (`data = edn::write` of the record), `:154/184` (query hydrate =
  `edn::read`) — **the store already round-trips as EDN text; the fault is the wire/decode, not the store.**
- The purity wall: **293.W** — a struct can no longer be typed onto a wire peer at check time (the invariant
  that makes opaque carriage + the dynamic-record reconstruction sound).
- Legacy `Tagged`/`NoTag` (the carriers to annihilate): `wat/edn.wat:32-33` (newtypes over
  `:wat::holon::HolonAST` — holon-coupled, on the 294 chopping block); consumers in `crates/wat-telemetry*`
  (`Event.wat`/`WorkUnit.wat`/`WorkUnitLog.wat`/`Reader.wat`), `src/edn_shim.rs` (`value_to_edn_notag`,
  `holon_ast_to_edn_notag`), `crates/wat-telemetry-sqlite/src/auto.rs:497-504` (the dispatch), tests + examples.

## Sequencing note
Stone A is the keystone the rest stands on (the general decode). Stone B is the telemetry application (opaque
carrier + un-ignore). Stone C is the annihilation/fold that rides on the opaque carrier + the retired legacy
crate. Order A → B → C, or B-first if the rete-green is wanted before the general capability (B works with strict
mode alone; A adds the type-less-consumer path). Each stone: draw the DESIGN + a RED disconfirming probe → brief
→ delegate a shadowdancer → weigh by own re-run.

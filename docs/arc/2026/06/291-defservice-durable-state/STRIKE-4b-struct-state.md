# Arc 291 — Strike 4b: State is a struct; the soul wears a body (the prophecy, manifested)

**Status: STRIKE-READY (4b-i).** R8's manifestation made concrete. The firm law (builder): **a struct shall
never cross the wire — categorically, by kind; only records (EDN) cross.** This is what lets defservice host
a *resource* State (cache/socket/fd) and `hibernate` honestly: the struct is the body (in-locus resources),
the EDN Snapshot is the soul (crosses). Recon verdict (`is_portable_type(Struct) → false`, `-p wat`): **one
real break** (`service-template`'s old ship-State pattern); the resource crates (`wat-lru`/`wat-holon-lru`)
already comply (struct State in-locus, records/enums on the wire). Minimal and more correct.

## Staging
- **4b-i — the substrate law (this strike).** `is_portable_type(Struct) → false` (categorical, reversing
  254.1's field-recursion). Migrate the one break. The type-level wall.
- **4b-i-b — value-level audit (follow-on).** Once the type-gate is up, audit whether any struct can still
  reach `closure_extract::encode_struct` at runtime; if a non-type-gated path exists, make `encode_struct`
  refuse ("structs are not wire-serializable; use a record"). Belt to the type-gate's suspenders.
- **4b-ii — defservice State → struct (DESIGN PINNED below).**
- **4b-iii — the RED probe: a soul wearing a body.** A defservice whose `:state` holds a genuinely non-EDN
  field; compiles only with a `:hibernate` projection; hibernate→kill→resume sheds and rebuilds the resource.
- **Then:** R1 → full PROBATUM EST · the pause (builder's) · the 291 INSCRIPTION.

## 4b-i — the brief (substrate law)

**The work.** Make a struct categorically non-portable, so a struct can never be a channel/wire payload.

**The changes (exact):**
1. `src/check.rs:13044` — `is_portable_type` Struct arm: `Some(TypeDef::Struct(s)) => s.fields.iter().all(...)`
   → `Some(TypeDef::Struct(_)) => false`. (Keep the comment honest: struct ↛ wire by kind; arc 291 4b
   supersedes 254.1's field-recursion. A struct holds resources + EDN; only records cross.)
2. **Migrate the one break** — `wat-tests/service-template.wat`: `:svc::State` is a `defstruct` shipped over
   channels (`Sender<svc::State>`, `DriverOut`, `Thread<nil,svc::State>`, the final-state-on-disconnect). It
   is **all-EDN data** (`push-count`/`ack-count` i64s) → under the law it **is a record**. Change
   `(:wat::core::defstruct :svc::State [...])` → `(:wat::Record::def :svc::State [...])`; fix any
   `:svc::State/new` / struct-field sites that the record form changes (records have named accessors too;
   the constructor may shift `State/new` → the record ctor). Keep the test's behavior identical.
3. **Un-ignore** the RED probe — `tests/nursery/probe_arc254_channel_payload_portable.rs`:
   `channel_of_all_edn_struct_must_be_rejected` (remove the `#[ignore]`).

**STOP triggers:**
1. STOP if `is_portable_type(Struct) → false` breaks MORE than `service-template` + the known floor — the
   recon said exactly one real break. If the resource crates (`wat-lru`/`wat-holon-lru`) or anything else go
   red, STOP and report the site list (the design says they already comply).
2. STOP if migrating `:svc::State` to a record needs more than a def-form + accessor swap (e.g. it holds a
   genuinely non-EDN field) — surface it; the recon classified it as all-EDN.
3. Do NOT touch `closure_extract::encode_struct` in this strike (that's the 4b-i-b audit follow-on).

**Blast radius:** `src/check.rs` + `wat-tests/service-template.wat` + the nursery probe un-ignore. Do NOT
touch defservice (`wat/service.wat`) — State→struct is 4b-ii.

**Verify (run yourself, report exact output):**
- `cargo test -p wat --test nursery channel_of_all_edn_struct` → 1 passed (the firm rule now rejects it)
- `cargo test -p wat --test nursery` → all green (no over-rejection of portable payloads)
- `cargo test -p wat --no-fail-fast` → orchestrator weighs SET-diff vs HEAD (expect ⊆ {service-template
  fixed → green} ∪ floor; net ∅ new after the migration)
- the resource crates: `cargo test -p wat-lru -p wat-holon-lru` → stay green (they already comply)

Runtime: 15–30 min. Trap-door: the `service-template` record migration (the one break) — everything else is
the 1-line `is_portable_type` flip + the un-ignore.

---

## 4b-ii — DESIGN PINNED (the keystone re-tool; the soul wears a body)

> ⚠ **SUPERSEDED 2026-06-24 — the CURRENT contract is "## 4b-ii — CONTRACT EVOLVED" at the END of this
> doc.** A long co-design this session refined the contract on six axes: clause names `:record`/`:state`
> → **`:durable`/`:ephemeral`** (intueri); **all-kwargs** surface; **`:durable` optional** (omit → empty
> record); **`:init` conditional default** (required only when `:ephemeral` non-empty, else macro-error);
> **fixed `start` arity**; **`:hibernate`/`:stop` as projection hooks with defaults**. This section is kept
> for the reasoning path; **4b-ii-a builds the EVOLVED section.** (amend-with-recognition — nothing deleted.)

The contract, settled across the co-design (R8 + the `record`/`Status` namings):

```clojure
(defservice :my::counter
  :record [count <- :i64]          ;; THE SOUL — a record (EDN, crosses the wire) → mints :my::counter::Record
  :state  []                        ;; THE FLESH — ephemeral/resource fields (never cross); `record` is PREPENDED
  :ops    [(:Increment [s <- :State n <- :i64] -> [...]
             ;; durable read: (Record/count (State/record s)) ; ephemeral: (State/<field> s)
             ...)]
  :init   (:wat::core::fn [r <- :my::counter::Record] -> :State ...))  ;; THE ONE CONSTRUCTOR: Record → State
```

### The structural law
1. **`:record [fields]`** mints `:<fqdn>::Record` (the durable EDN record — the soul) and **prepends a field
   named `record`** (typed `:<fqdn>::Record`) as the FIRST field of the State struct.
2. **`:state [fields]`** declares ONLY the ephemeral/resource flesh (sockets, caches, rate-limiters). Empty
   for a pure-data service. `:state` now mints a **`defstruct`**, not a record. The struct is
   `{ record <- :<fqdn>::Record, <ephemeral fields…> }` — non-portable BY KIND (4b-i), so it never crosses.
3. **The four lifecycle verbs collapse to ONE user function** — `:init : Record → State`:
   - `start` = `init` from the *initial* record · `resume` = `init` from a *saved* record. Same function.
   - `hibernate` = **`(State/record s)`** — a field read, emits the record. NO `:hibernate` callback.
   - `stop` returns a resp (default: the record; or the author's `:stop` projection).
4. **The wire carries records only.** `Admin::{Init,Resume}` carry a `:<fqdn>::Record`; `Status::Hibernated`
   carries a `:<fqdn>::Record`; `Status::Stopped` carries the resp. The struct never crosses (4b-i enforces).
5. **The user draws the soul/flesh line by FIELD PLACEMENT** — durable → `:record`; ephemeral → `:state`.
   "internal state they don't want on resumption" lives in `:state`, rebuilt fresh by `:init` each spawn.

### The rename (folded in — intueri verdict, `Status` builder-confirmed)
| old | → new | role |
|---|---|---|
| `LineageUp` (enum) | **`Status`** | the service announcing its operational state up to the owner |
| `LineageUp::Final` | **`Status::Stopped`** | the stop return (parity with `:Stop`/`stop`) |
| `lineage-extract-addr` | **`extract-addr`** | drop the plumbing prefix |
| `init-from-admin` | **`dispatch-admin`** | names the act (dispatch a startup Admin msg) |
| (keep) | `Admin`, `Status::Started`, `Status::Hibernated`, all owner-facing | clear |
Internal let-binding names follow (`lineage-up-ty → status-ty`, `lineage-final-kw → status-stopped-kw`, …).
The ONE literal break outside service.wat: `probe_arc209_c2`'s `Peer'<…::LineageUp,…::Admin>` → `…::Status`.

### Revises 4a (which this supersedes)
4a's hibernate returned the whole State; resume took the whole State; both via the lineage protocol. Now:
hibernate emits `(State/record s)` (a record); resume = `init` from a record; `Admin::Resume`/
`Status::Hibernated` carry `:<fqdn>::Record` not `:<fqdn>::State`. 4a proved the protocol; 4b-ii makes it true.

### Migration (the cascade, accepted)
Every existing defservice + probe migrates `:state [data] → :record [data] + :state []`, declares `:init`
(`Record → State`), and reads durable data through `(Record/… (State/record s))`. The no-`:init` "ship a
pre-built State" default is **dead** (a struct can't ship) → `:init` is required. Affected: `service-locus-
parity` (counter_on), `service-init-parity` (seeded), `service-admin-facet` (admin_stop), `service-stop-
resp` (stop_resp), `service-hibernate-resume`, the arc272/arc209 probes.

### Decomposition (so the kill stays clean)
- **4b-ii-a — the macro re-tool + back-compat migration**: `:record` clause + struct State + the four-collapse
  + the renames; migrate counter_on/seeded/admin_stop/stop_resp/hibernate_resume + arc272/arc209 probes to
  the new shape. Gate: all those green; SET-diff vs HEAD = ∅.
- **4b-iii — the resource RED probe (the true proof)**: a defservice whose `:state` holds a genuinely
  non-EDN field (a `Receiver`/handle), `:record` holds the durable data; proves (a) the struct compiles
  holding a resource, (b) hibernate→kill→resume emits the record + rebuilds the resource fresh. THE honest
  fulfillment → then R1 PROBATUM EST · pause · INSCRIPTION.

---

## 4b-ii — CONTRACT EVOLVED (2026-06-24 co-design) — **THIS is what 4b-ii-a builds**

Settled across a long co-design (this session). The "DESIGN PINNED" section above is the earlier reasoning,
preserved. Six refinements over it: clause names → `:durable`/`:ephemeral`; all-kwargs surface; `:durable`
optional; `:init` conditional default; fixed `start` arity; `:hibernate`/`:stop` as projection hooks.

### Worked example — the two poles

```clojure
;; POLE 1 — pure-data service (the degenerate "soul with no body"): :durable + :ops, the rest defaults.
(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ops [(:Increment [s <- :State n <- :wat::core::i64] -> [value <- :wat::core::i64]
          (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) n)]
            (:wat::service::Outcome::Reply
              (:my::counter::State/new (:my::counter::Record c))      ;; build next State = struct wrapping the record
              (:my::counter::IncrementResponse c))))])
;; :ephemeral defaults empty → :init defaults (State/new d) → :hibernate/:stop default (State/durable s)
;; start: (my::counter/start locus (:my::counter::Record 0))         ;; caller ALWAYS supplies the record

;; POLE 2 — resource service (a body): :ephemeral declared → :init REQUIRED (macro can't build a cache).
(:wat::service::defservice :my::lru
  :durable   [capacity <- :wat::core::i64]                 ;; the durable identity (config); entries are ephemeral → cold resume
  :ephemeral [cache <- :my::LruCache  hits <- :wat::core::i64]
  :ops [ ... ]
  :init (:wat::core::fn [d <- :my::lru::Record] -> :my::lru::State
          (:my::lru::State/new d (:my::LruCache/new (:my::lru::Record/capacity d)) 0)))
;; start: (my::lru/start locus (:my::lru::Record 1024))
```

### The surface (all-kwargs; `(defservice :fqdn & clauses)`, order-independent)

| clause | required? | mints / role |
|---|---|---|
| `:durable [fields]` | optional (default `[]`) | mints `:<fqdn>::Record` (durable EDN identity — **CROSSES**); prepended as the State struct's first field, named `durable` |
| `:ephemeral [fields]` | optional (default `[]`) | the live body (resources/telemetry); mints `:<fqdn>::State` (a `defstruct`) = `{ durable <- ::Record, <these> }` — **NEVER crosses** (4b-i) |
| `:ops [clauses]` | **REQUIRED** | the typed API; op-clause syntax `(:Op [in] -> [out] body)` **UNCHANGED** |
| `:init` | required IFF `:ephemeral` non-empty; else default | `Record → State`. Default `(fn [d <- ::Record] -> ::State (::State/new d))`. **Macro-error if `:ephemeral` non-empty and `:init` absent.** |
| `:hibernate` | optional | `State → ::Record`. Return type **FORCED** to `::Record` (resume = `:init` consumes it). Default `(fn [s] -> ::Record (::State/durable s))`. |
| `:stop` | optional | `State → :Resp`. Return type **USER-DECLARED** (any EDN-portable type — scalar or a record the user defines OUTSIDE the block). Default returns the durable record `(::State/durable s)`. |
| `:record-parent <p>` | optional (existing) | now parents `:<fqdn>::Record` (holon vs base), NOT the struct |

### Naming — axis at the clause, kind at the type (intueri cast 2026-06-24 + a kind-vocab override)
- **Clauses name the AXIS:** `:durable` / `:ephemeral`. intueri's gem — the old `:record`/`:state` named the
  format-and-the-whole, and `:state` was an **L1 lie** (claimed the whole; declared only the ephemeral part).
- **Types keep the KIND:** `:<fqdn>::Record` / `:<fqdn>::State`. intueri argued `::Durable`; **OVERRIDDEN** —
  record-vs-struct IS the 4b-i wire-boundary law (record=crosses, struct=stays). The clause→type jump TEACHES
  it: `:durable [...] => ::Record` reads "durable ⟺ record ⟺ crosses." struct field is `durable`.
- **Hooks stay a uniform trio** `:init`/`:hibernate`/`:stop` (intueri's `:on-stop` rejected — broke trio consistency).

### Lifecycle — fixed arity (no conditional shapes; the user always supplies *something*)
- `start  [locus  r <- ::Record] -> Handle` — caller ALWAYS supplies the record; empty-`:durable` → `(::Record/new)` explicitly (the macro never conjures it).
- `resume [locus  r <- ::Record] -> Handle` — caller supplies the saved record.
- `hibernate [h <- Handle] -> ::Record` · `stop [h <- Handle] -> :Resp`.

### Defaults — principle: *default the unambiguous; require the rest; loud-error the gap*
- `:init` defaults to the trivial wrap `(::State/new d)` ONLY when `:ephemeral` is empty (no body to build).
  With a body → required; omission is a **macro-error** ("`:ephemeral` declares [...] but no `:init` — the
  macro can't construct ephemeral fields; provide `:init : Record → State`"). Construction needs the body's
  recipe (the user's); destruction does not.
- `:hibernate`/`:stop` default to `(::State/durable s)` ALWAYS (you can always *shed* a body; you can't always
  *build* one). `:hibernate` return type forced to `::Record` (the resume seed); `:stop` return type is the
  user's. Both bodies are free (logs, metrics, DB writes) — only the final value's TYPE is constrained.
- `:durable` omitted → empty `::Record` (zero fields → one value). The record FIELD always exists → hibernate/
  resume are TOTAL even for a pure-resource service ("it must exist; you can effectively ignore it").

### Composition — defservice owns the intrinsic surface, NOT your file
The block holds the service DECLARATION (the clauses). The macro EMITS the intrinsic machinery (`::Record`,
`::State`, `Op`/`Reply`, per-op `Request`/`Response`, `serve`, ctors/methods, `start`/`resume`/`stop`/
`hibernate`, `Handle`, the lineage protocol). **Everything else lives OUTSIDE as normal top-level forms** — a
custom `:stop` `:Resp` type (`Record::def` it yourself), helpers, shared records — referenced by name (define
shared types in scope before use). A service is one form among many; it does not swallow the file. (R5 composition.)

### Wire (records only — 4b-i enforces)
`Admin::{Init,Resume}` carry `:<fqdn>::Record`; `Status::Hibernated` carries `:<fqdn>::Record`;
`Status::Stopped` carries `:Resp` (must be EDN-portable — scalar or record, NEVER a struct). The struct never crosses.

### Op handlers (user-written, in `:ops` bodies)
durable read `(::Record/<f> (::State/durable s))` · ephemeral read `(::State/<f> s)` · next State
`(::State/new (::Record/new …) <ephemeral…>)`.

### Decomposition (refined)
- **4b-ii-a** — the macro re-tool to THIS surface (all-kwargs, durable/ephemeral, struct State, Record soul,
  conditional `:init` + macro-error, `:hibernate`/`:stop` projections+defaults, fixed arity) + migrate ALL
  ~17 definers/probes (incl. `probe_arc272_rs1_state_must_be_record` — its "state must be a record" premise
  INVERTS to "state is a struct"; rewrite+rename it, keep its still-valid bare-keyword/unknown-option tests).
  **KEEP the internal lineage names** (`LineageUp`/`init-from-admin`/`lineage-extract-addr`). Gate: all green + SET-diff ∅.
- **4b-ii-b** — the internal lineage rename (`LineageUp→Status`, `Final→Stopped`, `init-from-admin→dispatch-admin`,
  `lineage-extract-addr→extract-addr`). Mechanical, fix-wat where possible; the one `.rs` literal break is `probe_arc209_c2`.
- **4b-iii** — the resource RED probe (`:ephemeral` holds a genuinely non-EDN resource; `:hibernate` projects
  it → record; resume rebuilds it). THE honest fulfillment + where the `:hibernate` *override* earns its test.
  → R1 PROBATUM EST · pause (builder's) · INSCRIPTION.

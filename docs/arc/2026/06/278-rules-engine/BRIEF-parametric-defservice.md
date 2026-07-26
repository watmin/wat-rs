# BRIEF — make `defservice` type-param aware: prove a minimal PARAMETRIC service builds

> Builder-ruled 2026-07-25: *"alright - let's get it functional - prove a minimal parametric service can be
> built."* Unblocks cache **Stone 2** (`:wat::cache::lru-svc` over the generic `:wat::cache::Lru<K,V>`).
>
> **This closes a question that has been formally OPEN for a month.** `docs/arc/2026/06/290-crate-resync/SCOPE.md:64-69`
> posed it — *"Open design question (probe FIRST): does `defservice` support a generic `<K,V>` service, or only
> monomorphic? … Write a 10-line defservice-with-type-params probe before briefing the build"* — and that probe was
> never written; arc 290 was absorbed into 291, which cut generics from its own scope by deferring to an arc-290
> ruling that never came (`291/DESIGN.md:128`). **There is no prior ruling. We are ruling it now, by building it.**

## What the orchestrator already PROVED by probe (do not re-derive)

A minimal parametric service was written and `--check`ed this session. Results:

- **`defsurface` takes type params FINE.** `(:wat::core::defsurface :probe::Box<T> :nature :wat::kernel::Peer' …)`
  passed. Parametric surfaces are a shipped capability (arc 170 C2 — `src/types/surface.rs:16-24`; live at
  `wat/capability.wat:44-46`, `Dialable<S,R>`).
- **`defservice` FAILS**, and here is the exact error:
  ```
  #wat.type/MalformedName {:message "malformed type name \":probe::box-svc<T>::Record\":
    parametric name must close with '>'"  :location wat/service.wat:380}
  ```
- **The parser is INNOCENT.** `:probe::box-svc<T>::Record` genuinely *is* malformed. The bug is that the macro
  BUILDS that name.

## The defect (grounded — `wat/service.wat:260-264`)

```clojure
;; ── 4b-ii: mint state-ty as :<fqdn>::State, record-ty as :<fqdn>::Record ──
state-ty  (keyword/from-string (string::interpolate "{fqdn-str}::State"  :fqdn-str fqdn-str))
record-ty (keyword/from-string (string::interpolate "{fqdn-str}::Record" :fqdn-str fqdn-str))
```

Naive string concatenation onto the raw fqdn. `probe::box-svc<T>` + `::State` → `probe::box-svc<T>::State`.
It must be **`probe::box-svc::State<T>`** — suffix appended to the BASE, type params re-attached at the end.

## The work

1. **A name/type-param split**, applied wherever a companion name is minted. `fqdn-str` → `(base, params)`.
   - Tools available in wat: `:wat::core::string::{split, subs, length, concat, join}` (see `wat/string.wat`'s
     header for the inventory).
   - Precedent that a Rust intrinsic is also viable: `wat/string.wat:13` — *"pascal->kebab is a Rust intrinsic
     (the defservice macro calls it at expand time)"*. A reference implementation exists at
     `src/runtime.rs:3207` (`split_name_and_type_params`) — **not callable from wat today**; expose it only if the
     wat-side split proves genuinely awkward, and say so in your report.
2. **Apply it at EVERY derived name, not just the two above.** `wat/service.wat` mints many from `fqdn-str` —
   `state-ty`/`record-ty` (`:260-264`), `state-new-kw` (`:276`), `state-durable-kw` (`:325`), `init-name` (`:318`),
   `stop-project-name` (`:340`), `hibernate-project-name` (`:369`), and any others you find. Grep `fqdn-str`
   and treat the whole set.
3. **Thread the type params onto the GENERATED defns.** e.g. `:320` `(defn ~init-name ~init-params-vec -> ~state-ty …)`
   needs its params declared for a parametric service. Generic top-level `defn`s are ordinary today —
   `wat/cache.wat:55,64,77,85` are all `<K,V>`.

## ⚠ THE LOAD-BEARING SAFETY PROPERTY

**For a name with NO type params, the split MUST be the identity.** All nine existing concrete `defservice`s
must keep working byte-identically: `wat/query/sqlite-store.wat:243`, `wat/query/mem.wat:93`,
`wat/telemetry/span.wat:16`, `wat/telemetry/journal.wat:62`, `wat/kernel/services/stdio-primes.wat:{42,67,93}`,
and the macro-generated one at `wat/query.wat:338`. This is the regression surface — the whole floor rides on it.

## The gate — a minimal parametric service that CHECKS and RUNS

Recreate the probe (the orchestrator's version, with the three corrections the checker already taught us — keep
them, they are mandatory and unrelated to generics):

```clojure
(:wat::core::defsurface :probe::Box<T> :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Box::PutRequest [item <- :wat::core::i64])
   (:wat::core::defenum :probe::Box::PutResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64])]   ; ruling A — MANDATORY
  :features
  [(put [self <- :probe::Box<T>  req <- :probe::Box::PutRequest]
     -> :probe::Box::PutResponse :max-request-bytes 1024)])                   ; Stone 16.3 — MANDATORY

(:wat::service::defservice :probe::box-svc<T>
  :satisfies :probe::Box<T>
  :durable   [held <- :wat::core::Option<T>]
  :ephemeral []
  :impls
  [(put [s req] (:wat::service::Outcome::Reply s (:probe::Box::PutResponse::Ok)))])
```

Then go beyond `--check`: **stand the service up and round-trip one call** (`connect'` + one `put`), so we prove
it RUNS, not merely type-checks. Land the proof as a real `deftest` gate.

`target/release/wat --check <file>` is the fast per-file arbiter (~0.2s). **Read the output, not `$?` through a
pipe** — a pipe returns the last command's exit, not wat's.

## STOP triggers — halt and report, do NOT improvise

1. **If the cascade goes beyond name derivation into the WIRE PROTOCOL** (the `:satisfies` match against a
   parametric surface, the EDN codec, the generated client verbs) — STOP. That is a design ruling about whether a
   generic service is meaningful at an EDN boundary, and it belongs to the builder, not this strike.
2. **Generic MESSAGE records are OUT OF SCOPE.** A first probe declared `PutRequest<T>` in `:messages` and the
   checker rejected it — *"feature `put` references protocol type `:probe::Box::PutRequest` which is not declared
   in this surface's `:messages`"* — the reference resolved without the type argument. v1 keeps messages
   CONCRETE. If you believe generic messages are unavoidable, STOP and surface it.
3. **Do NOT "fix" a parametric record by making it concrete.** `docs/arc/2026/06/266-records-parametric-question/STUB.md`
   is marked CLOSED/REJECTED ("records stay concrete by purpose"), but HEAD contradicts it — `wat/cache.wat:49-51`
   ships `defrecord :wat::cache::Entry<K,V>` and it demonstrably works (Stone 1's gate constructs it and reads
   `Entry/key`). If a parametric record blocks you, STOP and report; that ruling needs revisiting by the builder.
4. If any of the nine existing concrete services changes behaviour — STOP. Identity-on-no-params is not negotiable.

## Blast radius

`wat/service.wat` (the name derivation + generated defns), possibly a small string helper, plus the new gate.
Nothing else. **STOP + report if it exceeds this.**

## Gate

- The parametric probe `--check`s clean AND runs a round-trip, landed as a `deftest`.
- All nine existing concrete defservices unchanged and green.
- `cargo build --release` clean; `cargo nextest run --release` — report the **Summary line VERBATIM**.
  Current floor: **4169 passed, 314 skipped**.
- Run everything FOREGROUND. **Do NOT commit** — the orchestrator weighs by their own re-run and commits.

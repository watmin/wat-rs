# BRIEF — stone S0: the `:wat::query` Store CONTRACT (surfaces + records + Error), core

> **Executor: one sonnet SHADOWDANCER.** Orchestrator drew this; weighs the kill against its own re-run. Work ONLY in
> `/home/watmin/work/holon/wat-rs/` (NEVER worktrees; `pwd` first — any `.claude/worktrees/` path is illegal, re-anchor).
> `cargo nextest run` (NEVER `cargo test`); `cargo wat <file>` to dogfood a wat file. Commit NOTHING — leave the tree.

## The work (one paragraph)

Create the **backend-agnostic storage contract** as a new **core** wat source, `wat/query.wat`, registered in
`src/stdlib.rs`. It declares the `:wat::query` narrow-waist vocabulary: the methods-bearing **`Store`** / **`ReadStore`**
surfaces, the **`Error`** recovery-axis enum + its **`Fault`** record, and the plain data **records** every satisfier
speaks (`StoredRow`, `IndexKey`, `Row`, `IndexRow`, `ScanRequest`, `IndexScanRequest`, `Page`, `IndexPage`,
`TableSchema`, `IndexSchema`). NO backend, NO logic — pure declarations (the sqlite satisfier is a *later* stone, S2).
This is the abstraction S1/S2/telemetry all hang on. `:wat::query` is **net-new + unprimed** (no battery collides); a
baked core source may define under `:wat::` (stdlib bypasses the reserved-prefix gate).

## The forms (from `DESIGN-store-contract.md § contract`, translated to keyword-FQDN; two pinnings applied)

Mirror the *exact* keyword-FQDN syntax proven in `probes/surface-field-dispatch.wat` (defsurface `:features
[(method [self <- :Surface  arg <- :T] -> :R)]`) and `probes/enum-holds-record.wat` (enum variants are `:Keyword`,
each carrying record fields).

- **`Error` + `Fault`** (recovery axis — variants are the caller's forced branch: retry / surface / abort):
  ```
  (:wat::core::defrecord :wat::query::Fault [op <- :wat::core::Keyword  code <- :wat::core::i64
                                             sql <- :wat::core::String   message <- :wat::core::String])
  (:wat::core::defenum :wat::query::Error :wat::enum::Pure
    :Transient  [fault <- :wat::query::Fault]
    :Constraint [fault <- :wat::query::Fault]
    :Fatal      [fault <- :wat::query::Fault])
  ```
- **The records** — one keyword-FQDN `defrecord` each, fields per `DESIGN-store-contract.md` lines 89–118. Parametric
  field types use the `(:wat::core::Vector :T)` / `(:wat::core::Option :T)` / `(:wat::core::HashMap :K :V)` forms.
  `StoredRow [pk sk data index-keys<HashMap String IndexKey>]` · `IndexKey [ipk isk]` · `Row [pk sk data]` ·
  `IndexRow [pk sk ipk isk data]` · `ScanRequest [pk sk-lo sk-hi limit cursor<Option String>]` ·
  `IndexScanRequest [index ipk isk-lo isk-hi limit cursor<Option String>]` ·
  `Page [rows<Vector Row> next-cursor<Option String>]` · `IndexPage [rows<Vector IndexRow> next-cursor<Option String>]` ·
  `TableSchema [pk sk]` · `IndexSchema [pk sk ipk isk]`. (all `pk/sk/…` are `:wat::core::String`.)
- **`Store` / `ReadStore` surfaces** — `:holder :wat::core::Struct` (a satisfier holds a live connection → impure).
  **PINNING: fallible methods return `(:wat::core::Result T :wat::query::Error)`** (errors-are-values), NOT a bare type:
  ```
  (:wat::core::defsurface :wat::query::Store :holder :wat::core::Struct
    :features [(ensure-schema [self <- :wat::query::Store  table <- :wat::query::TableSchema
                               indexes <- (:wat::core::Vector :wat::query::IndexSchema)] -> (:wat::core::Result :wat::core::nil :wat::query::Error))
               (put        [self <- :wat::query::Store  rows <- (:wat::core::Vector :wat::query::StoredRow)] -> (:wat::core::Result :wat::core::nil :wat::query::Error))
               (scan       [self <- :wat::query::Store  q <- :wat::query::ScanRequest]      -> (:wat::core::Result :wat::query::Page :wat::query::Error))
               (scan-index [self <- :wat::query::Store  q <- :wat::query::IndexScanRequest] -> (:wat::core::Result :wat::query::IndexPage :wat::query::Error))])
  (:wat::core::defsurface :wat::query::ReadStore :holder :wat::core::Struct
    :features [(scan       [self <- :wat::query::ReadStore  q <- :wat::query::ScanRequest]      -> (:wat::core::Result :wat::query::Page :wat::query::Error))
               (scan-index [self <- :wat::query::ReadStore  q <- :wat::query::IndexScanRequest] -> (:wat::core::Result :wat::query::IndexPage :wat::query::Error))])
  ```
  (If `(:wat::core::Result …)` is spelled differently in wat — grep `wat/core.wat` for the real `Result` form and use it;
  confirm before you commit to the annotation.)

## Read in order (the rooms)

1. **`DESIGN-store-contract.md`** — § contract (the ratified record shapes, lines 65–118) + § Naming (names are ratified;
   do NOT rename). The two pinnings above (Error-in-contract, Result-wrapping) supersede the doc's `-> Ok`/`-> Page`.
2. **`probes/surface-field-dispatch.wat`** — the WORKING `defsurface … :features [(method …)]` + `extend-type` +
   dispatch-through-a-field syntax (→ 142). Mirror it exactly; your `Store` is the same shape, more methods.
3. **`probes/enum-holds-record.wat`** — the WORKING enum-of-records (`:Variant [field <- :Record]`). `Error` mirrors it.
4. **`wat/core.wat`** — grep a real `defrecord` with parametric fields + the real `Result` / `Option` / `Vector` /
   `HashMap` type forms; copy the exact spellings.
5. **`src/stdlib.rs`** — the baked `include_str!` list (order = dependency order; `verify-stdlib` enforces it). Add
   `wat/query.wat` AFTER `wat/core.wat` (defsurface/defrecord/defenum/extend-type + Result/Option/Vector) and near the
   rete sources (it is the query engine's vocabulary). Its only outward refs are `:wat::core::*` + `:wat::enum::Pure`.

## Where it lands (bounded blast radius)

- New core source **`wat/query.wat`** (the declarations above — NO logic, NO backend).
- **`src/stdlib.rs`** — ONE new list entry for `wat/query.wat` at the right dependency position.
- A **`deftest'` gate** (co-located per current standard — a `wat-tests/` file or a `.wat` deftest') that: constructs a
  `StoredRow` + a `ScanRequest` + a `Page`; defines a tiny in-file `defstruct` satisfier, `extend-type`s it to
  `:wat::query::ReadStore`, and **dispatches `scan`** through it returning a `Page`; asserts the round-trip. This proves
  the surfaces + records + dispatch are real. (Mirror `probes/surface-field-dispatch.wat` for the extend+dispatch shape.)

## STOP triggers (rejection criteria — surface, don't hack)

- **STOP-RESERVED:** if `:wat::query::` is rejected at registration (reserved-prefix gate) even from a baked stdlib
  source, STOP and report — do NOT fall back to a non-`:wat::` namespace. (Baked sources should bypass the gate; if not,
  it's a substrate finding for the orchestrator.)
- **STOP-RESULT:** if `(:wat::core::Result T E)` is not the real annotation form for a fallible return, STOP and report
  the real form — do NOT invent one or drop the error channel.
- **STOP-SURFACE-METHOD:** if a surface method returning a `(:Result …)` (or taking a parametric-typed arg) fails to
  parse/register, STOP and report — do NOT simplify the signatures to make it pass.
- **STOP-DEPORDER:** if `verify-stdlib` rejects the placement, move the entry per its message; do NOT disable the check.

## The gate (EXPECTATIONS)

| what | command | expected |
|---|---|---|
| the contract loads (stdlib boots with query.wat) | `cargo wat <a tiny wat constructing a :wat::query::StoredRow>` | prints its EDN, no error |
| the acceptance deftest' green | `cargo nextest run --release -E 'test(query_contract)'` (or the deftest's runner) | 1+ passed |
| whole floor | `cargo nextest run --release` | `0 failed` (modulo the known arc-290-300 `no_inlined_wat_in_tests` reminder) |

Runtime ~30–45 min (stdlib change → release rebuild). Trap-door: the `Result`-wrapping annotation form + the parametric
field-type spellings — grep `wat/core.wat` for the real ones before committing; the compiler names any mismatch in one shot.

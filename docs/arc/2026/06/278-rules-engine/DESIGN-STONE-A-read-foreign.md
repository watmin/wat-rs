# DESIGN-STONE-A — `read-foreign`: the dynamic EDN decode (the keystone)

> **Parent:** `DESIGN-dynamic-edn-decode-and-opaque-sink.md` (the campaign). This is Stone A, drawn against the
> **live disk at HEAD `a5a48aa1`** (A.0 committed `c9bfa8fd` — the uniform variant encoding floor is clean).
> Realizations: R45 `LVCEM TENEBRASQVE FERO` (the Onyx — bear the KNOWN and the UNKNOWN in one) + R46 `IN LVCE
> PVRGATI` (the floor wiped). Stone A is the substrate learning to **bear the unknown** on that clean floor.

## Why (the grounded blocker)
The telemetry sink must accept logs from **arbitrary callers** and let **arbitrary callers process** them; a forked
`journal'` child faults `UnknownTag` decoding a user payload whose type isn't in its baked registry
(`probe_arc278_journal_logs_on_process` is `#[ignore]`'d). The **sink** stays opaque (Stone B — never decodes,
`feedback_sink_is_opaque_store_consumer_decodes`); the **consumer** owns the decode. When the consumer HOLDS the
type → strict `read` → a typed value. When it LACKS the type → `read-foreign` → a **self-describing dynamic value**.
That dynamic-decode capability is Stone A — general substrate, benefits everything that reads possibly-foreign EDN.

## The one contract decision (PINNED — the interface choice)
`edn::read` gains an opt-in **DATA MODE**, surfaced as a sibling verb `:wat::edn::read-foreign`. Strict
`:wat::edn::read` is **UNCHANGED** — unknown tag → `UnknownTag` error (catches typos; holds the no-hidden-failures
floor, R41 `EGO SVM LEX`). A silent global default would turn a typo'd tag into a masked dynamic value — forbidden.
Strict-typed vs data-dynamic **is** the builder's "the querier decides."

**The dynamic values (names ratified-by-use — R45's UX forms, builder-praised; NOT re-cast):**

| body shape (post-A.0, total discriminator) | strict (unchanged) | `read-foreign` (new) |
|---|---|---|
| map `{…}` under `#ns/Type` | record/struct; `UnknownTag` on unregistered | **`:wat::edn::ForeignRecord {class, name-keyed fields}`** |
| vector `[…]` under `#<enum-path>/<Variant>` | enum tagged-variant; `UnknownTag` | **`:wat::edn::ForeignVariant {enum-class, variant, positional fields}`** |
| `nil` body | the unit value (A.0) | unchanged — `nil` is the unit value, never a variant |

- **Self-describing:** the tag is fully qualified (`#ns/Type` record / `#<enum-path>/<Variant>` variant), so the
  class/variant + fields are all present — **no registry lookup, no registry mutation.**
- **Recursive:** a dynamic value's fields decode the same way — a `ForeignRecord` *containing* a `ForeignVariant`
  field (or a variant carrying a foreign record) decodes all the way down. (A.0's uniform encoding is what makes
  this symmetric — vector bodies are variants at every depth.)
- **Re-serializes faithfully:** back to the same tag + body (store / forward / re-query round-trip).
- **One shape per fully-qualified tag, contradiction = exception:** body-shape is already a total discriminator
  (A.0), so within a decoded document a tag is consistently record-vs-variant; a tag appearing as *both* a map and
  a vector body is the contradiction → exception (honest). Enum variants are distinct tags (`/A` vs `/B`), so a
  multi-variant enum does **not** false-contradict. Cross-document consistency is the consumer's concern (it holds
  the values). **No separate tag→shape registry needed** — the body-shape dispatch enforces it structurally.

**Accessors (the consumer navigates DATA, not a typed value):**
```clojure
;; consumer HOLDS the type → strict read, TYPED value, typed accessor (unchanged):
(:wat::core::let [action (:wat::edn::read msg)]        ;; → :app::UserAction (typed, checked)
  (:app::UserAction/verb action))
;; consumer LACKS the type → read-foreign, a FOREIGN value navigated as DATA (get-by-key):
(:wat::core::let [fr (:wat::edn::read-foreign msg)]    ;; → :wat::edn::ForeignRecord
  (:wat::edn::ForeignRecord/get fr :verb))             ;; navigate by key; you don't hold the type
;; nested — a foreign record CONTAINING a foreign variant field (auto, recursive):
(:wat::edn::ForeignVariant/variant                     ;; → :Click
  (:wat::edn::ForeignRecord/get fr :kind))
```
`ForeignRecord/get` (key → field value), `ForeignRecord/class` (the fqdn), `ForeignVariant/variant` (the variant
keyword), `ForeignVariant/enum-class`, `ForeignVariant/fields` (positional vector). Exact accessor set + return
types are the shadowdancer's to finalize against the surface machinery; these are the ratified shape.

**Representation (the pinned Rust choice — the decision worth your eyes):** `ForeignRecord`/`ForeignVariant` are
**first-class dynamic values with a baked `:wat::edn::` surface + accessors**, carrying their fully-qualified
class/variant string + fields (name-keyed map / positional vector), so they re-serialize by writing the tag back.
They are pure data (records-are-EDN, arc 300) — they satisfy the opaque-carriage + round-trip contract. Whether
the Rust carrier is a new `Value` variant vs a baked aggregate is the shadowdancer's implementation call, grounded
against how `Value`/aggregates are represented today; the CONTRACT (fqdn class + fields, re-serializable, recursive)
is fixed here.

## The mechanism (grounded, file:line)
- **`read-foreign` verb:** a sibling of `eval_edn_read` (`edn_shim.rs:183`) — same String→parse path, threads the
  foreign mode into the decode.
- **Mode threading:** a `foreign` flag (or a small `DecodeMode`) rides the decode call-chain **beside the existing
  `allow_caps: bool`** — already threaded through `edn_to_value_caps` → `tagged_to_value` → `reconstruct_struct`
  (`:2457`) / `reconstruct_record` (`:2515`) / `reconstruct_enum_tagged` (`:2685`). STOP-CASCADE: no new param
  through the world; it rides the plumbing already in scope (the reserved-privilege / PRIMVS VSVS lesson).
- **The reroute (data-mode ONLY):** at the three `UnknownTag` misses — `:2470` (struct), `:2529` (record), `:2697`
  (enum-variant) — when the tag doesn't resolve to a registered type AND foreign-mode is on, build a
  `ForeignRecord` (map body) / `ForeignVariant` (vector body) from the fully-qualified tag + the recursively-decoded
  fields, instead of erroring. Strict-mode behaviour at these sites is untouched.
- **The `match body` dispatch** (`tagged_to_value:2359`) stays the total discriminator A.0 made it (map/vector/nil);
  foreign-mode changes only what happens on an *unregistered* tag, not the shape routing.

## RED gate (the disconfirming probe — draw + commit BEFORE the brief)
`tests/…/probe_arc278_read_foreign.{rs,wat}`. Assert BOTH:
1. **`read-foreign` reconstructs a foreign record CONTAINING a foreign variant field** — an EDN string
   `#some.unknown/Rec {:kind #some.unknown.Kind/Click [42]}` (a tag not in the registry, whose field is itself an
   unregistered enum-variant vector body) round-trips through `read-foreign` to a `ForeignRecord` whose `:kind` is a
   `ForeignVariant{variant :Click, fields [42]}` — navigated by the accessors, recursive all the way down; and
   re-serializes back to the same tag + body.
2. **strict `read` on the same input STILL ERRORS** `UnknownTag` (the no-hidden-failures floor held — R41).

At HEAD both fail (no `read-foreign`, no dynamic values). GREEN when the mode + the two dynamic-value types land.

## Scope
- **IN:** unknown *user* tags (record map / enum-variant vector), **data-mode only**; the two dynamic values +
  their accessors + faithful re-serialization + recursion.
- **OUT (rejected, not deferred):** stdlib tags, `Option`/`Result`, `#inst`, all registered types (unchanged both
  modes); strict mode's unknown-tag error (the floor — untouched); the sink (Stone B — never decodes); the
  annihilation/fold/de-prime (Stone C).

## Sequencing
Stone A is the keystone; B applies it (opaque sink + un-`#[ignore]` the journal probe), C annihilates the legacy.
Each stone: DESIGN + RED probe → brief → delegate a shadowdancer → weigh by own re-run.

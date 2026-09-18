# ward `doc-comment-contradiction-sweep` — vigilia 2026-09-05

> Cast at HEAD `21530efab`, branch `grok-rete`. Read-only unless the ward says otherwise.
> **Verbatim ward return, preserved unedited.** The 2026-08-30 cast lost its 19 returns
> because they lived only as subagent messages; this file exists so that cannot recur.
> Nothing here is adjudicated — see `../WORK-LIST.md` for what the orchestrator credited.

---

I read both the doc and the body for every item below. Ten confirmed contradictions, strongest first.

---

## 1. `src/runtime.rs:21631` — an entire doc block is attached to the wrong function (mid-sentence split)

**Doc claim** (lines 21631–21643, sitting directly above `bundle_capacity_verdict`):
> ``/// `(:wat::holon::Bundle <list-of-holons>)` — superposition, with Kanerva-capacity enforcement…``
> ``/// Return type is `(:Result :- [:wat::holon::HolonAST :wat::holon::CapacityExceeded])`. Always. Under every mode. Callers are forced by the type system to acknowledge the possibility of failure…``

**Falsified by** `src/runtime.rs:21661`:
```rust
pub(crate) fn bundle_capacity_verdict(cost: usize, ctx: &EncodingCtx) -> Option<(i64, i64)> {
```
This is not the `Bundle` verb and returns `Option<(i64,i64)>`, never a `Result`. The Bundle doc ends mid-sentence at 21642 (`"…statement exactly. There is"`), then 21643 abruptly starts the real `bundle_capacity_verdict` doc (`"Arc 294.c.2a — Kanerva width-bound verdict…"`). The severed tail reappears at **21672** — `/// no codebook factor — under AST-primary…` — as the *opening line* of the doc for `fn eval_algebra_bundle` (21688), whose header therefore begins with a sentence fragment and never states what the function is.

**Reader gets wrong:** rustdoc renders `bundle_capacity_verdict` as the wat `Bundle` verb with a `Result` return and a type-system failure contract; the actual `Bundle` verb has no header at all.

---

## 2. `src/value/numeric_order.rs:3, 11, 24, 161` — "three call sites", but there are four, and the fourth is the hot one

**Doc claims:**
- L3: `//! Three call sites (`runtime.rs::values_compare`, `runtime.rs::walk_match_clause`'s `RawClause::Compare` arm, `rete/matcher.rs::compare_values`)`
- L11: `//! Each of the three callers owns its own policy…`
- L24: `/// Three outcomes, because the three callers have three different policies…`
- L161: `//` …`callers 2 and 3 are unreachable through the checked path per the design stone's reachability ruling, so this table-level test is their only executable regression coverage.`

**Falsified by** `src/runtime.rs:11833`, inside `fn eval_compare` (declared 11803) — a fourth, unlisted caller:
```rust
let result = match crate::value::numeric_order::numeric_order(&a, &b) {
```
`eval_compare` is dispatched from `runtime.rs:6204/6207/6210/6213/6235…` for `:wat::core::<`, `>`, `<=`, `>=`, `i64::>` etc. — i.e. the ordinary user-facing comparison operators.

**Reader gets wrong:** "only caller 1 is reachable from checked wat; 2 and 3 have no end-to-end coverage" — in fact the most-exercised path in the language (`<`/`>`/`<=`/`>=`) routes through this table, with its own fourth policy (`Incomparable ⇒ false`, `NotNumeric ⇒ fall back to values_compare`).

---

## 3. `src/rete/vocabulary.rs:293-295` — "the five `Redispatch` rows" carrying `&[]`; there are 13 and four are non-empty

**Doc claim** (on field `type_params`):
> ``/// `&[]` on every row that does not need one (all pre-existing rows, plus the five `Redispatch` rows, which carry no scheme at all).``

**Falsified by** the table itself:
```rust
// vocabulary.rs:811-818
ReteOp { type_params: &["T"], rete_name: ":wat::rete::core::PersistentVector", …
         class: OpClass::Redispatch, params: &[], …
// :839  ":wat::rete::core::PersistentMap"  type_params: &["K", "V"], class: Redispatch
```
`grep -c "class: OpClass::Redispatch"` = **13**, and `PersistentVector` / `Vector` / `List` carry `&["T"]`, `PersistentMap` carries `&["K","V"]`.

**Reader gets wrong:** believes `Redispatch ⇒ type_params == &[]` is an invariant and could gate on `class` when deciding whether to read `type_params`.

---

## 4. `src/runtime.rs:1956` (and `src/value/value.rs:201`) — synthesized enum-ctor body named as a form that doesn't exist

**Doc claim** on `register_enum_methods`:
> ``///   - Body `(:wat::core::enum-new :my::ns::E :Variant f1 f2 ... fn)``

**Falsified by** `src/runtime.rs:2028-2031`, which builds the body:
```rust
// Body: (:wat::core::variant :enum-path :Variant p1 p2 ... pn)
body_items.push(WatAST::Keyword(":wat::core::variant".into(), …));
```
`:wat::core::enum-new` occurs nowhere in `src/` or `wat/` outside these two doc comments; `:wat::core::variant` is the live head (dispatched at `runtime.rs:5789`). `src/value/value.rs:201` repeats the stale name: *"an auto-synthesized Function entry whose body calls `:wat::core::enum-new`"*.

**Reader gets wrong:** greps for a head that no longer exists when tracing how `(:E::Variant a b)` evaluates.

---

## 5. `src/config.rs:377` — "Enforces: Required fields (`dims`, `capacity-mode`) set" — nothing requires them

**Doc claim** on `collect_entry_file`:
> ``/// - Required fields (`dims`, `capacity-mode`) set; `global-seed` defaults to 42 if unset.``

Repeated on the error variant at **`src/config.rs:229-231`**:
> ``/// A required field was not set. `global-seed` is optional (defaults to 42); `dims` and `capacity-mode` are required.``
> `RequiredFieldMissing { field: String },`

**Falsified by** `collect_entry_file_inner`:
```rust
// config.rs:412
let mut dim_count: usize = inherit.map(|c| c.dim_count).unwrap_or(DEFAULT_DIM_COUNT);
// config.rs:715-719  "Arc 037 slice 6: all setters optional. capacity-mode defaults to :error…"
let capacity_mode = capacity_mode.unwrap_or(DEFAULT_CAPACITY_MODE);
```
`RequiredFieldMissing` is **never constructed** anywhere in the tree — only declared (231) and rendered (275). The module doc at `config.rs:30-35` says the opposite of line 377 ("As of arc 037: every field has a default… Empty entry files commit a fully-defaulted Config").

**Reader gets wrong:** believes an entry file omitting `set-dims!` is rejected; it silently gets 10000.

---

## 6. `src/rete/validate/typing.rs:544` — "THREE outcomes, all explicit"; the enum has five variants

**Doc claim:**
> ``/// What an operand's type resolution actually yielded. THREE outcomes, all explicit — there is no `Option` here on purpose.``

**Falsified by** `enum OperandType` at 552: `Resolved` (554), `NotComparable` (557), `UnboundInThisRule` (562), `ComputedNotDerivableHere` (584), `MistypedEnumVariant` (594) — **five**. The header count was never updated when the last two were added, and the variant docs know it: line 570 says *"The three sources this function documents as 'exhaustive' were written before fix-list F made a nested call a legal operand, and nothing re-read them."*

**Reader gets wrong:** a header that says the split is three-way, on a type whose entire purpose is that the split is fine-grained.

---

## 7. `src/runtime.rs:9727-9749` — "Three canonical shapes" for a four-variant `LetBinding`

**Doc claim:**
> `/// Three canonical shapes — all honest about types.` … three bullets (`Single`, `Destructure`, `StructDestructure`) … `/// Arc 233 Stone 233.2.e: added per-name spans to all three variants…`

**Falsified by** `enum LetBinding<'a>` (9750) which has **four** variants — `Single`, `Destructure`, `StructDestructure`, and `HashDestructure` (**runtime.rs:9775**), the last being "Receiver-polymorphic over `Value::Aggregate` (all natures) and `wat__std__HashMap`" and completely absent from the header.

**Reader gets wrong:** believes `let` has three binder shapes and that a map-destructure binder is unsupported.

---

## 8. `src/freeze.rs:1229-1231` (and `src/config.rs:390-391`) — caller list naming annihilated verbs

**Doc claim** on `startup_from_forms_with_inherit`:
> ``/// Called by `:wat::kernel::run-sandboxed-ast`, `:wat::kernel::run-sandboxed-hermetic-ast`, and `:wat::kernel::spawn-process` children — each passes the active runtime's [`Config`] as `inherit`.``

**Falsified by** `src/runtime.rs:6901-6903`:
```rust
// Arc 105c — substrate `:wat::kernel::run-sandboxed` /
// `-ast` dispatch arms are GONE.
```
Neither `run-sandboxed-ast` nor `run-sandboxed-hermetic-ast` exists as a dispatchable head anywhere in `src/`. The only in-tree caller is `src/process/verbs.rs:405`. `src/config.rs:390-391` carries the identical stale list for `collect_entry_file_with_inherit`.

**Reader gets wrong:** goes looking for two verbs that were deleted, and misses that the process-fork path is the sole live consumer.

---

## 9. `src/check.rs:395-397` — documented `args(callee, expected, got)` on a one-parameter function

**Doc claim:**
> `/// Arguments match the `TypeMismatch` variant's field idents (passed by the derive as `&String` references from the destructured pattern):`
> ``/// `args(callee, expected, got)`.``

**Falsified by** the signature immediately below (398-400):
```rust
pub(crate) fn type_error_remedies_via(
    callee: &str,
) -> Option<wat_edn::OwnedValue> {
```
and by the only use site, `src/check/error.rs:99`:
```rust
#[to_edn(via(key = "remedies", fn = crate::check::type_error_remedies_via, args(callee)))]
```

**Reader gets wrong:** copies the documented three-arg `args(...)` incantation onto a new variant and gets an arity error; also believes `expected`/`got` participate in remedy computation, which they do not.

---

## 10. `src/edn_shim.rs:3713` (and `:18`) — intra-doc links to functions that were deleted/renamed

**Doc claim** on `edn_string_to_value` (3712-3713):
> ``/// Decode a compact EDN `String` back to a `Value` — the inverse of [`value_to_edn_string`].``

**Falsified by** `src/edn_shim.rs:3657`, thirty lines above:
```rust
// ⛔ `value_to_edn_string(v)` — the types-less door — is DELETED (2026-08-14).
```
The live functions are `value_to_edn_string_with` (3705) and `value_to_edn_string_lossy` (3691). The module doc's "The walker" section, **`src/edn_shim.rs:18`**, has the same problem: ``[`value_to_edn`] converts a wat `Value` into a `wat_edn::OwnedValue``` — no `value_to_edn` exists; it is `value_to_edn_with` (3845). Both are broken rustdoc links pointing at the pre-`types`-threading API.

---

### Lower-confidence, verified but weaker (grouped)

- **`src/runtime.rs:23109` and `:23169`** — `presence?`/`coincident?` docs say the verdict is `cosine > :wat::config::presence-floor` / `(1 - cosine) < :wat::config::coincident-floor`. No such keywords exist; the ops are `:wat::holon::presence-floor` / `:wat::holon::coincident-floor` (`runtime.rs:6696-6697`), and the config surface is `set-presence-sigma!`. Wrong namespace, right concept.
- **`src/types.rs:197-198`** — ``/// `Nature` is … the EDN capability trit. Three variants:`` followed by a three-item list; the enum has **four** (`Peer`, added arc 293 S3-Nature-2, `types.rs:211`). Self-corrected two lines later, so a careful reader survives; a skimmer does not. Also `check.rs:13773-13774` still spells the discriminator `TypeDef::Aggregate(kind=Record|HolonRecord)` where the field is `nature` (`types.rs:310`), and that classification list omits both `Nature::Peer` and the `TypeDef::Surface` arm the body handles (`check.rs` `is_pure_type`).
- **Stale "live witness" test paths**: `src/hash.rs:63-64` points at `tests/probe_hash_scope_renumber.rs`; the file is `tests/macros/probe_hash_scope_renumber.rs` (there are no top-level `.rs` files under `tests/` at all). Same shape in `src/collection/mod.rs:21-31`, `src/function/mod.rs:35-46`, `src/argspec/mod.rs:51`, `src/config.rs:89` — each names a `tests/probe_*.rs` that does not exist at the stated path.

### Checked and found honest (so you don't re-tread)

`remedy/rank.rs` + `remedy/mod.rs` sort/tie-break claims (manual `Ord` really is score→form); `value/pmap.rs` promote/never-demote/entry-set-`Eq`/`Hash` claims and `extend`'s "zero pairs never clones"; `rete/kernel/arm.rs::merge_sorted_ids`; `rete/kernel/node.rs::sorted_node_ids`; `rete/validate/mod.rs:1270` sole-caller-guard `unreachable!`; `rete/kernel/stratify.rs` one-caller claim; `rete/matcher.rs:978` one-caller claim; `check.rs` `CheckResult` three-variant / no-`Silent` claims; `rete/kernel/fire/acc.rs::acc_var_i64` "never panics"; `rete/kernel/fire/mod.rs::key_of_el` "common cases never allocate"; `string_ops.rs::is_canonical_uuid_string`; `collection/seq_container.rs::of_value`; `intrinsic/bytes.rs::decode_nibble`.

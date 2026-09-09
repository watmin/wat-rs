# TABLE — STONE A-2 RELAND-5: the 13 `Value::Aggregate(a)` sites in `src/runtime.rs`

Census taken before any fix, per the brief. `grep -n "Value::Aggregate(a)" src/runtime.rs` on the
tree as found (before this stone's edits) returned exactly 13 lines. Classified below by whether the
site is a field-READING operation the checker now admits a variant to (per Stone A-2 RELAND-1's
widened `{:keys}`/accessor predicate), or something else (construction, transport, nature/purity,
type-identity dispatch) where a variant does not belong per the same shape argument.

| # | Line (pre-fix) | Site | Classification | Verdict |
|---|---|---|---|---|
| 1 | 3499 | Surface-method dispatch: derive `concrete_type_fqdn` from the receiver to look up `:<T>/<method>`. | **NOT field-reading** — type-identity for method dispatch. The `other_val` catch-all already runs for `Value::Enum` and calls `other_val.type_name()`, which returns the *generic* `"wat::core::Enum"` rather than the declared enum FQDN (`declared_type_name()` would give the right answer). | **Pre-existing gap, orthogonal to this stone — flagged, NOT fixed.** This is a dispatch/nature-identity defect (STOP-4's "a variant arm needed for a non-field-reading reason"), not the "carries named fields" predicate this stone closes. Fixing it is a different mechanism (`type_name()` vs `declared_type_name()`) and predates A-2 (enums have always been `Value::Enum`); rebuilding it here would be scope creep on a stone whose acceptance rows don't touch surface methods on enum values. |
| 2 | 3667 | Keyword-as-accessor fall-through (record path), inside the `match receiver` block guarded by `a.nature != Nature::Struct`. | **Field-reading.** Checker's own `acceptable` match (`src/check.rs` ~5686-5714) now must admit a singleton-variant `TypeExpr::Path` the same way it admits a registered `Aggregate` — this is target #1 named in the brief (measured: `"unknown callee: :inside"`). | **FIXED.** Added `Value::Enum(e) => keyword_accessor_enum(...)` arm (reads `e.names`/`e.fields` directly — no TypeEnv lookup). Checker widened in lock-step (see below). |
| 3 | 3677 | Same match block as #2 — the `Value::Aggregate(a)` struct-nature catch-all. | Same match block, same fix. | Counted with #2 (one added arm covers both). |
| 4 | 4035 | `LetBinding::StructDestructure` (`{:keys [...]}`) — evaluate rhs, require `Value::Aggregate`. | **Field-reading.** Target #2 named in the brief (measured: check=0, run=1, `"expected an aggregate type, got wat::core::Enum"`). Checker side (`process_let_binding`'s `Keys` arm) was already widened in a prior stone (RELAND-1) to accept a singleton `TypeDef::Enum`; only the runtime side was missing. | **FIXED.** Restructured to branch `Value::Aggregate` (TypeEnv-backed, unchanged) vs `Value::Enum` (reads `e.names`/`e.fields` directly, no lookup — brief's point 5) into a common `(class_label, declared_names, values)` shape, then one shared lookup loop. |
| 5 | 4121 (now 4168) | `LetBinding::HashDestructure` (`{var :field ...}`) — record-nature arm, reuses `keyword_accessor_record`. | **Field-reading**, same shape as #2/#3 (literally reuses the same helper). The checker's `MapDestructureKind::Hash` arm (`process_let_binding`) never validated field existence against ANY receiver type (Aggregate, HashMap, or otherwise) — every binding gets a fresh type var unconditionally — so it was already maximally permissive; there is no separate "admits a variant" predicate to duplicate (STOP-5 does not fire). | **FIXED** — added `Value::Enum(e) => ...` arm using `keyword_accessor_enum`, for consistency with the rule ("every field-reading operation must accept one") and because leaving it as a `TypeMismatch` would reproduce exactly this stone's defect (checker silently permits, runtime refuses) one form over. |
| 6 | 4138 (now 4185) | Same match block as #5 — the struct-nature catch-all. | Same match block, same fix. | Counted with #5. |
| 7 | 8992 (now ~9051) | `try_match_pattern`'s `WatAST::Map` arm — `{var :field ...}` as a match-arm sub-pattern — record-nature arm, reuses `keyword_accessor_record`. | **Field-reading**, identical shape to #5/#6 at a different call site (match position instead of let position). The checker's `MatchArm::HashDestructure` arm (`infer_match`) is likewise unconditional (fresh var per binding, no receiver check) — same "already maximally permissive" argument. | **FIXED** — added `Value::Enum(e) => ...` arm using `keyword_accessor_enum`. |
| 8 | 9009 (now ~9068) | Same match block as #7 — the struct-nature catch-all. | Same match block, same fix. | Counted with #7. |
| 9 | 9082 | Doc comment (`/// - Value::Aggregate(a) → a.class ...`), not code. | N/A. | No change (not a site). |
| 10 | 9721 | `conforms_check`'s nominal check for the `wat::core::Record` / `wat::holon::Record` umbrella supertype: `matches!(value, Value::Aggregate(a) if a.nature != Nature::Struct)`. | **NOT field-reading** — a nature/identity predicate ("is this value a Record"). A tagged variant is an `Enum`, not a `Record`; answering `false` for `Value::Enum` here is *correct*, not a gap. | No change — out of scope by STOP-4 (nature/purity check, not field access). |
| 11 | 9741 | `conforms_check`'s fallback for a type name that is neither in the `TypeEnv` nor a built-in primitive: `Value::Aggregate(a) => a.class.as_ref() == stripped`, else `Err`. | **NOT field-reading.** This branch only runs for names *absent* from the `TypeEnv`; since A-2's `register_variant_types` registers every variant (including singletons), a variant's own type name never reaches this `None` arm in the first place. | No change — the population this arm serves does not include registered variants. |
| 12 | 14342 | `reply_failed_reason`'s decode of a `*::Reply::Failed[cause]` — `cause.fields[0]` is asserted to be the `:wat::kernel::Failure` record. | **NOT general field-reading** — transport/decode of one fixed, known-always-a-record internal shape (`Failure` is a `defrecord`, never a variant). | No change — the value being read is never an `Enum` by construction. |
| 13 | 14618 | Test code (`cache_probe_startup_error_is_navigable_edn_not_string`) asserting a `RuntimeError` record decodes to `Value::Aggregate`. | Not a runtime "site" at all — a test assertion on a fixed internal shape. | No change (not a site; test-only). |

## Count

- **13** total `Value::Aggregate(a)` sites censused (matches the brief's count exactly).
- **4 distinct fix locations** needed widening (8 of the 13 lines, since 4 of the "sites" are the
  second arm of a two-arm match already being touched): the keyword-accessor fall-through (#2/#3),
  the `{:keys}` `StructDestructure` (#4), the let-binding `HashDestructure` (#5/#6), and the
  match-arm hash sub-pattern (#7/#8).
- **1 site flagged and NOT fixed** (#1, surface-method dispatch) — a real but pre-existing, orthogonal
  defect per STOP-4 (a non-field-reading reason: type-identity for dispatch, via `type_name()` instead
  of `declared_type_name()`), reported to the orchestrator rather than folded into this stone.
- **4 sites classified as correctly out of scope** (#9 comment, #10/#11 nature checks, #12/#13
  fixed-shape transport/test code) — no change.

4 fix locations is well under the ~6 STOP-3 threshold, so no stop was triggered; all four were fixed.

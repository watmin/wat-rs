# SCORE — STONE N RELAND 4: the wake

No commit. Floor left to the orchestrator. Lands on N RELAND-3's tree.

## Class 1 — golden drift. Recaptured. Pin kept.

**peers_bijection ×4.** `UPDATE_EDN=1`. Diff is **only** `:end :col 51 → 67` on the fixture-file span (the `defservice` impl's end). RELAND-2 wrap of `EchoResponse::Ok {:reply …}` lengthened that form. Not a `wat/service.wat` line this time. Same error kind, same reason text. Recaptured.

**pprintln_doc_row.** Byte-golden. Diff is **one token** in the example result: `:rule :wat.core/None` → `:rule :wat.core.Option/None`. RELAND-3 qualified Option; the printer now names `Option/None`. Recaptured. `ordinary_multiline_string_stays_escaped` was already green.

**metadata_of_example_formats.** Was **not** golden drift. It died at startup on rete `RhsArityMismatch` for `BreakKind::{Align,Block}` — class 3. After the rete unit-map fix it **PASS**ed with no recapture.

`normalize_rust_source_span_lines` was **not** extended (STOP-2).

## Class 2 — residual positional. Wrapped / hand-edited.

| site | what | why wrap didn't (or did) |
|---|---|---|
| `probe_arc265_acronym_registry_svc.wat` | `Op::CreateWebACL {:req …}`, `Response::Ok {:value …}` | generated surface enum; wrap would scream UNRESOLVED (same as dead_child Echo::Op before defservice-stripped retry). Hand-edited. |
| `probe_let_splice_enum_constructor.wat` / `probe_do_splice_enum_constructor.wat` | `Request::Push {:value 99}` | `defenum` lives **inside** `let`/`do`, not a top-level decl. Same class as RELAND-3's defmacro body. Hand-edited. |

`grep_smoke_target.wat` had been over-migrated. Restored parse-only **census bait** `(:wat::core::Some 1)` / `Ok` / `Err` so `bare-variant-constructors.wat` still matches. Not a type-checked program.

## Class 3 — arity, not spelling. Diagnosed. Rete map-ctor.

`constructor :wat::grep::NodeKind::Keyword wants 0 fields, got 1` (and the same for `BreakKind::{Align,Block}` in fmt `:then`).

**Not** a wrap that put a key in `{}`. The empty map **is** the unit spelling (`(:Keyword {})`). Rete's `lower_construct` / `walk_nested_constructors` counted `items[1..]` positionally, so `{}` was one argument against unit arity 0.

Fix (named, not a wall softening):

- `rete_enum_unit_arg_count`: a single empty Map is 0 fields
- tagged: unpack `(:E::V {:k v})` in declaration order (`parse_key_first_pairs`)

`nested_surface_enum_variant_correct_arity` (already map-form `{:level 7}`) and metadata_of / fmt-loading tests: **PASS**.

## Class 4 — error-text assertions. Wall was right. Fixtures were stale.

| test | old error expected | what it got | verdict |
|---|---|---|---|
| `try_with_zero_args_rejected_at_check` | `ArityMismatch` on `Result/try` | bare `:wat::core::Ok` retired | Wall **correct**. Fixture's Ok was incidental. Updated to `Result::Ok {:value (Result/try)}` so the test still measures try arity. |
| `match_arm_type_mismatch_named_by_arm_index` | TypeMismatch match arm #2 | bare `:wat::core::Some`/`None` + Open non-exhaustive | Wall **correct**. Updated fixture to `Option::Some`/`None` so arm #2 String-vs-i64 is what fires. |
| `defservice_consults_its_namespace_acronyms` | startup success | positional `Op::` / `Response::Ok` | Class 2, not a wrong wall. Wrapped (above). |

No wall softening (STOP-5).

## Probe / clippy

- `probe_arc296_enum_map_ctor`: **5 passed**
- `cargo clippy --release --all-targets --workspace`: **0 errors**, 5 pre-existing dead-code warnings

## Floor

**Not run.** Orchestrator. Pre-strike (central): 69 failed.

Delta per class (this strike's tests, not the floor):

| class | before | after |
|---|---|---|
| (1) golden | 7 named | recaptured / metadata was (3) |
| (2) positional | arc265, let/do splice | wrapped |
| (3) unit `{}` arity | NodeKind / BreakKind | rete understands unit/tagged maps |
| (4) error text | 3 named | fixtures legal except the intended defect |

## STOP rows

| STOP | result |
|---|---|
| STOP-1 recapture without checking what moved | **held.** col 51→67 (wrap length); `:wat.core/None`→`Option/None` (print) |
| STOP-2 normalize extended to wat/*.wat | **held.** |
| STOP-3 arity folded into migration | **held.** Named rete positional-count vs unit map |
| STOP-4 error-text updated without asking | **held.** Wall was right; fixtures updated to keep the measured error |
| STOP-5 wall weakened | **held.** |

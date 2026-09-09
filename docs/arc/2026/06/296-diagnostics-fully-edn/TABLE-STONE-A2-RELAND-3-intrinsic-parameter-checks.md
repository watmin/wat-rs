# TABLE — STONE A-2 RELAND-3: census of `src/check.rs`'s `infer_*` parameter checks

72 `fn infer_*` functions in `src/check.rs` (grep `^fn infer_\|^pub(crate) fn infer_`, excluding the
one `#[test] fn infer_rete_form_names_...` match). For each: does it compare an argument's inferred
type against a declared/expected type at all; if so, via `unify` (exact, no subtype/widening
awareness) or `assignable` (the door a user `defn`'s call-site check always uses, which knows
`Variant <: Enum` via the restored join's registered edges); and can a user-supplied TAGGED enum
variant construction (e.g. `(Option::Some {:value 1})`) reach that comparison as the argument.

**Bottom line: 3 functions carry the bare-`unify`-instead-of-`assignable` defect class** (well under
the ~15 STOP-1 threshold — this stays a stone, not a campaign):

| # | Function | Status |
|---|---|---|
| 1 | `infer_ordering` (`<`/`>`/`<=`/`>=`) | **FIXED this stone** — the two sub-defects brief ② names |
| 2 | `infer_hashset_constructor` | **FOUND, NOT fixed** (out of this stone's prescribed scope — see below) |
| 3 | `infer_hashmap_constructor` | **FOUND, NOT fixed** (ditto) |

`infer_hashset_constructor`/`infer_hashmap_constructor` are a genuine, **confirmed-by-probe**
instance of the identical class (declared element type is ALWAYS present — the `:- [T]`/`:- [K V]`
bracket is mandatory grammar for both forms, never bracket-less-inferred — so per the same rationale
`infer_persistentmap_constructor`/`infer_persistentvector_constructor` already encode ("declared →
`assignable`; bracket-less-inferred → `unify`"), these two should route through `assignable` when
the declared slot is present, and currently always use bare `unify`). This stone's brief prescribes
exactly two fixes (the comparison family, `src/check.rs:~13822`) plus the golden recapture — it does
not ask for a sweep of everything the census turns up, and STOP-1's philosophy is "report, let the
orchestrator shape a campaign" once more than one hand-picked site shares a class. Reported here,
**not fixed**, per that instruction.

Verbatim confirmation (probe, `/tmp` scratch, not committed):
```
./target/release/wat --check probe_hashset.wat
#wat.check/CheckErrors {:message "2 type-check errors" ... :errors [
  #wat.check/TypeMismatch {:message ":wat::core::HashSet: parameter element #1 expects
    (:wat::core::Option :- [:wat::core::i64]); got (:wat::core::Option::Some :- [:wat::core::i64])" ...}
  #wat.check/TypeMismatch {:message ":wat::core::HashSet: parameter element #2 expects
    (:wat::core::Option :- [:wat::core::i64]); got (:wat::core::Option::None :- [:?2991])" ...}]}
```
(HashMap: byte-identical shape, `param "value #1"`/`"value #2"`.)

## Full census

Legend — **Cmp?**: does this fn compare an arg's inferred type against a declared/expected type at
all (Y/N). **Door**: `unify` / `assignable` / both (mixed, by design or defect) / `n/a` (no
comparison). **Enum-reach?**: can a user-supplied TAGGED variant construction reach the comparison
as the argument — **Y** (confirmed or structurally certain), **N** (expected type can never be an
enum — bool/i64/String/keyword/Tuple/Address/opaque-handle leaves, or unit-only enum whose bare
keyword literal never narrows — see `infer_signal` probe below), **N/A** (no comparison exists, or
the comparison is body-vs-declared-return-type / operand-vs-operand rather than arg-vs-declared-
param, out of this census's frame per the brief's own wording).

| Function | Cmp? | Door | Enum-reach? | Note |
|---|---|---|---|---|
| `infer_some_constructor` | N | n/a | N/A | payload passed through, no expected-type gate here |
| `infer_ok_constructor` | N | n/a | N/A | ditto |
| `infer_err_constructor` | N | n/a | N/A | ditto |
| `infer_rete_form` | N | n/a | N/A | dispatch/shape only |
| `infer_list` | Y | **both** | Y | the general call/dispatch path — already the reference impl of the FORWARD/REVERSE `assignable` fan (lines ~5447-5534) plus per-arg `assignable` (5935,5960,5995,6127) that a user `defn` resolves through; bare `unify` calls here (3154,3169,3664,3739,5562) are internal element-type solving / holon-arg shape checks, not param-vs-declared gates |
| `infer_match` | Y | **assignable** (+ `join_types`) | Y | scrutinee check at 6219 (`assignable`); arm-join goes through `combine_match_arm`/`join_types` (already RELAND-1/2 territory) |
| `infer_if` | Y | unify | N | condition vs `:bool` — bool has no variants |
| `infer_do` | N | n/a | N/A | sequencing only |
| `infer_let` | N | n/a | N/A | binding only, no expected-type gate |
| `infer_def` | N | n/a | N/A | ditto |
| `infer_defclause` | Y | unify | N/A | ALL FIVE `unify` calls (8890,8929,8962,8996,9057,9011) are body-vs-own-declared-return-type or ensure-fn-declared-annotation-vs-declared-return-type checks — DECLARATION vs DECLARATION / BODY vs OWN RETURN, not a call-site ARGUMENT vs param; out of this census's frame (see legend) |
| `infer_config_set_bool` | N | n/a | N/A | literal bool value only |
| `infer_try` | Y | assignable | Y | 9688 |
| `infer_option_try` | Y | assignable | Y | 9795 |
| `infer_option_expect` | Y | assignable (opt) / unify (msg) | Y (opt) / N (msg=String) | **fixed RELAND-2**, mechanism ① |
| `infer_result_expect` | Y | assignable (res) / unify (msg) | Y (res) / N (msg=String) | **fixed RELAND-2**, mechanism ① |
| `infer_kernel_readln_prime` | N | n/a | N/A | cap i64 side-effect check only |
| `infer_ioreader_read_frame` | N | n/a | N/A | structural only |
| `infer_apply` | N | n/a | N/A | delegates to the callee's own scheme check |
| `infer_positional_accessor` | N | n/a | N/A | field-index lookup, no expected-type gate |
| `infer_nth` | Y | unify | N | index vs `:i64` |
| `infer_program_self_peer` | N | n/a | N/A | structural |
| `infer_listener_prime` | N | n/a | N/A | structural (Parametric arity/head match arms) |
| `infer_connect_prime` | Y | unify | N | addr vs `Address` — `Address` is an internal transport type, not a `defenum` |
| `infer_accept_prime` | N | n/a | N/A | structural |
| `infer_allow_prime` | Y | unify | N | pid vs `:i64` |
| `infer_deny_prime` | Y | unify | N | pid vs `:i64` |
| `infer_thread_prog_type` | N | n/a | N/A | structural |
| `infer_process_prog_type` | N | n/a | N/A | structural |
| `infer_spawn_thread_prime` | N | n/a | N/A | structural |
| `infer_spawn_process_prime` | N | n/a | N/A | structural |
| `infer_kernel_fn_forms` | Y | assignable | Y | 11425 |
| `infer_kernel_after` | Y | assignable | Y | 11567, 11584 |
| `infer_send_prime` | N | n/a | N/A | delegates to `relate_value_to_slot` (already mechanism ③, RELAND-2) |
| `infer_try_send_prime` | N | n/a | N/A | ditto |
| `infer_recv_prime` | N | n/a | N/A | ditto |
| `infer_close_prime` | N | n/a | N/A | structural |
| `infer_signal` | Y | unify | **N (probed)** | sig vs `Signal` (a unit-only `defenum`); a unit variant's bare keyword literal types directly to the parent enum, never narrows — probed clean: `(:wat::kernel::signal p :wat::kernel::Signal::Terminate)` checks OK today, confirming no live gap |
| `infer_peer_process` | N | n/a | N/A | structural |
| `infer_peer_wire` | N | n/a | N/A | structural |
| `infer_address_wire` | Y | unify | N | same `Address` transport type as `infer_connect_prime` |
| `infer_require_wire_address` | Y (discarded) | unify | N/A | `let _ = unify(...)` — result thrown away, not a real gate |
| `infer_serve_dispatch_op` | N | n/a | N/A | structural |
| `infer_retag_op` | N | n/a | N/A | structural |
| `infer_select_prime` | N | n/a | N/A | structural |
| `infer_poll_prime` | Y | unify | N | rhs vs a synthesized `Tuple` shape — Tuple has no variants |
| `infer_hashset_constructor` | Y | **unify only** | **Y — CONFIRMED DEFECT** | declared `T` always present (mandatory `:- [T]`); see census summary above |
| `infer_equality` (`=`/`not=`) | Y (operand-vs-operand) | unify + hand `is_subtype` (Path~Path only) | latent, unconfirmed | `types_compatible`'s subtype fallback only handles `(Path,Path)`; two Parametric operands (e.g. `Option::Some<i64>` vs bare `Option<?N>`) fall straight to `unify`-fails→`false`, no `assignable`/`join_types` fallback — same SHAPE as ②'s pre-fix ordering bug, but `=`/`not=` is operand-vs-operand (not arg-vs-declared-param), and not named by this stone's brief or any floor red; noted, not fixed, not counted in the 3 above |
| `infer_ordering` (`<`/`>`/`<=`/`>=`) | Y (operand-vs-operand) | **unify → FIXED this stone** | **Y — FIXED** | brief ② sub-defects 1+2; see `SCORE-STONE-A2-RELAND-3.md` |
| `infer_aggregate_new_check` | Y | assignable | Y | field-value checks route through `assignable` (per its own doc, deliberately NOT the kwargs-construct pattern, but still `assignable`) |
| `infer_enum_map_ctor` | Y | assignable | Y | 14219-ish |
| `infer_kwargs_construct_check` | Y | n/a (delegates) | — | body shows 0 direct `unify`/`assignable` in this pass's scan; constructs a synthetic call checked via the standard `infer_list`/scheme path, which is the FORWARD `assignable` fan already covered above |
| `infer_projection_verb_check` | Y | assignable | Y | |
| `infer_polymorphic_time_arith` | Y | assignable | Y (via surface bound) | |
| `infer_form_matches` (`:wat::form::matches?`) | Y (guard vs bool; operand-vs-operand via helper `check_comparison`) | unify | N (guard=bool) / latent for the comparison helper | `check_comparison`'s `l`/`r` unify (a RETE clause's `(= field lit)`/`(< field lit)` comparison) is operand-vs-operand, same shape as `infer_equality`'s gap — not itself one of the 72 `infer_*` fns, noted not counted |
| `infer_holon_bind` | N | n/a | N/A | holon-algebra specific, no enum param position |
| `infer_holon_bundle` | N | n/a | N/A | ditto |
| `infer_polymorphic_holon_pair_to_bool` | N | n/a | N/A | ditto |
| `infer_polymorphic_holon_pair_to_path` | N | n/a | N/A | ditto |
| `infer_polymorphic_holon_to_i64` | N | n/a | N/A | ditto |
| `infer_hashmap_constructor` | Y | **unify only** | **Y — CONFIRMED DEFECT** | declared `K,V` always present (mandatory `:- [K V]`); see census summary above |
| `infer_persistentmap_constructor` | Y | **both, correctly branched** | Y when declared | ALREADY correct: `if declared.is_some() { assignable(...) } else { unify(...) }` — bracket-less (T inferred from first pair) stays invariant BY DESIGN, matching the same convention `infer_map_literal`/`infer_set_literal` use; declared-bracket path already routes through `assignable` |
| `infer_persistentvector_constructor` | Y | **both, correctly branched** | Y when declared | same pattern as above |
| `infer_map_literal` (`{...}` sugar) | Y | unify | N/A (by design) | NO declared-type spelling exists for this form — T is ALWAYS inferred from the first pair, same "bracket-less-inferred stays invariant" convention `infer_persistentmap_constructor`'s own doc states; not the same class |
| `infer_set_literal` (`#{...}` sugar) | Y | unify | N/A (by design) | ditto, no declared-type spelling exists |
| `infer_tuple_constructor` | Y | **assignable** (via `check_tuple_constructor_against` → `assignable` at line ~16674, when the optional `:- [T1..Tn]` bracket is present) | Y when bracketed | already correct |
| `infer_string_concat` | Y | unify | N | vs `:String` |
| `infer_string_interpolate` | Y | unify | N | vs `:String` (×2) |
| `infer_list_constructor` (`(:wat::core::vec :- [T] ...)`) | Y | **both, correctly branched** | Y | Surface bound → `assignable`; concrete elem → `unify` THEN `assignable` fallback (the R7 `Never`-bottom subtype case) — already the established two-mechanism fallback pattern |
| `infer_component_against` | Y | **assignable** (6 call sites) | Y | the ann-form-directed compound-checker family; already correct |
| `infer_linked_list_constructor` (`(:wat::core::List a b c)` bare form) | Y | unify | N/A (by design) | bracket-less-inferred — T comes from the first element, no declared slot exists for this bare form; same convention as `infer_map_literal`/`infer_set_literal` |
| `infer_boolean_shortcircuit` (`and`/`or`) | Y | unify | N | each arg vs `:bool` |

## Method

- `grep -n "fn infer_"` on `src/check.rs` → 72 (73rd match is a `#[test] fn` sharing the name
  prefix, excluded).
- For each, extracted the function's own body span (next `infer_*` start = this one's end) and
  scanned for `unify(`/`assignable(` call sites; read the surrounding code for every fn with at
  least one hit to classify the comparison's shape (arg-vs-declared-param vs body-vs-return vs
  operand-vs-operand vs discarded-result).
- Where the declared/expected side's type could plausibly be an enum, confirmed reachability by
  reading the grammar (mandatory vs optional type-param bracket) and, for the two live findings
  (`infer_hashset_constructor`, `infer_hashmap_constructor`) and one clean control (`infer_signal`),
  by an actual `./target/release/wat --check` probe against a throwaway `/tmp` fixture (not
  committed — this repo's scratch-`.wat` convention is for durable loadable references, not a
  one-shot census probe).

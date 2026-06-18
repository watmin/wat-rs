# BRIEF — Stone 0d: transform-op check-side parity (the vector disparity)

Single-hop sonnet in `/home/watmin/work/holon/wat-rs`. **No sub-agents. No `git`.** A bounded Rust stone:
8 projective custom infer arms (the `infer_conj`/`infer_get`/`infer_assoc` pattern). Build, run the named
tests, report verbatim. Another agent weighs independently against the disk.

## The work
0c gave `PersistentVector` the transform/sequence ops at RUNTIME; the CHECKER never followed. Make the 8
transform ops type-check on `Vector<T> | PersistentVector<T>`, type-preserving, by adding a **projective
custom infer arm** for each in `src/collection/infer.rs` and dispatching it from the keyword-head match in
`src/check.rs` — exactly mirroring the existing `infer_conj`/`infer_get`/`infer_assoc`. **Retire the static
Vec-only `TypeScheme`** for each (they become arm-driven, single source of truth — like conj/get/assoc, which
have NO static scheme).

The 8 ops + their projective signatures (`C ∈ {Vector, PersistentVector}`, type-preserving):

| op | call shape | coll arg | signature |
|---|---|---|---|
| `map`     | `(map f xs)`        | arg[1] | `C<T> × fn(T)->U → C<U>` |
| `filter`  | `(filter pred xs)`  | arg[1] | `C<T> × fn(T)->bool → C<T>` |
| `foldl`   | `(foldl f init xs)` | arg[2] | `fn(Acc,T)->Acc × Acc × C<T> → Acc` |
| `foldr`   | `(foldr f init xs)` | arg[2] | `fn(T,Acc)->Acc × Acc × C<T> → Acc` |
| `reverse` | `(reverse xs)`      | arg[0] | `C<T> → C<T>` |
| `take`    | `(take xs n)`       | arg[0] | `C<T> × i64 → C<T>` |
| `drop`    | `(drop xs n)`       | arg[0] | `C<T> × i64 → C<T>` |
| `concat`  | `(concat a b)`      | arg[0]+arg[1] | `C<T> × C<T> → C<T>` (SAME kind; mixed → TypeMismatch) |

## Read FIRST (in order) and implement EXACTLY
1. `docs/arc/2026/06/278-rules-engine/DESIGN-STONE-0d-transform-dispatch-parity.md` — the full contract, the
   four-questions decision (custom arms, NOT Dispatch-machinery), the per-op table, the ONE contract decision
   (type-preserving over `{Vector, PersistentVector}`; arms total; concat same-kind-only).
2. `src/collection/infer.rs` (lines 1-485) — the home + the EXACT precedent: `infer_contains` (:31),
   `infer_conj` (:129), `infer_get` (:226), `infer_assoc` (:338). Mirror their shape — extract the collection
   arg's parametric type, accept the two container heads, project the result type-preservingly, emit a
   teaching `TypeMismatch` for any other shape.
3. `src/check.rs:5136-5205` — the keyword-head dispatch arms for `contains?`/`conj`/`get`/`assoc` (the
   `return`-early pattern). Add one arm per transform op, the same shape. Also read the PARTITION doctrine
   comment at `5144-5167` — these are PROJECTIVE intrinsics; this is their declared home.
4. `src/check.rs:17963-18073` — the static Vec-only schemes to RETIRE (`reverse`/`take`/`drop`/`map`/`foldl`/
   `foldr`/`filter`). Leave `range` (:17972, monomorphic) and `sort'` (:18003, out of scope) ALONE.
5. `src/collection/eval.rs` (the 0c arms) + `src/value/value.rs` — for the `wat__core__PersistentVector`
   parametric type shape, so your infer arm matches/constructs the right `TypeExpr::Parametric { head:
   "wat::core::PersistentVector", args: [..] }`.
6. `tests/probe_arc278_0d_transform_dispatch_parity.rs` — remove the `#[ignore]` on
   `transform_ops_typecheck_on_persistent_vector`. It is your contract. The guard
   `wrong_element_still_rejected` must STAY green (parity ≠ permissiveness).

## The concat grounding sub-step (STOP-1 lives here)
`concat` is a `defalias` for `:wat::core::Vector/concat` (`wat/core.wat:44`); it checks via that impl's scheme,
NOT a surface arm (the probe shows it rejects BOTH args #1 and #2). FIRST ground how the alias reaches the
checker: can `concat` (or `:wat::core::Vector/concat`) take a surface custom infer arm like the other 7, or
does parity belong on a `:wat::core::PersistentVector/concat` impl scheme paralleling `Vector/concat`?
Implement whichever the alias path actually supports — end state identical: `(concat pv pv) → PersistentVector`,
same-kind-only, mixed → TypeMismatch (mirrors the runtime `vector_concat_inner`). **If concat's check path is
structurally different from the other 7 and can't take a clean arm → STOP and report what you found; do not
invent a dispatch path.**

## STOP triggers (HALT + report; do NOT improvise)
1. concat's alias/scheme path is structurally different and won't take a clean arm (above) → STOP, report.
2. Retiring a static scheme breaks a Vector call that the scheme uniquely served (your arm must cover the full
   Vector behavior the scheme did — same element-typing, same return) → if you cannot make the arm total over
   Vector, STOP.
3. A floor moves beyond the new probe (a pre-existing test changes count) → STOP, report which.

## Verify (run each; paste output VERBATIM)
```
cargo test --release -p wat --test probe_arc278_0d_transform_dispatch_parity -- --include-ignored   # 2/2 GREEN
cargo test --release -p wat --test probe_arc278_0a_persistent_map -- --include-ignored               # 1/1 (unchanged)
cargo test --release -p wat --test probe_arc278_0b_persistent_vector -- --include-ignored             # 1/1 (unchanged)
cargo test --release -p wat --test probe_arc278_0c_persistent_parity -- --include-ignored             # 1/1 (unchanged)
cargo test --release -p wat --lib 2>&1 | grep "test result"                                          # 931/36 (UNCHANGED)
cargo test --release --test test 2>&1 | grep "test result"                                           # 264/1 (UNCHANGED)
cargo test --release --test test_stdlib_load_order | grep result                                     # 1/0
cargo build --release 2>&1 | tail -2                                                                  # clean
```
Report: the diff (the 8 infer arms + the 8 dispatch arms + the retired schemes + concat handling), all command
outputs verbatim, any STOP hit, and the concat grounding finding. Do not claim a green you did not see.
Un-ignore the parity test. No git.

## Blast radius
`src/collection/infer.rs` (8 new projective infer fns) · `src/check.rs` (8 `return`-early dispatch arms; retire
the 8 static schemes at 17963-18073) · maybe `wat/core.wat` / a per-Type `PersistentVector/concat` scheme for
the concat path · the probe (un-ignore). **NO runtime change** (0c shipped it). NO new dispatch machinery. No git.

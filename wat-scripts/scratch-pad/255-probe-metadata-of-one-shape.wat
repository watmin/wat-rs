;; Scratch probe — arc 255 Stone "metadata-of answers in one shape".
;;
;; Proves `(:wat::runtime::metadata-of X)` answers ACROSS FIVE axes — `:purity`,
;; `:determinism`, `:totality`, `:expand-time`, `:category` — in the SAME `Value::Enum` shape
;; whether `X` is a registered Rust intrinsic (`:wat::core::sort$native`) or a wat `defn`
;; carrying a doc-axis metadata map (`:wat::string::capitalize`). Converging one key and
;; leaving the rest moves the defect rather than removing it (DESIGN's own acceptance bar), so
;; every axis is checked, not `:purity` alone.
;;
;; ★ Checked with `:wat::core::=`, not `pprintln` — the NOTE's own finding is that a
;; `Value::Enum` and a raw `Value::wat__WatAST(Keyword ...)` render IDENTICALLY in EDN
;; (`:wat.runtime.Purity/Pure` either way), so a print-only probe cannot tell them apart. `=`
;; raises a TypeMismatch (not a quiet `false`) when the two sides aren't even the same `Value`
;; variant — so on the PRE-FIX binary this probe is expected to ABORT with a runtime
;; TypeMismatch, not print "false", the moment it compares the two shapes.
;;
;; ⚠ Structural surprise found writing this probe, unrelated to `eval_metadata_of` itself:
;; `:wat::hashmap::get`'s return type follows the map's DECLARED value type — here
;; `:wat::core::Value`, the universal top (`hm`'s type is `(HashMap :- [keyword Value])`,
;; mirroring `metadata-of`'s own `@ret`) — so `(get hm :purity)` types as `Option<Value>`, and
;; comparing THAT directly against `(Some <narrow-enum-literal>)` is a genuine `Option<Value>`
;; vs `Option<Purity>` mismatch: `infer_equality`'s subtype fallback (`Purity <: Value`) only
;; fires for bare `Path` types, not through a generic wrapper like `Option` (measured: bare
;; `(= (Some Purity::Pure) (Some Purity::Pure))` and even two DIFFERENT bare-enum `=` calls in
;; one body type-check fine — `/tmp/test_unify3.wat`; only the hashmap-sourced `Option<Value>`
;; side trips it). Fixed below by `match`-unwrapping each `Option` FIRST, then comparing the
;; bare inner `Value` against the bare enum literal directly — bare-Path subtyping applies
;; there. (Not a claim about `eval_metadata_of`'s own correctness — purely how this probe had
;; to be shaped to type-check.)
;;
;; `sort$native` and `capitalize` happen to declare the SAME five axis values
;; (`Pure`/`Deterministic`/`Total`/`Legal`/`Transform` — checked against `wat/string.wat` and
;; `src/intrinsic/collection.rs`'s doc comments), which is what lets ONE literal per axis check
;; BOTH branches.
;;
;; Also checked:
;; - `:defined-in` DISCRIMINATES (`Rust` for the intrinsic, `Wat` for the verb) instead of both
;;   branches asserting the same spliced constant — the stone's second defect (STOP-4).
;; - `:layer` is left alone — NOT asserted here at all (STOP-3): this probe does not touch it in
;;   either direction, on either branch.
;;
;; NOT exercised live here: the capability-only-map case (`{:restricted-to […]}`, STOP-2). All
;; 4 live corpus sites (`write-fd-raw`/`flood-stdout-raw`/`str-double` in
;; `wat/kernel/services/stdio.wat`, one in `wat/spawn.wat`) are themselves `{:restricted-to
;; [:wat::kernel:: / :wat::spawn:: :wat::test::]}`-gated, and arc 198's restriction walker
;; governs MENTION, not call-head position (`check.rs::walk_for_restricted_call`) — so even
;; passing one of their FQDNs as a bare keyword ARGUMENT to `metadata-of` from a `:user::`
;; caller trips `DefRestrictedCallerNotAllowed`, unrelated to this stone. STOP-2 is verified by
;; construction instead: the new doc-path arm is gated on `meta_has_doc_axis_key(meta)`
;; (`src/runtime.rs`), the SAME predicate the registration gate uses, and a `{:restricted-to
;; […]}`-only map contains none of `DOC_AXIS_KEYS` — so it can only ever fall to the raw,
;; unchanged, un-decoded arm. See the rider's report for the full argument.
;;
;; This is a READ-ONLY probe (no `deftest` — `metadata-of` is a pure reflection query); it
;; type-checks and runs on whatever binary is currently on disk via `target/release/wat`. See
;; the rider's report for what the CURRENT (pre-rebuild) binary actually does when run against
;; this file, and an explicit note on what only a rebuild can show.

(:wat::core::defn :user::axis-ok?
  [got <- (:wat::core::Option :- [:wat::core::Value]) want <- :wat::core::Value]
  -> :wat::core::bool
  (:wat::core::match got
    ((:wat::core::Some v) (:wat::core::= v want))
    (:None false)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::runtime::metadata-of :wat::core::sort$native)
    ((:wat::core::Some intrinsic-hm)
     (:wat::core::match (:wat::runtime::metadata-of :wat::string::capitalize)
       ((:wat::core::Some wat-hm)
        (:wat::core::do
          (:wat::kernel::println "── raw maps, for eyeball comparison ──")
          (:wat::kernel::pprintln intrinsic-hm)
          (:wat::kernel::pprintln wat-hm)
          (:wat::kernel::println "── the five axes, both branches, one literal each ──")
          (:wat::kernel::println (:wat::string::concat "purity:       intrinsic=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get intrinsic-hm :purity) :wat::runtime::Purity::Pure)) " wat=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get wat-hm :purity) :wat::runtime::Purity::Pure))))
          (:wat::kernel::println (:wat::string::concat "determinism:  intrinsic=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get intrinsic-hm :determinism) :wat::runtime::Determinism::Deterministic)) " wat=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get wat-hm :determinism) :wat::runtime::Determinism::Deterministic))))
          (:wat::kernel::println (:wat::string::concat "totality:     intrinsic=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get intrinsic-hm :totality) :wat::runtime::Totality::Total)) " wat=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get wat-hm :totality) :wat::runtime::Totality::Total))))
          (:wat::kernel::println (:wat::string::concat "expand-time:  intrinsic=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get intrinsic-hm :expand-time) :wat::runtime::ExpandTime::Legal)) " wat=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get wat-hm :expand-time) :wat::runtime::ExpandTime::Legal))))
          (:wat::kernel::println (:wat::string::concat "category:     intrinsic=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get intrinsic-hm :category) :wat::runtime::Category::Transform)) " wat=" (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get wat-hm :category) :wat::runtime::Category::Transform))))
          (:wat::kernel::println "── :defined-in discriminates (STOP-4) ──")
          (:wat::kernel::println (:wat::string::concat "defined-in:   intrinsic=Rust? " (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get intrinsic-hm :defined-in) :wat::runtime::DefinedIn::Rust)) " wat=Wat? " (:wat::edn::write (:user::axis-ok? (:wat::hashmap::get wat-hm :defined-in) :wat::runtime::DefinedIn::Wat))))))
       (:None (:wat::kernel::println "capitalize metadata-of => NONE (unexpected)"))))
    (:None (:wat::kernel::println "sort$native metadata-of => NONE (unexpected)"))))

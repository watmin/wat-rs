;; Scratch probe — arc 255 Stone "a declaration cannot be STORED unvalidated".
;;
;; `record_binding_metadata` (`src/runtime.rs`) is now the ONE door every `sym.binding_metadata`
;; insert routes through (`grep -c "binding_metadata.insert"` == 1). It validates a map that
;; claims a substrate axis property (any of `AXIS_DECLARATION_KEYS`) through
;; `wat_doc::from_metadata` at the DECLARATION's own span, and leaves a capability-only map
;; (no axis key) untouched — same `meta_has_doc_axis_key` predicate the read side
;; (`eval_metadata_of`) uses, so registration and reflection can never disagree about what
;; counts as a declaration.
;;
;; This probe demonstrates the two shapes that are SAFE to keep loading forever in this file:
;;   1. `:user::probe-complete` — every required doc-axis key present. Loads clean, and
;;      `metadata-of` reads back the declared axis values.
;;   2. `:user::probe-restricted` — `{:restricted-to […]}` only, no axis key. Loads clean and
;;      `metadata-of` reads back the raw, un-decoded map — exactly as before this stone
;;      (STOP-3: capability maps must not start failing).
;;
;; ⛔ NOT exercised live here, on purpose: a PARTIAL declaration (e.g. `{:purity …}` alone, the
;; DESIGN doc's `:my::halfdecl` shape). `record_binding_metadata` runs at `register_defines`/
;; `register_stdlib_defines` time — i.e. at LOAD, before `:user::main` ever runs — for every
;; `def`/`defn` in the file, top-level or nested in a `do`/`let`. A bad declaration therefore
;; doesn't raise a catchable runtime `Err` `:user::main` could report; it fails the WHOLE file's
;; load. `:wat::eval-ast!` can't reach it either: `def` at expression position is refused with
;; `DeclarationInExpressionPosition` before `record_binding_metadata` is ever called — that path
;; is definitionally freeze-time-only (see `src/runtime.rs`'s `:wat::core::def` arm in
;; `dispatch_keyword_head`). And this repo's OWN `every_wat_scripts_file_loads` gate
;; (`tests/lint/wat_scripts_fixes_load.rs`) parses + type-checks (i.e. fully LOADS, registration
;; included) every `.wat` under `wat-scripts/` — so a scratch file that fails to load is not a
;; probe, it is a permanent floor regression. Demonstrating the refusal is therefore a one-shot
;; act on a throwaway file outside this corpus, run directly against the binary; see the rider's
;; report for that transcript (pre-fix binary: misattributes to a reader's line; post-rebuild:
;; expected to name the declaration's own line — not measured here, no rebuild available).

(:wat::core::defn :user::probe-complete
  {:doc "Scratch probe verb — returns its argument unchanged."
   :added "1.0.0"
   :ret [:wat::core::i64 "x, unchanged"]
   :purity :wat::runtime::Purity::Pure
   :determinism :wat::runtime::Determinism::Deterministic
   :totality :wat::runtime::Totality::Total
   :expand-time :wat::runtime::ExpandTime::Legal
   :category :wat::runtime::Category::Transform
   :examples [["(:user::probe-complete 41)" "41"]]}
  [x <- :wat::core::i64]
  -> :wat::core::i64
  x)

(:wat::core::defn :user::probe-restricted
  {:restricted-to [:user::]}
  [x <- :wat::core::i64]
  -> :wat::core::i64
  x)

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── probe-complete: calls fine, metadata-of decodes the axes ──")
    (:wat::kernel::println (:wat::string::concat "call result: " (:wat::edn::write (:user::probe-complete 41))))
    (:wat::core::match (:wat::runtime::metadata-of :user::probe-complete)
      ((:wat::core::Some hm) (:wat::kernel::pprintln hm))
      (:None (:wat::kernel::println "probe-complete metadata-of => NONE (unexpected)")))
    (:wat::kernel::println "── probe-restricted: capability-only map, raw and unvalidated (STOP-3) ──")
    (:wat::core::match (:wat::runtime::metadata-of :user::probe-restricted)
      ((:wat::core::Some hm) (:wat::kernel::pprintln hm))
      (:None (:wat::kernel::println "probe-restricted metadata-of => NONE (unexpected)")))))

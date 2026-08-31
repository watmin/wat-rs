;; Scratch probe — arc 255 STONE "an example is a FORM, not a string".
;;
;; `wat_doc::DocExample.expr`/`.expected` are now parsed `WatAST` forms, not text — a malformed
;; `@example`/metadata `:examples` entry is a `DocError` at the site that wrote it (a
;; `compile_error!` for a Rust `///` block via `#[wat_intrinsic]`/`#[wat_special_form]`; a
;; `RuntimeError` at stdlib-registration time for a wat `defn`'s `{...}` metadata map), instead of
;; surviving as opaque text until a later reflection-test re-parse fails.
;;
;; `wat/string.wat`'s `capitalize` is the walked verb (arc 255's "wire the wat side to wat-doc"
;; door-plus-one-verb precedent). Its `:examples` now write LITERAL wat forms —
;;   :examples [[(:wat::string::capitalize "object") "Object"]
;;              [(:wat::string::capitalize "") ""]]
;; — no escaped-string source (`"(:wat::string::capitalize \"object\")"`) anywhere.
;;
;; "its examples read back" — MEASURED, not what I first assumed: `metadata-of` (`src/runtime.rs`)
;; deliberately does NOT emit `:args`/`:examples`/`:see`/`:yields`/`:deprecated` for EITHER branch
;; (registry or wat) — a pre-existing, explicitly-commented scope cut ("CARRIED on the entry but
;; rendered by the iv-b2 verifier seam, not here"), unrelated to and unchanged by this stone. So
;; there is no wat-level query that shows `capitalize`'s `:examples` back as data today — the probe
;; below cannot demonstrate that literally. What it DOES prove: `record_binding_metadata` runs
;; `wat_doc::from_metadata` on this declaration's WHOLE metadata map at stdlib-load time, in one
;; pass — it cannot succeed on `:doc`/`:added`/`:ret`/… while silently skipping a malformed
;; `:examples`. So the fact that this file loads at all AND `metadata-of` below successfully
;; decodes `capitalize`'s OTHER axes IS the proof that `:examples`' new literal-form shape parsed
;; clean through `from_metadata` too — the same all-or-nothing decode `record_binding_metadata`
;; already gated stdlib startup on, before this stone existed.
;;
;; ⛔ NOT exercised live here, on purpose — the negative (a malformed example) is now a COMPILE
;; failure, so it cannot live in a committed `.wat` (`every_wat_scripts_file_loads`,
;; `tests/lint/wat_scripts_fixes_load.rs`, fully loads every file under `wat-scripts/`, so a
;; failing declaration here would be a permanent floor regression) or in a committed `.rs` (it
;; would not build at all). Demonstrated once out-of-tree instead, against a throwaway file run
;; directly with `target/release/wat` (which does not contain this stone's Rust changes, so it
;; exercises `wat_reader::parse_one_with_file` — the same reader `wat_doc::parse_example_form`
;; calls — directly, as a proxy): the DESIGN's own motivating defect text,
;; `Record/field-at`'s shipped `#=> <r's first field's value>`, does NOT parse as a single,
;; complete wat form (multiple bare symbols / an unterminated `<`), where `(:wat::string::capitalize
;; "object")` does. See the rider's report for the transcript.

(:wat::core::defn :probe::check [name <- :wat::core::String got <- :wat::core::String want <- :wat::core::String]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::= got want)
    (:wat::kernel::println (:wat::string::concat "PASS " name))
    (:wat::kernel::println (:wat::string::concat "FAIL " name " got=" got " want=" want))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "── capitalize still works ──")
    (:probe::check "capitalize object" (:wat::string::capitalize "object") "Object")
    (:probe::check "capitalize empty"  (:wat::string::capitalize "") "")

    (:wat::kernel::println "── metadata-of still decodes (from_metadata parsed the WHOLE map, :examples included) ──")
    (:wat::core::match (:wat::runtime::metadata-of :wat::string::capitalize)
      ((:wat::core::Some hm) (:wat::kernel::pprintln hm))
      (:None (:wat::kernel::println "capitalize metadata-of => NONE (unexpected)")))))

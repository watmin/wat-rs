;; ─── Arc 255 "the walls must not be muted" — BRIEF-STONE-the-edn-doc-row-is-imposed ──────────
;;
;; STOP-3's instrument: capture `(:wat::runtime::metadata-of :wat::core::char)` and
;; `(:wat::core::render-doc :wat::core::char)` before AND after converting `:wat::core::char`'s
;; `///` doc block from the `@name value` text grammar to a ```edn `#wat.doc/Row {...}` fence.
;; Run this script, save the output, convert the row, rebuild, run again, diff. Any difference
;; at all is STOP-3.
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "== metadata-of :wat::core::char ==")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of :wat::core::char))
    (:wat::kernel::println "== render-doc :wat::core::char ==")
    (:wat::kernel::println (:wat::core::render-doc :wat::core::char))))

;; ─── Arc 255 "the walls must not be muted" — BRIEF-STONE-the-edn-doc-row-is-imposed ──────────
;;
;; STOP-3's instrument, amended by 255.81: the bare keyword `:wat::core::char` is the
;; retired name itself, so a loadable program cannot write it as a node (the checker
;; refuses it). The old spelling stays the SUBJECT, carried as a string into
;; `keyword-node`, and both doors refuse that keyword with the retirement remedy.
;; A metadata diff of the old row is no longer a reading this file can take.
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::do
    (:wat::kernel::println "== metadata-of :wat::core::char ==")
    (:wat::kernel::pprintln (:wat::runtime::metadata-of (:wat::core::keyword-node ":wat::core::char")))
    (:wat::kernel::println "== render-doc :wat::core::char ==")
    (:wat::kernel::println (:wat::core::render-doc (:wat::core::keyword-node ":wat::core::char")))))

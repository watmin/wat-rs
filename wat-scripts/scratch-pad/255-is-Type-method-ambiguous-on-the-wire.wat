;; ─── Arc 255 — does edn::write emit READABLE EDN for a Type/method wat keyword? ──────────────
;;
;; The Type/method form (`:wat::holon::Hologram/make`, 82 registered names) and the
;; Type::method form (`:wat::core::Bytes::to-hex`, 5 names) are both live wat spellings. This
;; asks what each becomes on the EDN wire, and whether the wire form reads back.
;;
;; ⛔ MEASUREMENT, never a ratchet.

(:wat::core::defn :p::show [label <- :wat::core::String k <- :wat::core::keyword] -> :wat::core::nil
  (:wat::kernel::println (:wat::string::concat (:wat::string::concat label "  -> ") (:wat::edn::write k))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "-- what each live spelling becomes on the wire --")
    (:p::show "Type::method (5 names) " :my::ns::Bytes::to-hex)
    (:p::show "Type/method  (82 names)" :my::ns::Bytes/to-hex)
    (:wat::kernel::println "")))

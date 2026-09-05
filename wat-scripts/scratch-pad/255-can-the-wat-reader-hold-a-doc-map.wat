;; ─── Arc 255 — can the WAT reader hold a doc metadata map? ───────────────────────────────────
;;
;; Builder: "wat has defns with metadata-maps already showing real things that part correctly."
;; True — `:wat::core::sort`'s defclause carries :doc/:added/:ret/:purity/…/:examples and the wat
;; reader parses it. But `wat_edn::parse` REFUSES `::` keywords (measured:
;; InvalidKeyword("keyword begins with :: ")), so an EDN-read doc map cannot use wat's own
;; spelling — which would put TWO spellings back, the exact thing the migration exists to remove.
;;
;; The real question is therefore not the SHAPE (wat already answers it) but the READER.
;; `wat-reader` is ALREADY a `wat-macros` dependency. This asks what it accepts.
;;
;; ⛔ MEASUREMENT, never a ratchet.

(:wat::core::defn :p::kind [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string src)
    ((:wat::core::ReadOutcome::Forms fs)
      (:wat::core::ast-kind (:wat::core::first fs)))
    ((:wat::core::ReadOutcome::Malformed c)
      (:wat::string::concat "MALFORMED: " (:wat::core::Error/message c)))))

(:wat::core::defn :p::try [label <- :wat::core::String src <- :wat::core::String] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::string::concat (:wat::string::concat label "  -> ") (:p::kind src))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:p::try "map with :: keywords "
      "{:purity :wat::runtime::Purity::Pure :added \"1.0.0\"}")
    (:p::try "map with :ret vector  "
      "{:ret [:wat::core::Vector \"a new vector\"]}")
    (:p::try "map with :examples    "
      "{:examples [[\"(:wat::core::sort [3 1 2])\" \"[1 2 3]\"]]}")
    (:p::try "TAGGED #wat.doc/Row   "
      "#wat.doc/Row {:added \"1.0.0\"}")
    (:wat::kernel::println ""))) 

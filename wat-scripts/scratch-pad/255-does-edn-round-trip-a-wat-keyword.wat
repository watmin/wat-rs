;; ─── Arc 255 — does the EDN wire round-trip a wat FQDN keyword? ──────────────────────────────
;;
;; `wat_edn`'s lexer terminates a keyword body at `:`, so `:my::doc::some-name` lexes as `:wat`
;; followed by a token beginning `::` — InvalidKeyword. Meanwhile `edn/render.rs:818` renders a
;; `WatAST::Keyword` by pushing its text verbatim. If the writer emits what the reader refuses,
;; that is a round-trip defect in the wire format, not merely a constraint on doc comments.
;;
;; ⛔ MEASUREMENT, never a ratchet.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println "-- a bare keyword VALUE through edn::write --")
    (:wat::kernel::println (:wat::edn::write :my::doc::some-name))
    (:wat::kernel::println "-- LOSSLESS? read(write(k)) == k --")
    (:wat::kernel::println
      (:wat::core::bool::to-string
        (:wat::core::= (:wat::edn::read (:wat::edn::write :my::doc::some-name)) :my::doc::some-name)))
    ;; The tagged-read is NOT run here — it RAISES today, and that raise is the finding:
    ;;   (:wat::edn::read "#wat.doc/Row {:added \"1.0.0\"}")
    ;;   -> MalformedForm: "unknown tag #wat.doc/Row (body shape: map); no matching struct or
    ;;      enum in the type registry"  (src/edn/render.rs:3310)
    ;; i.e. a tag RESOLVES TO A REGISTERED RECORD. `#wat.doc/Row` starts working the moment
    ;; `:wat::doc::Row` exists as a type — and the reader then VALIDATES the map against it.
    ;; Kept as a comment so this probe stays exit-0 and remains a durable instrument.
    (:wat::kernel::println "")))

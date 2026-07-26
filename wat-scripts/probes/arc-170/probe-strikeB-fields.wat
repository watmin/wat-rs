;; STRIKE B RED PROBE — struct-field reflection. Given a type, get its fields (names + types).
;; EXPECT (RED before B): unknown verb :wat::runtime::field-names-of / field-types-of.
;; EXPECT (after B): field-names-of :probe::Bag -> the field names; field-types-of -> the field types.
;; NOTE: println RENDERS EDN directly (it IS value->edn->wat_edn::write). NEVER wrap in edn/write —
;; that double-encodes (re-quotes + escapes the already-EDN text). Print the value straight.
;; (print-on-edn is heresy; edn/write is only for EDN-as-a-String: wire send / sqlite / concat.)
(:wat::core::defrecord :probe::Bag [kv <- :wat::kernel::Peer'<probe::Kv::Op,probe::Kv::Reply>  n <- :wat::core::i64])
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer'
  :messages [(:wat::core::defenum :probe::Kv::R :wat::enum::Pure :Ok [x <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                               :RequestMalformed [path <- :wat::core::Vector<wat::core::String>  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(get [self <- :probe::Kv req <- :probe::Kv::R] -> :probe::Kv::R :max-request-bytes 524288)])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:wat::runtime::field-names-of :probe::Bag))
     _ (:wat::kernel::println (:wat::runtime::field-types-of :probe::Bag))]
    (:wat::kernel::println "fields-of: ok")))

;; STRIKE B RED PROBE — struct-field reflection. Given a type, get its fields (names + types).
;; EXPECT (RED before B): unknown verb :wat::runtime::field-names-of / field-types-of.
;; EXPECT (after B): field-names-of :probe::Bag -> the field names; field-types-of -> the field types.
;; NOTE: println RENDERS EDN directly (it IS value->edn->wat_edn::write). NEVER wrap in edn/write —
;; that double-encodes (re-quotes + escapes the already-EDN text). Print the value straight.
;; (print-on-edn is heresy; edn/write is only for EDN-as-a-String: wire send / sqlite / concat.)
;; A STRUCT, not a record — arc 278 2026-08-03, builder-ruled: peers are RESOURCES, not pure.
;; `kv` is a live `Peer`, so this aggregate can never cross the wire; a record is GUARANTEED
;; pure data ([[reference_struct_holds_resources_record_is_pure_data]]). It was a defrecord
;; until the §7 purity wall was corrected to cover Peer/Thread/Process — this file was the
;; ONLY one of 260 under wat-scripts/ that the correction lit. The probe's own header already
;; called it "struct-field reflection"; the declaration simply did not match the name.
(:wat::core::defstruct :probe::Bag [kv <- (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply])  n <- :wat::core::i64])
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Kv::GetRequest [k <- :wat::core::String])
             (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure :Ok [x <- :wat::core::String] :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
                                                                                               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat::kernel::println (:wat::runtime::field-names-of :probe::Bag))
     _ (:wat::kernel::println (:wat::runtime::field-types-of :probe::Bag))]
    (:wat::kernel::println "fields-of: ok")))

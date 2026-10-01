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
;; CLAIM: field-names-of :probe::Bag == [:kv :n] exactly (keywords are Equatable data).
;; field-types-of returns a `(Vector :- [wat::WatAST])` — actual AST nodes, NOT opaque type
;; descriptors and NOT strings (`src/check.rs` field-types-of: "rendered to the canonical
;; wat.type/ WatAST form … plain-EDN, decomposable, reparseable"). Field types are
;; deterministic, not pid-shaped, so asserted EXACTLY as data: each is AST-node-equal
;; (`wat.type/AST` extends Equatable; `WatAST`'s PartialEq compares structure, skipping span)
;; to a node built by parsing the SAME source spelling `field-types-of` itself emits
;; (`:wat::core::read-string`), never by a length-only or string-rendered proxy (coordinator
;; correction, 255.76 weigh: "measuring strings is anti-wat").
(:wat::core::defstruct :probe::Bag [kv <- (:wat::kernel::Peer :- [:probe::Kv::Op :probe::Kv::Reply])  n <- wat.type/i64])
(:wat::core::defsurface :probe::Kv :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::Kv::GetRequest [k <- wat.type/String])
             (:wat::core::defenum :probe::Kv::GetResponse :wat::enum::Pure :Ok [x <- wat.type/String] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                               :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features [(get [self <- :probe::Kv req <- :probe::Kv::GetRequest] -> :probe::Kv::GetResponse :max-request-bytes 524288)])
(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [names (:wat::runtime::field-names-of :probe::Bag)
     types (:wat::runtime::field-types-of :probe::Bag)
     t0    (:wat::core::nth types 0)
     t1    (:wat::core::nth types 1)
     parsed
       (:wat::core::match
         (:wat::core::read-string "(wat.kernel/Peer :- [probe.Kv/Op probe.Kv/Reply]) wat.type/i64")
         [:wat::core::ReadOutcome.Forms {:forms fs} fs]
         [:wat::core::ReadOutcome.Malformed {:cause c}
           (:wat::kernel::assertion-failed! :message (:wat::core::Error/message c))])
     expected-t0 (:wat::core::nth parsed 0)
     expected-t1 (:wat::core::nth parsed 1)]
    (:wat::core::do
      (:wat::test::assert-eq names (wat.type/Vector :- [wat.type/keyword] :kv :n))
      (:wat::test::assert-eq t0 expected-t0)
      (:wat::test::assert-eq t1 expected-t1)
      (:wat::kernel::println names)
      (:wat::kernel::println types)
      (:wat::kernel::println "fields-of: ok"))))

;; tests/types/probe_arc234_7b_holon_record_roundtrip.wat — co-located fixture
;;
;; Arc 234 Stone 234.7b — holon wat__holon__Record round-trips on the EDN wire.

(:wat::holon::defrecord :test::rd::HPt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :user::write-hpt [] -> :wat::core::String
    (:wat::core::let [h (:test::rd::HPt :x 7 :y 8)]
        (:wat::edn::write h)))

(:wat::core::defn :user::roundtrip-eq [] -> :wat::core::bool
    (:wat::core::let
        [h  (:test::rd::HPt :x 7 :y 8)
         s  (:wat::edn::write h)
         h2 (:wat::edn::read s)]
        (:wat::core::= h h2)))

(:wat::core::defn :user::roundtrip-field-x [] -> :wat::core::i64
    (:wat::core::let
        [h  (:test::rd::HPt :x 7 :y 8)
         s  (:wat::edn::write h)
         h2 (:wat::edn::read s)]
        (:test::rd::HPt/x h2)))


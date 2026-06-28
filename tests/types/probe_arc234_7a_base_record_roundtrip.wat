;; tests/types/probe_arc234_7a_base_record_roundtrip.wat — co-located fixture
;;
;; Arc 234 Stone 234.7a — base wat__Record round-trips on the EDN wire.

(:wat::core::defrecord :test::rd::Pt [x <- :wat::core::i64  y <- :wat::core::i64])

(:wat::core::defn :user::write-pt [] -> :wat::core::String
    (:wat::core::let [p (:test::rd::Pt 3 4)]
        (:wat::edn::write p)))

(:wat::core::defn :user::roundtrip-eq [] -> :wat::core::bool
    (:wat::core::let
        [p  (:test::rd::Pt 3 4)
         s  (:wat::edn::write p)
         p2 (:wat::edn::read s)]
        (:wat::core::= p p2)))


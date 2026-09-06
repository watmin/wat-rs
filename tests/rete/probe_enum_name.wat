;; POSITIVE — :wat::rete::core::variant-name in a :then, and :wat::core::variant-name
;; as an ordinary wat call (the core reader must stand alone).
(:wat::core::defenum :en::K :wat::enum::Pure :Aa [] :Bb [])
(:wat::core::defrecord :en::Box [label <- :wat::core::String])
(:wat::core::defrecord :en::Src [k <- :en::K])

(:wat::rete::defrule :en::render
  :when [(:en::Src (?k <- :k))]
  :then [(:en::Box :label (:wat::rete::core::variant-name ?k))])

(:wat::rete::defquery :en::q-Box
  :params []
  :when [(:en::Box (?label <- :label))])

(:wat::core::defn :user::direct [] -> :wat::core::String
  (:wat::core::variant-name (:en::K::Bb)))

(:wat::core::defn :user::via-then [] -> :wat::core::String
  (:wat::core::let
    [rules (:wat::rete::collect-rules :en)
     s0    (:wat::rete::insert
             (:wat::rete::compile-all rules (:wat::core::PersistentVector (:en::q-Box)))
             (:en::Src :k (:en::K::Bb)))
     fired (:wat::rete::fire-rules s0)
     hits  (:wat::rete::query fired (:en::q-Box))]
    (:wat::core::Option/expect
      (:wat::map::get (:wat::core::first hits) "?label")
      "q-Box: ?label")))

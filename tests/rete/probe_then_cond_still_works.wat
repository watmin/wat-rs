;; POSITIVE — cond with a terminal :else in a :then still works. C refuses match, not cond.
(:wat::core::defenum :cd::K :wat::enum::Pure :Aa [] :Bb [])
(:wat::core::defrecord :cd::Box [label <- :wat::core::String])
(:wat::core::defrecord :cd::Src [k <- :cd::K])

(:wat::rete::defrule :cd::go
  :when [(:cd::Src (?k <- :k))]
  :then [(:cd::Box :label (:wat::rete::core::cond
           ((:wat::rete::core::enum::= ?k (:cd::K::Bb)) "bb")
           (:else "other")))])

(:wat::rete::defquery :cd::q-Box
  :params []
  :when [(:cd::Box (?label <- :label))])

(:wat::core::defn :user::run [] -> :wat::core::String
  (:wat::core::let
    [rules (:wat::rete::collect-rules :cd)
     s0    (:wat::rete::insert
             (:wat::rete::compile-all rules (:wat::core::PersistentVector (:cd::q-Box)))
             (:cd::Src :k (:cd::K::Bb)))
     fired (:wat::rete::fire-rules s0)
     hits  (:wat::rete::query fired (:cd::q-Box))]
    (:wat::core::Option/expect
      (:wat::map::get (:wat::core::first hits) "?label")
      "q-Box: ?label")))

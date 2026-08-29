;; The same arm, inside a rete `where`. Subject is a bound HolonAST-free record field.
(:wat::core::defrecord :md::Point [x <- :wat::core::i64  y <- :wat::core::i64])
(:wat::core::defrecord :md::In  [k <- :wat::core::String  p <- :md::Point])
(:wat::core::defrecord :md::Out [k <- :wat::core::String])

(:wat::rete::defrule :md::rule
  :when
  [(:md::In (?k <- :k) (?p <- :p))
   (:wat::rete::where
     (:wat::rete::core::i64::=
       (:wat::rete::core::match ?p
         ({vx :x  vy :y} (:wat::rete::core::i64::+ vx vy :undefined 0)))
       42))]
  :then
  [(:md::Out :k ?k)])

(:wat::rete::defquery :md::q :params [] :when [(?fact <- :md::Out)])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println
    (:wat::core::length
      (:wat::core::let
        [rules   (:wat::rete::collect-rules :md)
         session (:wat::rete::compile-all rules (:wat::core::PersistentVector (:md::q)))
         session (:wat::rete::insert session (:md::In :k "hit"  :p (:md::Point :x 40 :y 2)))
         session (:wat::rete::insert session (:md::In :k "miss" :p (:md::Point :x 1 :y 1)))
         fired   (:wat::rete::fire-rules session)]
        (:wat::rete::query fired (:md::q))))))

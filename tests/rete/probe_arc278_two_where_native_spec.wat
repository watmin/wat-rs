;; Two standalone `:where` clauses — mid-chain AND trailing.
;;
;; Native `fire-rules` used to derive nothing; `fire-rules$oracle` derived Oslo.
;; Spec is the oracle. Native is the user path. They must agree. Clara
;; where-join-order rows 5–6 are this chain (Test → HashJoin → Test).

(:wat::core::defrecord :tw::Temp [c <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :tw::Wind [kph <- :wat::core::i64 loc <- :wat::core::String])
(:wat::core::defrecord :tw::ColdWindy [loc <- :wat::core::String])

(:wat::rete::defrule :tw::cold-and-windy
  :when [(:tw::Temp (?loc <- :loc) (?c <- :c))
         (:wat::rete::where (:wat::rete::i64::< ?c 20))
         (:tw::Wind (?loc <- :loc) (?k <- :kph))
         (:wat::rete::where (:wat::rete::i64::> ?k 30))]
  :then [(:tw::ColdWindy :loc ?loc)])

(:wat::rete::defquery :tw::q-ColdWindy
  :params []
  :when [(?fact <- :tw::ColdWindy)])


(:wat::core::defn :user::stage [] -> :wat::rete::Session
  (:wat::rete::insert
    (:wat::rete::compile-all (:wat::rete::collect-rules :tw) (:wat::core::PersistentVector (:tw::q-ColdWindy)))
    (:tw::Temp :c 5 :loc "oslo")
    (:tw::Wind :kph 40 :loc "oslo")
    (:tw::Temp :c 22 :loc "rome")
    (:tw::Wind :kph 35 :loc "rome")))

(:wat::core::defn :user::native-count [] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query (:wat::rete::fire-rules (:user::stage)) (:tw::q-ColdWindy))))

(:wat::core::defn :user::spec-count [] -> :wat::core::i64
  (:wat::core::length
    (:wat::rete::query (:wat::rete::fire-rules$oracle (:user::stage)) (:tw::q-ColdWindy))))

;; #49 — compiled where. lower at rule-compile; exec == eval-test.

(:wat::core::defrecord :eir::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :eir::Hit [c <- :wat::core::i64])

(:wat::rete::defquery :eir::q-Hit :params [] :when [(?fact <- :eir::Hit)])

(:wat::rete::defrule :eir::cool
  :when [(:eir::Temp (?c <- :c))
         (:wat::rete::where (:wat::rete::i64::< ?c 20))]
  :then [(:eir::Hit ?c)])

(:wat::core::defn :user::cmp-eval [] -> :wat::core::bool
  (:wat::rete::eval-test
    (:wat::core::quote (:wat::rete::i64::< ?c 20))
    (:wat::core::PersistentMap "?c" 10)))

(:wat::core::defn :user::cmp-lower-ok [] -> :wat::core::nil
  (:wat::rete::lower (:wat::core::quote (:wat::rete::i64::< ?c 20))))

(:wat::core::defn :user::fire-cool [] -> :wat::core::i64
  (:wat::core::let [s0 (:wat::rete::compile-all
                         (:wat::core::PersistentVector (:eir::cool))
                         (:wat::core::PersistentVector (:eir::q-Hit)))
                    s1 (:wat::rete::insert s0 (:eir::Temp :c 10))
                    s2 (:wat::rete::insert s1 (:eir::Temp :c 30))
                    fired (:wat::rete::fire-rules s2)]
    (:wat::core::length (:wat::rete::query fired (:eir::q-Hit)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:user::fire-cool)))

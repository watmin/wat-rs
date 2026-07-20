;; Dump the macroexpansion of sift-rules-defsvc to inspect the generated service body directly.

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [form (:wat::core::quote
            (:wat::query::sift-rules-defsvc
              :name :usr::my-sift
              :defs [(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
                     (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
                     (:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])]
              :rules [(:wat::rete::defrule :usr::hot-rule
                        :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
                        :then (:wat::rete::insert (:usr::Hot :c ?c)))
                      (:wat::rete::defrule :usr::warn-rule
                        :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
                        :then (:wat::rete::insert (:usr::Warn :c ?c)))]))
     expanded (:wat::core::macroexpand form)
     src      (:wat::core::ast->source expanded)]
    (:wat::kernel::println src)))

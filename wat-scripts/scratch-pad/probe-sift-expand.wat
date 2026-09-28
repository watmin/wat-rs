(:wat::core::defrecord :usr::Temp [c <- wat.type/i64])
(:wat::core::defrecord :usr::Hot  [c <- wat.type/i64])
(:wat::core::defrecord :usr::Warn [c <- wat.type/i64])

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [form (:wat::core::quote
            (:wat::query::sift-rules-defsvc
              :name :usr::my-sift
              :defs [(:wat::core::defrecord :usr::Temp [c <- wat.type/i64])
                     (:wat::core::defrecord :usr::Hot  [c <- wat.type/i64])
                     (:wat::core::defrecord :usr::Warn [c <- wat.type/i64])]
              :rules [(:wat::rete::defrule :usr::hot-rule
                        :when [(:usr::Temp (?c :- :c) (:wat::core::> ?c 50))]
                        :then [(:usr::Hot :c ?c)])
                      (:wat::rete::defrule :usr::warn-rule
                        :when [(:usr::Temp (?c :- :c) (:wat::core::> ?c 50))]
                        :then [(:usr::Warn :c ?c)])]))
     expanded (:wat::core::macroexpand-1 form)
     src (:wat::core::ast->source expanded)]
    (:wat::kernel::println src)))

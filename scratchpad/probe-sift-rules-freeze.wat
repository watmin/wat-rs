;; Freeze-check: does sift-rules-defsvc EXPAND + FREEZE cleanly (Option A hoists its surface
;; :messages), and do the generated request accessors mint? No Journal, no /start — just prove
;; the macro produces a valid frozen program + a callable accessor. (crashes now surface, so a
;; failure is diagnosable, not a deadlock.)

(:wat::query::sift-rules-defsvc
  :name  :usr::mysift
  :defs  [(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
          (:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
          (:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])]
  :rules [(:wat::rete::defrule :usr::hot-rule
            :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
            :then (:wat::rete::insert (:usr::Hot :c ?c)))
          (:wat::rete::defrule :usr::warn-rule
            :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
            :then (:wat::rete::insert (:usr::Warn :c ?c)))])

;; the surface's generated request accessor must mint (Option A hoisted :messages).
(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [req (:usr::mysift::SiftRulesRequest :namespace "ns" :time-lo 0 :time-hi 100 :limit 50)
     ns  (:usr::mysift::SiftRulesRequest/namespace req)]
    (:wat::kernel::println (:wat::core::string::concat "freeze ok, ns=" ns))))

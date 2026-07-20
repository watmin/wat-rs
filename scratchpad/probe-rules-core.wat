;; #6 rete-core crux (disconfirming probe): the three things the Rules-form op rests on —
;;   (1) per-item RESET is FREE via value-semantics: compile the rules ONCE → a template Session
;;       (WM empty); each item = (insert template seed) from the SAME template → fire → query.
;;       A Session is an immutable value, so the template is never mutated; one seed never
;;       poisons the next (alpha-only structural: one base fact per fire).
;;   (2) rete INFERS: one hot seed fires TWO rules → TWO deductions (output > input).
;;   (3) heterogeneous DEDUCTIONS collect into a (Vector :Value): a Hot AND a Warn in one PV<Value>
;;       (the reply-wire carrier the caller, holding the defs, matches back).

(:wat::core::defrecord :usr::Temp [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Hot  [c <- :wat::core::i64])
(:wat::core::defrecord :usr::Warn [c <- :wat::core::i64])

(:wat::rete::defrule :usr::hot-rule
  :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
  :then (:wat::rete::insert (:usr::Hot :c ?c)))

(:wat::rete::defrule :usr::warn-rule
  :when [(:usr::Temp (?c <- :c) (:wat::core::> ?c 50))]
  :then (:wat::rete::insert (:usr::Warn :c ?c)))

;; deduce-one: fire ONE seed from the fresh template, flat-map its deductions into a PV<Value>
;; (Hot's + Warn's — heterogeneous, up-cast to the universal top :wat::core::Value).
(:wat::core::defn :usr::deduce-one
  [template <- :wat::rete::Session  seed <- :usr::Temp]
  -> :wat::core::PersistentVector<wat::core::Value>
  (:wat::core::let
    [fired (:wat::rete::fire-rules (:wat::rete::insert template seed))
     hots  (:wat::rete::query fired :usr::Hot)
     warns (:wat::rete::query fired :usr::Warn)
     acc0  (:wat::core::foldl
             (:wat::core::fn [a <- :wat::core::PersistentVector<wat::core::Value>  h <- :usr::Hot]
               -> :wat::core::PersistentVector<wat::core::Value>
               (:wat::core::PersistentVector/conj a h))
             (:wat::core::PersistentVector)
             hots)]
    (:wat::core::foldl
      (:wat::core::fn [a <- :wat::core::PersistentVector<wat::core::Value>  w <- :usr::Warn]
        -> :wat::core::PersistentVector<wat::core::Value>
        (:wat::core::PersistentVector/conj a w))
      acc0
      warns)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [rules    (:wat::core::PersistentVector (:usr::hot-rule) (:usr::warn-rule))
     template (:wat::rete::compile rules)
     hot      (:usr::deduce-one template (:usr::Temp :c 60))   ;; expect 2 deductions (Hot + Warn)
     cold     (:usr::deduce-one template (:usr::Temp :c 10))   ;; expect 0 (below threshold)
     total    (:wat::core::+ (:wat::core::length hot) (:wat::core::length cold))]
    (:wat::core::do
      (:wat::kernel::println (:wat::core::string::concat "hot="   (:wat::core::str (:wat::core::length hot))))
      (:wat::kernel::println (:wat::core::string::concat "cold="  (:wat::core::str (:wat::core::length cold))))
      (:wat::kernel::println (:wat::core::string::concat "total=" (:wat::core::str total))))))

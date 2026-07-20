;; control for probe-sift-rules-stop1.wat — the SAME defsurface/defservice shape, HAND-WRITTEN
;; (no wrapping defmacro) — isolates whether the earlier UnresolvedReferences (FireRequest/c,
;; FireResponse::Deduction) is a macro-emission artifact or a plain naming/syntax mistake.

(:wat::core::defrecord :probe::Temp [c <- :wat::core::i64])

(:wat::core::defsurface :probe::Svc :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Hot [c <- :wat::core::i64])
   (:wat::core::defrecord :probe::Svc::FireRequest [c <- :wat::core::i64])
   (:wat::core::defenum :probe::Svc::FireResponse :wat::enum::Pure
     :Deduction [n <- :wat::core::i64])]
  :features
  [(fire [self <- :probe::Svc req <- :probe::Svc::FireRequest] -> :probe::Svc::FireResponse)])

(:wat::rete::defrule :probe::hot
  :when [(:probe::Temp (?c <- :c) (:wat::core::> ?c 50))]
  :then (:wat::rete::insert (:probe::Hot :c ?c)))

(:wat::service::defservice :probe::svc'
  :satisfies :probe::Svc
  :durable []
  :impls
  [(fire [s req]
     (:wat::core::let
       [c     (:probe::Svc::FireRequest/c req)
        item  (:probe::Temp :c c)
        rules (:wat::core::PersistentVector (:probe::hot))
        sess0 (:wat::rete::compile rules)
        s1    (:wat::rete::insert sess0 item)
        fired (:wat::rete::fire-rules s1)
        ded   (:wat::rete::query fired :probe::Hot)]
       (:wat::service::Outcome::Reply s
         (:probe::Svc::FireResponse::Deduction (:wat::core::count ded)))))])

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h    (:probe::svc'/start :locus (:wat::spawn::process) :record (:probe::svc'::Record))
     addr (:probe::svc'::Handle/addr h)
     cli  (:wat::kernel::connect' addr)
     hot  (:probe::Svc/fire cli (:probe::Svc::FireRequest :c 99))
     cold (:probe::Svc/fire cli (:probe::Svc::FireRequest :c 1))]
    (:wat::kernel::println (:wat::core::str "hot=" hot " cold=" cold))))

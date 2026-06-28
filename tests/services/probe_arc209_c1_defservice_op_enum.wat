(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Get [s <- :State]
         -> [value <- :wat::core::i64]
     (:wat::service::Outcome::Reply s (:my::counter::GetResponse (:my::counter::Record/count (:my::counter::State/durable s)))))

   (:Increment [s <- :State n <- :wat::core::i64]
               -> [value <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply (:my::counter::State/new (:my::counter::Record c)) (:my::counter::IncrementResponse c))))])

;; Exercise the GENERATED op enum (wrapped-record C.3 shape):
;;   1. Build an IncrementRequest via the generated constructor.
;;   2. Wrap it in the Op::Increment variant.
;;   3. Match: Get arm returns 0 (proves Get variant exists + wraps GetRequest);
;;      Increment arm extracts n via IncrementRequest/n accessor → 5.
(:wat::core::defn :user::probe-op [] -> :wat::core::i64
  (:wat::core::let [req (:my::counter/increment-request 5)
                    op  (:my::counter::Op::Increment req)]
    (:wat::core::match op -> :wat::core::i64
      ((:my::counter::Op::Get _r) 0)
      ((:my::counter::Op::Increment req) (:my::counter::IncrementRequest/n req)))))

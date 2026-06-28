;; tests/process/probe_arc272_rs2_process_stop_returns_final_state.wat — co-located fixture.
;; startup_beside(file!()) world — counter service on process locus; stop returns final Record.

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

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start :locus (:wat::spawn::process) :record (:my::counter::Record 0))
     c     (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _     (:my::counter/increment c (:my::counter/increment-request 5))
     final (:my::counter/stop h)]
    (:my::counter::Record/count final)))


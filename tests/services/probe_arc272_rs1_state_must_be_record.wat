;; Base (default) — :durable [fields] mints ::Record (the soul); ::State is a defstruct.
;; Handler reads through State/durable, builds next State via State/new, stop returns ::Record.
(:wat::service::defservice :my::counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :ops
  [(:Increment [s <- :State n <- :wat::core::i64]
               -> [count <- :wat::core::i64]
     (:wat::core::let [c (:wat::core::i64::+ (:my::counter::Record/count (:my::counter::State/durable s)) n)]
       (:wat::service::Outcome::Reply (:my::counter::State (:my::counter::Record c)) (:my::counter::IncrementResponse c))))])

(:wat::core::defn :user::compute [] -> :wat::core::i64
  (:wat::core::let
    [h     (:my::counter/start :locus (:wat::spawn::thread) :record (:my::counter::Record 0))
     c     (:wat::kernel::connect' (:my::counter::Handle/addr h))
     _     (:my::counter/increment c (:my::counter/increment-request 5))
     ;; arc 291 3a-ii-β: stop is owner-only — takes the Handle (h), not the client peer (c).
     ;; arc 291 4b-ii: stop returns ::Record (durable soul); read count via Record/count.
     final (:my::counter/stop h)]
    (:my::counter::Record/count final)))

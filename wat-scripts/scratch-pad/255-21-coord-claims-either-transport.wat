;; wat-scripts/scratch-pad/255-21-coord-claims-either-transport.wat — stone 255.21 (C-b1b), STOPPED.
;;
;; WITNESS of the hole the stone was drawn to close. BOTH defns below type-check (rc=0) on main
;; @ 50ad6be53: a THREAD service handle's `Dialable/coord` claimed a Wire address, and a PROCESS
;; handle's claimed a Shared one. Today coord's scheme returns the 2-arg `(Address :- [Op Reply])`
;; and the missing-slot arm admits either transport.
;;
;; The stone's surface change alone (`Dialable :- [S R T]`, defservice's edge binding the Handle's
;; own letter) does NOT close it: measured, coord on `(echo::Handle :- [Shared])` then returns
;; `(Address :- [Echo::Op Echo::Reply :T])` — the scheme is found under the letter-rewritten key
;; `(Handle :- [:T])/coord` and the Handle's letter is never bound back to the receiver's Shared, so
;; `:T` instantiates to Wire or Shared alike. Both defns still check rc=0 on that binary.
;; When this file stops type-checking, the hole is closed — move it to a .wat.bad row then.
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure
     :Ok              [reply <- :wat::core::String]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable []  :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome.Reply {:state s
              :reply (:probe::Echo::EchoResponse.Ok {:reply (:probe::Echo::EchoRequest/msg req)})}))])

(:wat::core::defn :probe::thread-coord-claimed-wire []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Wire])
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::thread) :record (:probe::echo::Record))]
    (:wat::capability::Dialable/coord h)))

(:wat::core::defn :probe::process-coord-claimed-shared []
  -> (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply :wat::kernel::Shared])
  (:wat::core::let
    [h (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))]
    (:wat::capability::Dialable/coord h)))

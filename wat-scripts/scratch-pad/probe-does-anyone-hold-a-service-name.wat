;; probe-does-anyone-hold-a-service-name.wat
;;
;; defservice emits a per-op client method under the SERVICE's own name
;; (`fqdn-base`, wat/service.wat:276) — `:queue::queue/send`. Path B in
;; src/runtime.rs handles the SURFACE name — `:queue::Queue/send`. Both are
;; client methods; both now carry their own `:max-entries` guard, written
;; separately (wat quasiquote vs hand-assembled WatAST).
;;
;; Measured across the corpus: Queue/send 21 uses, queue/send 0. Store/put 30,
;; *-store/put 0. Nothing has ever called a service-name op method.
;;
;; The open question this settles: are they a real alias, or were they never
;; callable? A client holds (Peer :- [Queue::Op Queue::Reply]) — typed by the
;; SURFACE — so nothing may even be in a position to call the service spelling.
;;
;; Refutation: if `:queue::queue/send` does not resolve, they are not an
;; unused alias, they are uncallable, and deleting them is a fact not a policy.
;; If it resolves and agrees, the guard duplication is at least OBSERVABLE.

(:wat::config::set-redef! true)
(:wat::load-file! "../queue/sqs.wat")

(:wat::core::defn :sn::bodies [n <- :wat::core::i64] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [:wat::core::String]) i <- :wat::core::i64]
      -> (:wat::core::Vector :- [:wat::core::String])
      (:wat::core::conj acc (:wat::core::format "b{i}" :i i)))
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::range 0 n)))

(:wat::core::defn :sn::tag
  [r <- (:wat::kernel::RecvOutcome :- [:queue::Queue::SendResponse])] -> :wat::core::String
  (:wat::core::match r
    ((:wat::kernel::RecvOutcome::Message m)
      (:wat::core::match m
        ((:queue::Queue::SendResponse::Accepted n)
          (:wat::core::format "Accepted({n})" :n n))
        ((:queue::Queue::SendResponse::RequestTooLarge _b _c) "RequestTooLarge")
        ((:queue::Queue::SendResponse::RequestTooManyEntries e c)
          (:wat::core::format "RequestTooManyEntries({e},{c})" :e e :c c))
        ((:queue::Queue::SendResponse::RequestMalformed _p _e _g) "RequestMalformed")))
    (_ "recv-failed")))

(:wat::core::defn :sn::req [n <- :wat::core::i64] -> :queue::Queue::SendRequest
  (:queue::Queue::SendRequest
    :queue "q" :bodies (:sn::bodies n)
    :now-ns (:wat::time::epoch-nanos (:wat::time::now))))

;; THE ONE VARIABLE: same peer, same request, two spellings of the same call.
(:wat::core::defn :sn::via-surface [q <- :queue::Queue  n <- :wat::core::i64] -> :wat::core::String
  (:sn::tag (:queue::Queue/send q (:sn::req n))))

(:wat::core::defn :sn::via-service
  [q <- (:wat::kernel::Peer :- [:queue::Queue::Op :queue::Queue::Reply])  n <- :wat::core::i64]
  -> :wat::core::String
  (:sn::tag (:queue::queue/send q (:sn::req n))))

(:wat::core::defn :sn::depth [q <- :queue::Queue] -> :wat::core::i64
  (:wat::core::match (:queue::Queue/stats q (:queue::Queue::StatsRequest))
    ((:wat::kernel::RecvOutcome::Message r)
      (:wat::core::match r
        ((:queue::Queue::StatsResponse::Ok _calls _ticks visible _unacked _ _ _) visible)
        (_ (:wat::kernel::assertion-failed! "sn: stats not Ok" :wat::core::None :wat::core::None))))
    (_ (:wat::kernel::assertion-failed! "sn: stats recv failed" :wat::core::None :wat::core::None))))

(:wat::core::defn :sn::run [] -> :wat::core::String
  (:wat::core::let
    [sh (:wat::query::mem-store/start :locus (:wat::spawn::thread)
          :record (:wat::query::mem-store::Record
                    :rows (:wat::core::PersistentVector :- [:wat::query::StoredRow])))
     qh (:queue::queue/start :locus (:wat::spawn::thread)
          :record (:queue::queue::Record :cap 64
                    :store-addr (:wat::query::mem-store::Handle/addr sh)
                    :drop-recv-bp 0 :drop-ack-bp 0 :drop-seed 0))
     q  (:wat::core::match (:wat::kernel::connect (:queue::queue::Handle/addr qh))
          ((:wat::kernel::ConnectOutcome::Connected c) c)
          (_ (:wat::kernel::assertion-failed! "sn: dial failed" :wat::core::None :wat::core::None)))
     ;; over the cap, both spellings
     s11 (:sn::via-surface q 11)   d1 (:sn::depth q)
     v11 (:sn::via-service q 11)   d2 (:sn::depth q)
     ;; at the cap, both spellings
     s10 (:sn::via-surface q 10)   d3 (:sn::depth q)
     v10 (:sn::via-service q 10)   d4 (:sn::depth q)]
    (:wat::core::format
      "surface-11={a};depth={p};service-11={b};depth={q};surface-10={c};depth={r};service-10={d};depth={s}"
      :a s11 :p d1 :b v11 :q d2 :c s10 :r d3 :d v10 :s d4)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:sn::run)))

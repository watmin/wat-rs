;; Arc 170 gap J — `each` + kwargs tail (the success-gate's "each+tail" proof).
;;
;; `map`/`each` share the identical `:name val` tail-parse (wat/bracket.wat) — this proves the
;; kwargs-provisioning layer rides `each`, not just `map`: a durable-counter service is
;; granted+dialed via `each`'s OWN tail (no `process/uses`, no `bracket/uses`/`uses'` — all
;; retired this stone), each worker's side effect (increment) fires for every item, `each`
;; itself returns nil, and the counter's durable state afterward equals the item count (every
;; item incremented exactly once — no double-count, no drop).
;;
;; This test FORKS processes (the counter service + N pool workers) — run --test-threads=1:
;; cargo nextest run -p wat -E 'test(/probe_arc170_gapj_each_kwargs/)' --test-threads=1

(:wat::core::defsurface :probe::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Counter::GetRequest       [])
   (:wat::core::defenum :probe::Counter::GetResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :probe::Counter::IncrementRequest  [n <- :wat::core::i64])
   (:wat::core::defenum :probe::Counter::IncrementResponse :wat::enum::Pure
     :Ok              [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(get       [self <- :probe::Counter  req <- :probe::Counter::GetRequest]       -> :probe::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :probe::Counter  req <- :probe::Counter::IncrementRequest] -> :probe::Counter::IncrementResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::counter
  :satisfies :probe::Counter
  :durable [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Reply s
       (:probe::Counter::GetResponse::Ok
         (:probe::counter::Record/count (:probe::counter::State/durable s)))))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::core::i64::+
                           (:probe::counter::Record/count (:probe::counter::State/durable s))
                           (:probe::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Reply
         (:probe::counter::State :durable (:probe::counter::Record :count c))
         (:probe::Counter::IncrementResponse::Ok c))))])

;; kwargs work-fn: item positional, `counter` a dialed `:key` kwarg (grant+Setup ride `each`'s
;; own tail). The side effect is the increment; the return value is discarded by `each`.
(:wat::core::defn :probe::record-hit
  [item <- :wat::core::String
   & [counter <- (:wat::kernel::Peer :- [:probe::Counter::Op :probe::Counter::Reply])]]
  -> :wat::core::i64
  (:wat::core::match
    (:probe::Counter/increment counter (:probe::Counter::IncrementRequest :n 1)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
    ((:probe::Counter::IncrementResponse::Ok value) value)
    ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
    ((:probe::Counter::IncrementResponse::RequestTooLarge bytes cap)
      (:wat::kernel::assertion-failed! "record-hit: unexpected RequestTooLarge"
        :wat::core::None :wat::core::None))
    ((:probe::Counter::IncrementResponse::RequestMalformed mpath mexpected mgot)
      (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))

;; `:probe::run` (a non-main defn — no `:user::main`; only freezes + is called directly).
;; Returns (each's own return value, the counter's final durable count) so the Rust driver can
;; assert BOTH halves of the success gate: `each` returns nil, and every item's side effect fired.
(:wat::core::defn :probe::run [] -> :(wat::core::nil,wat::core::i64)
  (:wat::core::let
    [h        (:probe::counter/start :locus (:wat::spawn::process) :record (:probe::counter::Record :count 0))
     each-out (:wat::bracket::each (:wat::spawn::process) ["a" "b" "c" "d" "e"] :probe::record-hit :counter h)
     c        (:wat::core::match (:wat::kernel::connect (:probe::counter::Handle/addr h)) ((:wat::kernel::ConnectOutcome::Connected p) p) ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)) ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None)))
     r        (:probe::Counter/get c (:probe::Counter::GetRequest))]
    (:wat::core::Tuple each-out
      (:wat::core::match r ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
        ((:probe::Counter::GetResponse::Ok value) value)
        ;; terminal caller: an unexpected wire-breach must SURFACE, never swallow.
        ((:probe::Counter::GetResponse::RequestTooLarge bytes cap)
          (:wat::kernel::assertion-failed! "run: unexpected RequestTooLarge"
            :wat::core::None :wat::core::None))
        ((:probe::Counter::GetResponse::RequestMalformed mpath mexpected mgot)
          (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None))))))

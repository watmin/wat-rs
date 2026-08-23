;; Arc 170 C2 — Strike 1 (RUNTIME) hand-wired MIXED proof — NO `bracket/uses` macro (Strike 2).
;;
;; 12 kwargs: 7 heterogeneous `Peer'` SERVICE kwargs (s1..s7, each dialed) + 5 DATA kwargs
;; (d1..d5, copied as EDN). Proves the record-carrier runtime in one shot:
;;  - NO service-count cap (N=7 > the retired first/second/third=3 positional-accessor limit) —
;;    the coords carrier is now the NAMED record `:probe::enrich::Coords`, reconciled BY FIELD NAME.
;;  - the DATA path — 5 data fields ride inside the same record, copied (not dialed), routed off
;;    `field-types-of` (Peer'-vs-not) in the generated dial-runner.
;;
;; Chain exercised end to end:
;;  - Strike 1a (wat/core.wat): the kwargs-defn site mints `:probe::enrich::Coords` (a defrecord,
;;    head-swapped Peer'→Address', same field names/order as ::Kwargs) + the checker
;;    `:probe::enrich::kwargs-check` that RETURNS that record (its return value IS `coords` below).
;;  - Strike 1b (wat/bracket.wat process-work-forms): the N-generalized, cap-free dial-runner that
;;    reconciles ::Coords → ::Kwargs by field name (connect' the 7 Peer' fields, copy the 5 data).
;;  - Strike 1c (wat/bracket.wat :wat::bracket::uses'): ONE `PoolMsg::Setup(coords-record)` per worker.

;; ── 7 heterogeneous services ─────────────────────────────────────────────────
(:wat::core::defsurface :probe::S1 :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::S1::OpRequest [m <- :wat::core::String])
             (:wat::core::defenum :probe::S1::OpResponse :wat::enum::Pure
               :Ok              [r <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(op [self <- :probe::S1  req <- :probe::S1::OpRequest] -> :probe::S1::OpResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::s1 :satisfies :probe::S1 :durable [] :ephemeral []
  :impls [(op [s ctx req] (:wat::service::Outcome::Reply s
            (:probe::S1::OpResponse::Ok (:wat::core::string::concat "s1:" (:probe::S1::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S2 :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::S2::OpRequest [m <- :wat::core::String])
             (:wat::core::defenum :probe::S2::OpResponse :wat::enum::Pure
               :Ok              [r <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(op [self <- :probe::S2  req <- :probe::S2::OpRequest] -> :probe::S2::OpResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::s2 :satisfies :probe::S2 :durable [] :ephemeral []
  :impls [(op [s ctx req] (:wat::service::Outcome::Reply s
            (:probe::S2::OpResponse::Ok (:wat::core::string::concat "s2:" (:probe::S2::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S3 :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::S3::OpRequest [m <- :wat::core::String])
             (:wat::core::defenum :probe::S3::OpResponse :wat::enum::Pure
               :Ok              [r <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(op [self <- :probe::S3  req <- :probe::S3::OpRequest] -> :probe::S3::OpResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::s3 :satisfies :probe::S3 :durable [] :ephemeral []
  :impls [(op [s ctx req] (:wat::service::Outcome::Reply s
            (:probe::S3::OpResponse::Ok (:wat::core::string::concat "s3:" (:probe::S3::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S4 :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::S4::OpRequest [m <- :wat::core::String])
             (:wat::core::defenum :probe::S4::OpResponse :wat::enum::Pure
               :Ok              [r <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(op [self <- :probe::S4  req <- :probe::S4::OpRequest] -> :probe::S4::OpResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::s4 :satisfies :probe::S4 :durable [] :ephemeral []
  :impls [(op [s ctx req] (:wat::service::Outcome::Reply s
            (:probe::S4::OpResponse::Ok (:wat::core::string::concat "s4:" (:probe::S4::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S5 :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::S5::OpRequest [m <- :wat::core::String])
             (:wat::core::defenum :probe::S5::OpResponse :wat::enum::Pure
               :Ok              [r <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(op [self <- :probe::S5  req <- :probe::S5::OpRequest] -> :probe::S5::OpResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::s5 :satisfies :probe::S5 :durable [] :ephemeral []
  :impls [(op [s ctx req] (:wat::service::Outcome::Reply s
            (:probe::S5::OpResponse::Ok (:wat::core::string::concat "s5:" (:probe::S5::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S6 :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::S6::OpRequest [m <- :wat::core::String])
             (:wat::core::defenum :probe::S6::OpResponse :wat::enum::Pure
               :Ok              [r <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(op [self <- :probe::S6  req <- :probe::S6::OpRequest] -> :probe::S6::OpResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::s6 :satisfies :probe::S6 :durable [] :ephemeral []
  :impls [(op [s ctx req] (:wat::service::Outcome::Reply s
            (:probe::S6::OpResponse::Ok (:wat::core::string::concat "s6:" (:probe::S6::OpRequest/m req)))))])

(:wat::core::defsurface :probe::S7 :nature :wat::kernel::Peer
  :messages [(:wat::core::defrecord :probe::S7::OpRequest [m <- :wat::core::String])
             (:wat::core::defenum :probe::S7::OpResponse :wat::enum::Pure
               :Ok              [r <- :wat::core::String]
               :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
               :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features [(op [self <- :probe::S7  req <- :probe::S7::OpRequest] -> :probe::S7::OpResponse :max-request-bytes 524288)])
(:wat::service::defservice :probe::s7 :satisfies :probe::S7 :durable [] :ephemeral []
  :impls [(op [s ctx req] (:wat::service::Outcome::Reply s
            (:probe::S7::OpResponse::Ok (:wat::core::string::concat "s7:" (:probe::S7::OpRequest/m req)))))])

;; ── the work-fn: item POSITIONAL; 7 Peer' service kwargs + 5 String data kwargs ──
(:wat::core::defn :probe::enrich
  [item <- :wat::core::String
   & [s1 <- (:wat::kernel::Peer :- [:probe::S1::Op :probe::S1::Reply])
      s2 <- (:wat::kernel::Peer :- [:probe::S2::Op :probe::S2::Reply])
      s3 <- (:wat::kernel::Peer :- [:probe::S3::Op :probe::S3::Reply])
      s4 <- (:wat::kernel::Peer :- [:probe::S4::Op :probe::S4::Reply])
      s5 <- (:wat::kernel::Peer :- [:probe::S5::Op :probe::S5::Reply])
      s6 <- (:wat::kernel::Peer :- [:probe::S6::Op :probe::S6::Reply])
      s7 <- (:wat::kernel::Peer :- [:probe::S7::Op :probe::S7::Reply])
      d1 <- :wat::core::String
      d2 <- :wat::core::String
      d3 <- :wat::core::String
      d4 <- :wat::core::String
      d5 <- :wat::core::String]]
  -> :wat::core::String
  (:wat::core::let
    [r1  (:wat::core::match (:probe::S1/op s1 (:probe::S1::OpRequest :m item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
           ((:probe::S1::OpResponse::Ok r) r)
           ((:probe::S1::OpResponse::RequestTooLarge bytes cap)
             (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
           ((:probe::S1::OpResponse::RequestMalformed mpath mexpected mgot)
             (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     r2  (:wat::core::match (:probe::S2/op s2 (:probe::S2::OpRequest :m item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
           ((:probe::S2::OpResponse::Ok r) r)
           ((:probe::S2::OpResponse::RequestTooLarge bytes cap)
             (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
           ((:probe::S2::OpResponse::RequestMalformed mpath mexpected mgot)
             (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     r3  (:wat::core::match (:probe::S3/op s3 (:probe::S3::OpRequest :m item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
           ((:probe::S3::OpResponse::Ok r) r)
           ((:probe::S3::OpResponse::RequestTooLarge bytes cap)
             (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
           ((:probe::S3::OpResponse::RequestMalformed mpath mexpected mgot)
             (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     r4  (:wat::core::match (:probe::S4/op s4 (:probe::S4::OpRequest :m item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
           ((:probe::S4::OpResponse::Ok r) r)
           ((:probe::S4::OpResponse::RequestTooLarge bytes cap)
             (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
           ((:probe::S4::OpResponse::RequestMalformed mpath mexpected mgot)
             (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     r5  (:wat::core::match (:probe::S5/op s5 (:probe::S5::OpRequest :m item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
           ((:probe::S5::OpResponse::Ok r) r)
           ((:probe::S5::OpResponse::RequestTooLarge bytes cap)
             (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
           ((:probe::S5::OpResponse::RequestMalformed mpath mexpected mgot)
             (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     r6  (:wat::core::match (:probe::S6/op s6 (:probe::S6::OpRequest :m item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
           ((:probe::S6::OpResponse::Ok r) r)
           ((:probe::S6::OpResponse::RequestTooLarge bytes cap)
             (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
           ((:probe::S6::OpResponse::RequestMalformed mpath mexpected mgot)
             (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     r7  (:wat::core::match (:probe::S7/op s7 (:probe::S7::OpRequest :m item)) ((:wat::kernel::RecvOutcome::Message __recv) (:wat::core::match __recv 
           ((:probe::S7::OpResponse::Ok r) r)
           ((:probe::S7::OpResponse::RequestTooLarge bytes cap)
             (:wat::kernel::assertion-failed! "enrich: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
           ((:probe::S7::OpResponse::RequestMalformed mpath mexpected mgot)
             (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None)))) ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "recv': peer closed" :wat::core::None :wat::core::None)))
     svc (:wat::core::string::concat r1
           (:wat::core::string::concat r2
             (:wat::core::string::concat r3
               (:wat::core::string::concat r4
                 (:wat::core::string::concat r5
                   (:wat::core::string::concat r6 r7))))))
     dat (:wat::core::string::concat d1
           (:wat::core::string::concat d2
             (:wat::core::string::concat d3
               (:wat::core::string::concat d4 d5))))]
    (:wat::core::string::concat item
      (:wat::core::string::concat "|"
        (:wat::core::string::concat svc dat)))))

;; `:probe::run` (a non-main defn — no `:user::main`; only freezes + is called directly).
(:wat::core::defn :probe::run [] -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [h1 (:probe::s1/start :locus (:wat::spawn::process) :record (:probe::s1::Record))
     h2 (:probe::s2/start :locus (:wat::spawn::process) :record (:probe::s2::Record))
     h3 (:probe::s3/start :locus (:wat::spawn::process) :record (:probe::s3::Record))
     h4 (:probe::s4/start :locus (:wat::spawn::process) :record (:probe::s4::Record))
     h5 (:probe::s5/start :locus (:wat::spawn::process) :record (:probe::s5::Record))
     h6 (:probe::s6/start :locus (:wat::spawn::process) :record (:probe::s6::Record))
     h7 (:probe::s7/start :locus (:wat::spawn::process) :record (:probe::s7::Record))
     ;; Strike 1a/C2-D's checker — the gate AND the carrier-assembly are ONE act. Kwargs are
     ;; ORDER-FREE (scrambled here on purpose); RAW HANDLES (no Dialable/coord upcast — a
     ;; handle satisfies TypedCapability<S,R> directly via the bodiless auto-emit); the checker
     ;; reorders to field order and returns `(::Coords, ::GrantHandles)`.
     pair    (:probe::enrich::kwargs-check
               :d2 "D2" :s3 h3 :s1 h1
               :d5 "D5" :s7 h7 :d1 "D1"
               :s2 h2 :s5 h5
               :d4 "D4" :s4 h4 :d3 "D3"
               :s6 h6)
     coords  (:wat::core::first pair)
     handles (:wat::core::second pair)]
    ;; Arc 170 gap J — `uses'` folded into `map-worker` (param order: locus items worker-init
    ;; grant-handles grant-fn revoke-fn setups; setups is the 0-or-1-element Setup fold vector).
    (:wat::bracket::map-worker (:wat::spawn::process)
      ["a" "b"]
      (:wat::core::fn [_worker-id <- :wat::core::i64] -> :wat::core::keyword :probe::enrich)
      handles
      :probe::enrich::grant-worker
      :probe::enrich::revoke-worker
      (:wat::core::Vector :probe::enrich::Coords coords))))

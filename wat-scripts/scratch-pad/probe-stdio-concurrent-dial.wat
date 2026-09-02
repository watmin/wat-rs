;; probe-stdio-concurrent-dial.wat — arc-170 stdio-as-defservice UNKNOWN (a), the LOAD-BEARING proof.
;;
;; QUESTION: can ONE always-on thread-tier defservice be dialed by N *concurrent* worker
;; threads, each `connect'`ing its OWN Peer' and issuing several ops, with (1) every op getting
;; its own correct typed reply (no cross-talk between the two clients' request/reply streams) and
;; (2) the service state serialized (final counter == the sum of all increments)?
;;
;; The defservice exemplars (probe_arc209_c3 / s2s-*) drive a service SINGLE-THREADED sequentially.
;; This probe hands ONE started counter's Address' to 3 concurrently-spawned worker threads (via
;; `spawn-program' (thread)` — the self-peer model, per wat/test.wat run-thread'), each of which
;; connects and does 4 increments of n=1. 3 workers × 4 = 12 total. Then main dials its OWN peer
;; and reads the final count, asserting it equals 12 (serialization held) and that each worker saw
;; 4 typed Ok replies (no cross-talk / no lost reply).

;; ── The counter surface + service (modelled on probe_arc209_c3_defservice_client_face.wat) ──
(:wat::core::defsurface :probe::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Counter::GetRequest        [])
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
  :durable   [count <- :wat::core::i64]
  :ephemeral []
  :impls
  [(get [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:probe::Counter::Reply::Get (:probe::Counter::GetResponse::Ok (:probe::counter::Record/count (:probe::counter::State/durable s))))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Counter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::counter::Op])])))
   (increment [s ctx req]
     (:wat::core::let [c (:wat::i64::+ (:probe::counter::Record/count (:probe::counter::State/durable s))
                                             (:probe::Counter::IncrementRequest/n req))]
       (:wat::service::Outcome::Continue (:probe::counter::State :durable (:probe::counter::Record :count c))
                                      (:wat::core::Some (:probe::Counter::Reply::Increment (:probe::Counter::IncrementResponse::Ok c))) (:wat::core::Vector :- [(:wat::service::Directed :- [:probe::Counter::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:probe::counter::Op])]))))])

;; ── A recursive worker helper: do `remaining` increments of n=1 on a connected client peer,
;;    counting how many typed IncrementResponse::Ok replies came back (a cross-talk / lost-reply
;;    detector — a garbled or misrouted reply would fail the typed match and raise). ────────────
(:wat::core::defn :probe::do-increments
  [c         <- (:wat::kernel::Peer :- [:probe::Counter::Op :probe::Counter::Reply])
   remaining <- :wat::core::i64
   acc       <- :wat::core::i64]
  -> :wat::core::i64
  (:wat::core::if (:wat::core::= remaining 0)
    acc
    (:wat::core::match (:probe::Counter/increment c (:probe::Counter::IncrementRequest :n 1))
      ((:wat::kernel::RecvOutcome::Message __recv)
        (:wat::core::match __recv
          ((:probe::Counter::IncrementResponse::Ok _v)
            (:probe::do-increments c (:wat::i64::- remaining 1) (:wat::i64::+ acc 1)))
          ((:probe::Counter::IncrementResponse::RequestTooLarge bytes cap)
            (:wat::kernel::assertion-failed! "do-increments: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
          ((:probe::Counter::IncrementResponse::RequestMalformed mpath mexpected mgot)
            (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost __cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message __cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "do-increments: stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "do-increments: peer closed" :wat::core::None :wat::core::None)))))

;; ── A worker body: connect' our OWN client Peer' to the shared Address', do 4 increments,
;;    send the Ok-count back up the self-peer. Factored to a defn so the 3 spawns are identical. ─
(:wat::core::defn :probe::worker-body
  [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])
   addr <- (:wat::kernel::Address :- [:probe::Counter::Op :probe::Counter::Reply])]
  -> :wat::core::nil
  (:wat::core::let
    [c  (:wat::core::match (:wat::kernel::connect addr)
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused cc)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed cc)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))
     ok (:probe::do-increments c 4 0)]
    (:wat::core::match (:wat::kernel::send self ok)
      (:wat::kernel::SendOutcome::Sent    nil)
      (:wat::kernel::SendOutcome::Closed  nil)
      (:wat::kernel::SendOutcome::Stopped nil)
      ((:wat::kernel::SendOutcome::Lost _c) nil))))

;; helper: recv' an i64 result from a joined worker thread peer.
(:wat::core::defn :probe::join-count
  [p <- (:wat::kernel::Thread :- [:wat::core::i64 :wat::core::i64])]
  -> :wat::core::i64
  (:wat::core::match (:wat::kernel::recv p)
    ((:wat::kernel::RecvOutcome::Message m) m)
    ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "join-count: stopped — the substrate was asked to stop; worker was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "join-count: worker closed before signalling" :wat::core::None :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h    (:probe::counter/start :locus (:wat::spawn::thread) :record (:probe::counter::Record :count 0))
     addr (:probe::counter::Handle/addr h)
     ;; spawn ALL THREE workers first (concurrent), each capturing the shared addr — then join.
     w1 (:wat::test::spawn-peer (:wat::spawn::thread)
          (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
            (:probe::worker-body self addr)))
     w2 (:wat::test::spawn-peer (:wat::spawn::thread)
          (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
            (:probe::worker-body self addr)))
     w3 (:wat::test::spawn-peer (:wat::spawn::thread)
          (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:wat::core::i64 :wat::core::i64])] -> :wat::core::nil
            (:probe::worker-body self addr)))
     r1 (:probe::join-count w1)
     r2 (:probe::join-count w2)
     r3 (:probe::join-count w3)
     total-ops (:wat::i64::+ r1 (:wat::i64::+ r2 r3))
     ;; main dials its OWN peer for the final read (h stays alive → service lives).
     mc (:wat::core::match (:wat::kernel::connect addr)
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused cc)  (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed cc)   (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None)))
     final (:wat::core::match (:probe::Counter/get mc (:probe::Counter::GetRequest))
             ((:wat::kernel::RecvOutcome::Message __recv)
               (:wat::core::match __recv
                 ((:probe::Counter::GetResponse::Ok value) value)
                 ((:probe::Counter::GetResponse::RequestTooLarge bytes cap)
                   (:wat::kernel::assertion-failed! "get: unexpected RequestTooLarge" :wat::core::None :wat::core::None))
                 ((:probe::Counter::GetResponse::RequestMalformed mpath mexpected mgot)
                   (:wat::kernel::assertion-failed! "unexpected RequestMalformed" :wat::core::None :wat::core::None))))
             ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "get: stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "get: peer closed" :wat::core::None :wat::core::None)))
     ;; ASSERT: every worker landed its 4 typed replies (no cross-talk / no lost reply).
     _  (:wat::core::if (:wat::core::= total-ops 12) nil
          (:wat::kernel::assertion-failed! "CROSS-TALK / LOST REPLY: total Ok replies != 12" :wat::core::None :wat::core::None))
     ;; ASSERT: serialization held — final counter == sum of all increments.
     _  (:wat::core::if (:wat::core::= final 12) nil
          (:wat::kernel::assertion-failed! "SERIALIZATION LOST: final counter != 12" :wat::core::None :wat::core::None))]
    (:wat::kernel::println
      (:wat::string::concat "PROBE-A GREEN: workers-ok="
        (:wat::string::concat (:wat::i64::to-string total-ops)
          (:wat::string::concat " final-counter="
            (:wat::i64::to-string final)))))))

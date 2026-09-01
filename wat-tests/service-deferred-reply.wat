;; wat-tests/service-deferred-reply.wat — Arc 278 the deferred reply (`Outcome::ReplyTo`).
;;
;; A defservice arm replies to a client OTHER than the one that invoked it, named by
;; `ctx`'s conn-id. The substrate half of long polling; the queue is a separate stone.
;;
;; Gates (EXPECTATIONS-deferred-reply.md):
;;   1. ★ client wakes client — A parks, B wakes, A's recv' returns B's value.
;;   2. ★ a TIMER wakes a client — internal arm returning ReplyTo (STOP-2's target).
;;   3. one call wakes several — two parked, one wake, both return.
;;   4. a vanished waiter is survivable — park, drop the client, wake; service keeps serving.
;;   7. the internal assertion still catches Reply (ReplyTo is the exemption, not a hole).
;;
;; Park/wake is wire-ordered: no sleep. After a raw park send, a ping on the SAME
;; connection is the FIFO barrier that proves the waiter is stored before wake runs.
;; Waiters are conj'd; wake sends in that order (FIFO; no other fairness policy).

;; ── the wire surface ─────────────────────────────────────────────────────────
(:wat::core::defsurface :wat-tests::Parker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::Parker::ParkRequest [])
   (:wat::core::defenum :wat-tests::Parker::ParkResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :wat-tests::Parker::WakeRequest [value <- :wat::core::i64])
   (:wat::core::defenum :wat-tests::Parker::WakeResponse :wat::enum::Pure
     :Ok               [value <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :wat-tests::Parker::AwaitTickRequest [])
   (:wat::core::defenum :wat-tests::Parker::AwaitTickResponse :wat::enum::Pure
     :Ok               [fired <- :wat::core::keyword]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :wat-tests::Parker::PingRequest [])
   (:wat::core::defenum :wat-tests::Parker::PingResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(park       [self <- :wat-tests::Parker  req <- :wat-tests::Parker::ParkRequest]       -> :wat-tests::Parker::ParkResponse       :max-request-bytes 524288)
   (wake       [self <- :wat-tests::Parker  req <- :wat-tests::Parker::WakeRequest]       -> :wat-tests::Parker::WakeResponse       :max-request-bytes 524288)
   (await-tick [self <- :wat-tests::Parker  req <- :wat-tests::Parker::AwaitTickRequest]  -> :wat-tests::Parker::AwaitTickResponse  :max-request-bytes 524288)
   (ping       [self <- :wat-tests::Parker  req <- :wat-tests::Parker::PingRequest]       -> :wat-tests::Parker::PingResponse       :max-request-bytes 524288)])

(:wat::core::defn :wat-tests::parker::empty-waiters []
  -> (:wat::core::PersistentVector :- [:wat::core::i64])
  (:wat::core::PersistentVector :- [:wat::core::i64]))

(:wat::service::defservice :wat-tests::parker
  :satisfies :wat-tests::Parker
  :durable [waiters <- (:wat::core::PersistentVector :- [:wat::core::i64])]
  :ephemeral []
  :impls
  [;; store this connection as a waiter; no reply — the caller blocks in recv'.
   (park [s ctx req]
     (:wat::service::Outcome::NoReply
       (:wat-tests::parker::State
         :durable (:wat-tests::parker::Record
                    :waiters (:wat::vector::conj
                               (:wat-tests::parker::Record/waiters (:wat-tests::parker::State/durable s))
                               (:wat::service::Invocation/conn-id ctx))))))

   ;; ReplyTo every stored waiter with the waker's value, then forget them.
   ;; The waker itself is not replied to (a cast); the test fire-and-forgets the send.
   (wake [s ctx req]
     (:wat::core::let
       [value   (:wat-tests::Parker::WakeRequest/value req)
        waiters (:wat-tests::parker::Record/waiters (:wat-tests::parker::State/durable s))
        sends   (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Parker::WakeResponse])])
                                   id  <- :wat::core::i64]
                     -> (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Parker::WakeResponse])])
                    (:wat::core::conj acc
                      (:wat::service::Directed
                        :conn-id id
                        :reply (:wat-tests::Parker::WakeResponse::Ok value))))
                  (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Parker::WakeResponse])])
                  waiters)
        s'      (:wat-tests::parker::State
                  :durable (:wat-tests::parker::Record
                             :waiters (:wat::core::PersistentVector :- [:wat::core::i64])))]
       (:wat::service::Outcome::ReplyTo s' sends)))

   ;; park + arm a deadline. The generated client method blocks in recv' until -tick ReplyTo.
   (await-tick [s ctx req]
     (:wat::service::Outcome::NoReplyAndArm
       (:wat-tests::parker::State
         :durable (:wat-tests::parker::Record
                    :waiters (:wat::vector::conj
                               (:wat-tests::parker::Record/waiters (:wat-tests::parker::State/durable s))
                               (:wat::service::Invocation/conn-id ctx))))
       [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)]))

   ;; ★ STOP-2's target: an internal arm naming a client. Directed.reply is the wire Reply
   ;; the parked await-tick is waiting for (sent as-is; no invoking client to wrap for).
   (-tick [s ctx]
     (:wat::core::let
       [waiters (:wat-tests::parker::Record/waiters (:wat-tests::parker::State/durable s))
        sends   (:wat::core::foldl
                  (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Parker::Reply])])
                                   id  <- :wat::core::i64]
                     -> (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Parker::Reply])])
                    (:wat::core::conj acc
                      (:wat::service::Directed
                        :conn-id id
                        :reply (:wat-tests::Parker::Reply::AwaitTick
                                 (:wat-tests::Parker::AwaitTickResponse::Ok :fired)))))
                  (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::Parker::Reply])])
                  waiters)
        s'      (:wat-tests::parker::State
                  :durable (:wat-tests::parker::Record
                             :waiters (:wat::core::PersistentVector :- [:wat::core::i64])))]
       (:wat::service::Outcome::ReplyTo s' sends)))

   (ping [s ctx req]
     (:wat::service::Outcome::Reply s (:wat-tests::Parker::PingResponse::Ok)))])

;; ── BadTick — row 7: an internal arm returning Reply still hits the located assertion ──
(:wat::core::defsurface :wat-tests::BadTick :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::BadTick::ArmTickRequest [])
   (:wat::core::defenum :wat-tests::BadTick::ArmTickResponse :wat::enum::Pure
     :Ok               []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(arm-tick [self <- :wat-tests::BadTick  req <- :wat-tests::BadTick::ArmTickRequest] -> :wat-tests::BadTick::ArmTickResponse :max-request-bytes 524288)])

(:wat::service::defservice :wat-tests::bad-tick
  :satisfies :wat-tests::BadTick
  :durable [n <- :wat::core::i64]
  :ephemeral []
  :impls
  [(arm-tick [s ctx req]
     (:wat::service::Outcome::ReplyAndArm s (:wat-tests::BadTick::ArmTickResponse::Ok)
       [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)]))
   ;; ILLEGAL — Reply from an internal op. R is the wire Reply so the generated
   ;; ReplyTo arm (dead here) still type-checks; the assertion fires at runtime.
   (-tick [s ctx]
     (:wat::service::Outcome::Reply s
       (:wat-tests::BadTick::Reply::ArmTick
         (:wat-tests::BadTick::ArmTickResponse::Ok))))])

;; ── helpers ──────────────────────────────────────────────────────────────────
(:wat::core::defn :wat-tests::parker::connect!
  [h <- :wat-tests::parker::Handle] -> (:wat::kernel::Peer :- [:wat-tests::Parker::Op :wat-tests::Parker::Reply])
  (:wat::core::match (:wat::kernel::connect (:wat-tests::parker::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::parker::send-ok!
  [st <- :wat::kernel::SendOutcome] -> :wat::core::nil
  (:wat::core::match st
    (:wat::kernel::SendOutcome::Sent nil)
    (:wat::kernel::SendOutcome::Closed (:wat::kernel::assertion-failed! "send: closed" :wat::core::None :wat::core::None))
    (:wat::kernel::SendOutcome::Stopped (:wat::kernel::assertion-failed! "send: stopped" :wat::core::None :wat::core::None))
    ((:wat::kernel::SendOutcome::Lost _c) (:wat::kernel::assertion-failed! "send: lost" :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::parker::ping!
  [c <- (:wat::kernel::Peer :- [:wat-tests::Parker::Op :wat-tests::Parker::Reply])] -> :wat::core::nil
  (:wat::core::match (:wat-tests::Parker/ping c (:wat-tests::Parker::PingRequest))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:wat-tests::Parker::PingResponse::Ok) nil)
        ((:wat-tests::Parker::PingResponse::RequestTooLarge _b _c)
          (:wat::kernel::assertion-failed! "ping: RequestTooLarge" :wat::core::None :wat::core::None))
        ((:wat-tests::Parker::PingResponse::RequestMalformed _p _e _g)
          (:wat::kernel::assertion-failed! "ping: RequestMalformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "ping: stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "ping: closed" :wat::core::None :wat::core::None))))

;; Raw park send + same-connection ping: the ping reply proves the waiter is stored.
(:wat::core::defn :wat-tests::parker::park!
  [c <- (:wat::kernel::Peer :- [:wat-tests::Parker::Op :wat-tests::Parker::Reply])] -> :wat::core::nil
  (:wat::core::let
    [_ (:wat-tests::parker::send-ok!
         (:wat::kernel::send c (:wat-tests::Parker::Op::Park (:wat-tests::Parker::ParkRequest))))]
    (:wat-tests::parker::ping! c)))

(:wat::core::defn :wat-tests::parker::wake!
  [c <- (:wat::kernel::Peer :- [:wat-tests::Parker::Op :wat-tests::Parker::Reply])  value <- :wat::core::i64] -> :wat::core::nil
  (:wat-tests::parker::send-ok!
    (:wat::kernel::send c (:wat-tests::Parker::Op::Wake (:wat-tests::Parker::WakeRequest :value value)))))

(:wat::core::defn :wat-tests::parker::recv-wake!
  [c <- (:wat::kernel::Peer :- [:wat-tests::Parker::Op :wat-tests::Parker::Reply])] -> :wat::core::i64
  (:wat::core::match (:wat::kernel::recv c)
    ((:wat::kernel::RecvOutcome::Message recvd)
      (:wat::core::match recvd
        ((:wat-tests::Parker::Reply::Wake resp)
          (:wat::core::match resp
            ((:wat-tests::Parker::WakeResponse::Ok value) value)
            ((:wat-tests::Parker::WakeResponse::RequestTooLarge _b _c)
              (:wat::kernel::assertion-failed! "recv-wake: RequestTooLarge" :wat::core::None :wat::core::None))
            ((:wat-tests::Parker::WakeResponse::RequestMalformed _p _e _g)
              (:wat::kernel::assertion-failed! "recv-wake: RequestMalformed" :wat::core::None :wat::core::None))))
        (_ (:wat::kernel::assertion-failed! "recv-wake: expected Reply::Wake" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "recv-wake: stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "recv-wake: closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :wat-tests::parker::drive-await-tick
  [h <- :wat-tests::parker::Handle] -> :wat::core::keyword
  (:wat::core::let
    [c (:wat-tests::parker::connect! h)
     r (:wat-tests::Parker/await-tick c (:wat-tests::Parker::AwaitTickRequest))]
    (:wat::core::match r
      ((:wat::kernel::RecvOutcome::Message resp)
        (:wat::core::match resp
          ((:wat-tests::Parker::AwaitTickResponse::Ok fired) fired)
          ((:wat-tests::Parker::AwaitTickResponse::RequestTooLarge _b _c)
            (:wat::kernel::assertion-failed! "await-tick: RequestTooLarge" :wat::core::None :wat::core::None))
          ((:wat-tests::Parker::AwaitTickResponse::RequestMalformed _p _e _g)
            (:wat::kernel::assertion-failed! "await-tick: RequestMalformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "await-tick: stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "await-tick: closed" :wat::core::None :wat::core::None)))))

;; Confine a parked client to this frame so dropping it is RAII, not a sleep-guess.
;; Bindings, not tail: a Handle-param frame that tail-calls drops the Handle BEFORE the
;; call runs (eval_let_tail). park! must be a binding so `h` is still alive; `a` dies
;; when this frame returns, which is the drop we want.
(:wat::core::defn :wat-tests::parker::park-and-drop!
  [h <- :wat-tests::parker::Handle] -> :wat::core::nil
  (:wat::core::let
    [a (:wat-tests::parker::connect! h)
     _ (:wat-tests::parker::park! a)]
    nil))

(:wat::core::defn :wat-tests::bad-tick::connect!
  [h <- :wat-tests::bad-tick::Handle] -> (:wat::kernel::Peer :- [:wat-tests::BadTick::Op :wat-tests::BadTick::Reply])
  (:wat::core::match (:wat::kernel::connect (:wat-tests::bad-tick::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed c) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message c) :wat::core::None :wat::core::None))))

;; ── 1. ★ client wakes client ─────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::deferred-reply::client-wakes-client
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::parker/start :locus (:wat::spawn::thread)
           :record (:wat-tests::parker::Record :waiters (:wat-tests::parker::empty-waiters)))
       a (:wat-tests::parker::connect! h)
       b (:wat-tests::parker::connect! h)
       _ (:wat-tests::parker::park! a)
       _ (:wat-tests::parker::wake! b 7)]
      (:wat-tests::parker::recv-wake! a))
    7))

;; ── 2. ★ a TIMER wakes a client (both loci — env-grab, timerfd at process) ──
(:wat::test::deftest :wat-tests::service::deferred-reply::timer-wakes-client-on-thread
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::parker/start :locus (:wat::spawn::thread)
           :record (:wat-tests::parker::Record :waiters (:wat-tests::parker::empty-waiters)))]
      (:wat-tests::parker::drive-await-tick h))
    :fired))

(:wat::test::deftest :wat-tests::service::deferred-reply::timer-wakes-client-on-process
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::parker/start :locus (:wat::spawn::process)
           :record (:wat-tests::parker::Record :waiters (:wat-tests::parker::empty-waiters)))]
      (:wat-tests::parker::drive-await-tick h))
    :fired))

;; ── 3. one call wakes several ────────────────────────────────────────────────
(:wat::test::deftest :wat-tests::service::deferred-reply::one-call-wakes-several
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::parker/start :locus (:wat::spawn::thread)
           :record (:wat-tests::parker::Record :waiters (:wat-tests::parker::empty-waiters)))
       a (:wat-tests::parker::connect! h)
       c (:wat-tests::parker::connect! h)
       b (:wat-tests::parker::connect! h)
       _ (:wat-tests::parker::park! a)
       _ (:wat-tests::parker::park! c)
       _ (:wat-tests::parker::wake! b 11)
       va (:wat-tests::parker::recv-wake! a)
       vc (:wat-tests::parker::recv-wake! c)]
      (:wat::core::Tuple va vc))
    (:wat::core::Tuple 11 11)))

;; ── 4. a vanished waiter is survivable ───────────────────────────────────────
(:wat::test::deftest :wat-tests::service::deferred-reply::vanished-waiter-keeps-serving
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::parker/start :locus (:wat::spawn::thread)
           :record (:wat-tests::parker::Record :waiters (:wat-tests::parker::empty-waiters)))
       b (:wat-tests::parker::connect! h)
       _ (:wat-tests::parker::park-and-drop! h)
       _ (:wat-tests::parker::wake! b 1)
       _ (:wat-tests::parker::ping! b)]
      :ok)
    :ok))

;; ── 7. internal Reply is still the located assertion ─────────────────────────
;; The assertion is on the OWNER's crash channel (Handle/handle), not the client's
;; recv' — clients get a reason-free "peer crashed" by design (arc 294).
(:wat::test::deftest :wat-tests::service::deferred-reply::internal-reply-still-asserts
  (:wat::test::assert-true
    (:wat::core::let
      [h (:wat-tests::bad-tick/start :locus (:wat::spawn::thread)
           :record (:wat-tests::bad-tick::Record :n 0))
       c (:wat-tests::bad-tick::connect! h)
       _ (:wat::core::match (:wat-tests::BadTick/arm-tick c (:wat-tests::BadTick::ArmTickRequest))
           ((:wat::kernel::RecvOutcome::Message _r) nil)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "arm-tick: stopped" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "arm-tick: closed" :wat::core::None :wat::core::None)))
       msg (:wat::core::match (:wat::kernel::recv (:wat-tests::bad-tick::Handle/handle h))
             ((:wat::kernel::RecvOutcome::Message _m)
               (:wat::kernel::assertion-failed! "expected Lost from internal Reply, got Message" :wat::core::None :wat::core::None))
             ((:wat::kernel::RecvOutcome::Lost cause)
               (:wat::edn::write cause))
             (:wat::kernel::RecvOutcome::Stopped
               (:wat::kernel::assertion-failed! "expected Lost, got Stopped" :wat::core::None :wat::core::None))
             (:wat::kernel::RecvOutcome::Closed
               (:wat::kernel::assertion-failed! "expected Lost, got Closed" :wat::core::None :wat::core::None)))]
      (:wat::core::if (:wat::regex::matches? "an internal \\(-\\) op returned Outcome::Reply" msg)
        true
        (:wat::kernel::assertion-failed! msg :wat::core::None :wat::core::None)))))

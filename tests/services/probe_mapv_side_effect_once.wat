;; mapv is `(into [] (map f coll))`. empty?/first/rest each used to re-force
;; the uncached thunk, so f ran THREE times per element. map-worker uses mapv
;; to spawn runners — a thread kwargs map of N items with 1 runner incremented
;; N+2 times (3 primes of item 0, then the rest once). Process extras often
;; failed to send Work, which hid the same walk.
;;
;; EXPECT: one increment per element.

(:wat::core::defsurface :probe::Counter :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Counter::GetRequest [])
   (:wat::core::defenum :probe::Counter::GetResponse :wat::enum::Pure
     :Ok [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])
   (:wat::core::defrecord :probe::Counter::IncrementRequest [n <- :wat::core::i64])
   (:wat::core::defenum :probe::Counter::IncrementResponse :wat::enum::Pure
     :Ok [value <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64 cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(get [self <- :probe::Counter req <- :probe::Counter::GetRequest] -> :probe::Counter::GetResponse :max-request-bytes 524288)
   (increment [self <- :probe::Counter req <- :probe::Counter::IncrementRequest] -> :probe::Counter::IncrementResponse :max-request-bytes 524288)])

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

(:wat::core::defn :probe::inc
  [item <- :wat::core::i64
   & [counter <- (:wat::kernel::Peer :- [:probe::Counter::Op :probe::Counter::Reply])]]
  -> :wat::core::i64
  (:wat::core::match (:probe::Counter/increment counter (:probe::Counter::IncrementRequest :n 1))
    ((:wat::kernel::RecvOutcome::Message recvd)
      (:wat::core::match recvd
        ((:probe::Counter::IncrementResponse::Ok v) v)
        ((:probe::Counter::IncrementResponse::RequestTooLarge _b _c)
          (:wat::kernel::assertion-failed! "inc: too-large" :wat::core::None :wat::core::None))
        ((:probe::Counter::IncrementResponse::RequestMalformed _p _e _g)
          (:wat::kernel::assertion-failed! "inc: malformed" :wat::core::None :wat::core::None))))
    ((:wat::kernel::RecvOutcome::Lost cause)
      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause)
        :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped
      (:wat::kernel::assertion-failed! "inc: stopped" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed
      (:wat::kernel::assertion-failed! "inc: closed" :wat::core::None :wat::core::None))))

(:wat::core::defn :probe::read
  [h <- :probe::counter::Handle] -> :wat::core::i64
  (:wat::core::let
    [c (:wat::core::match (:wat::kernel::connect (:probe::counter::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))]
    (:wat::core::match (:probe::Counter/get c (:probe::Counter::GetRequest))
      ((:wat::kernel::RecvOutcome::Message recvd)
        (:wat::core::match recvd
          ((:probe::Counter::GetResponse::Ok v) v)
          ((:probe::Counter::GetResponse::RequestTooLarge _b _c)
            (:wat::kernel::assertion-failed! "read: too-large" :wat::core::None :wat::core::None))
          ((:probe::Counter::GetResponse::RequestMalformed _p _e _g)
            (:wat::kernel::assertion-failed! "read: malformed" :wat::core::None :wat::core::None))))
      ((:wat::kernel::RecvOutcome::Lost cause)
        (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause)
          :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Stopped
        (:wat::kernel::assertion-failed! "read: stopped" :wat::core::None :wat::core::None))
      (:wat::kernel::RecvOutcome::Closed
        (:wat::kernel::assertion-failed! "read: closed" :wat::core::None :wat::core::None)))))

(:wat::core::defn :probe::run-mapv [] -> :wat::core::i64
  (:wat::core::let
    [h (:probe::counter/start :locus (:wat::spawn::process) :record (:probe::counter::Record :count 0))
     c (:wat::core::match (:wat::kernel::connect (:probe::counter::Handle/addr h))
          ((:wat::kernel::ConnectOutcome::Connected p) p)
          ((:wat::kernel::ConnectOutcome::Refused e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Rejected e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None))
          ((:wat::kernel::ConnectOutcome::Failed e) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message e) :wat::core::None :wat::core::None)))
     _ (:wat::core::mapv
          (:wat::core::fn [i <- :wat::core::i64] -> :wat::core::i64
            (:wat::core::match (:probe::Counter/increment c (:probe::Counter::IncrementRequest :n 1))
              ((:wat::kernel::RecvOutcome::Message r) i)
              ((:wat::kernel::RecvOutcome::Lost e) i)
              (:wat::kernel::RecvOutcome::Stopped i)
              (:wat::kernel::RecvOutcome::Closed i)))
          (:wat::core::Vector :wat::core::i64 0))]
    (:probe::read h)))

(:wat::core::defn :probe::run-thread-kwargs [] -> :wat::core::i64
  (:wat::core::let
    [h (:probe::counter/start :locus (:wat::spawn::process) :record (:probe::counter::Record :count 0))
     _ (:wat::bracket::map (:wat::spawn::thread/runner-count 1)
          (:wat::core::Vector :wat::core::i64 0 1 2 3)
          :probe::inc :counter h)]
    (:probe::read h)))

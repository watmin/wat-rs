;; PROBE — does a CLIENT dying abnormally kill the SERVICE it was talking to?
;;
;; THE CLAIM UNDER TEST (a READ of wat/service.wat:2549, not yet a measurement):
;;   the serve loop's `ServiceEvent::Lost` arm calls `assertion-failed!`, which NEVER RETURNS, so
;;   the eviction and the `recur` beneath it are dead code — and the service dies with its client.
;;   Its own comment says the intent was the opposite: "surface the reason on the honest loud sink
;;   (stderr) BEFORE evicting + continuing to serve ... do not DROP it".
;;
;; WHY THIS MECHANISM. Two other routes to an abnormal client death are closed:
;;   - an OVERSIZED FRAME is documented as FrameTooLarge -> ServiceEvent::Lost (service.wat:576),
;;     but `probe-frame-cap-severs-one-conn.wat` measured the service ALIVE throughout. So either
;;     that route does not reach this arm, or it is not Lost. Not this probe's mechanism.
;;   - userland CANNOT kill a lineage: `close` is :wat::kernel::-restricted by arc 259 S2d,
;;     "the user never holds the rope". That is why no client-side injector exists at all.
;;   What IS reachable: a client that RAISES IN AN INTERNAL ARM. D1-a wrapped op-handler bodies
;;   only; the SelfOutcome branch is unwrapped, so an internal arm still dies — which is exactly
;;   what the publisher's `-run` did before `the-publisher-gives-up-in-time`.
;;
;; ⛔ Every observation PRINTS. A probe that raises on the thing it measures reports nothing.
(:wat::core::defsurface :v::Svc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :v::Svc::PingRequest [tag <- :wat::core::i64])
   (:wat::core::defenum :v::Svc::PingResponse :wat::enum::Pure
     :Ok               [tag <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :v::Svc  req <- :v::Svc::PingRequest]
     -> :v::Svc::PingResponse :max-request-bytes 4096)])

;; THE VICTIM — a plain, innocent service. It opts into nothing.
(:wat::service::defservice :v::svc
  :satisfies :v::Svc
  :durable   [served <- :wat::core::i64]
  :ephemeral []
  :impls
  [(ping [s ctx req]
     (:wat::service::Outcome::Continue s
       (:wat::core::Some (:v::Svc::Reply::Ping (:v::Svc::PingResponse::Ok (:v::Svc::PingRequest/tag req))))
       (:wat::core::Vector :- [(:wat::service::Directed :- [:v::Svc::Reply])])
       (:wat::core::Vector :- [(:wat::service::Alarm :- [:v::svc::Op])])))])

;; THE DOOMED CLIENT — a service that dials the victim at :init, then raises in an INTERNAL arm
;; while that connection is open. Its death is abnormal, and it happens holding a live peer.
(:wat::core::defsurface :c::Cli :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :c::Cli::ArmRequest [ms <- :wat::core::i64])
   (:wat::core::defenum :c::Cli::ArmResponse :wat::enum::Pure
     :Armed            []
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(arm [self <- :c::Cli  req <- :c::Cli::ArmRequest]
     -> :c::Cli::ArmResponse :max-request-bytes 4096)])

(:wat::service::defservice :c::cli
  :satisfies :c::Cli
  :durable   [victim-addr <- (:wat::kernel::Address :- [:v::Svc::Op :v::Svc::Reply])]
  :ephemeral [v <- (:wat::kernel::Peer :- [:v::Svc::Op :v::Svc::Reply])]
  :peers     [:v::Svc]
  :init (:wat::core::fn [record <- :c::cli::Record] -> :c::cli::State
          (:c::cli::State :durable record
            :v (:wat::core::match (:wat::kernel::connect (:c::cli::Record/victim-addr record))
                 ((:wat::kernel::ConnectOutcome::Connected p) p)
                 (_ (:wat::kernel::assertion-failed! "cli: dial victim failed" :wat::core::None :wat::core::None)))))
  :impls
  [(arm [s ctx req]
     ;; use the connection first, so the victim has a LIVE accepted peer for us
     (:wat::core::let
       ;; the outcome wall is right to demand this be faced; we only need the connection USED
       [_ (:wat::core::match (:v::Svc/ping (:c::cli::State/v s) (:v::Svc::PingRequest :tag 1))
            ((:wat::kernel::RecvOutcome::Message _r) nil)
            ((:wat::kernel::RecvOutcome::Lost _c) nil)
            (:wat::kernel::RecvOutcome::Stopped nil)
            (:wat::kernel::RecvOutcome::Closed nil)
            (:wat::kernel::RecvOutcome::TimedOut nil)
            ((:wat::kernel::RecvOutcome::Malformed _c) nil))]
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:c::Cli::Reply::Arm (:c::Cli::ArmResponse::Armed)))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:c::Cli::Reply])])
         [(:wat::service::Alarm :delay (:wat::time::Milliseconds (:c::Cli::ArmRequest/ms req)) :op :-boom)])))
   ;; THE ABNORMAL DEATH — an INTERNAL arm, which D1-a's seam does not wrap.
   (-boom [s ctx]
     (:wat::kernel::assertion-failed! "cli: dying on purpose, holding an open peer"
       :wat::core::None :wat::core::None))])

(:wat::core::defn :p::ping
  [c <- (:wat::kernel::Peer :- [:v::Svc::Op :v::Svc::Reply])  label <- :wat::core::String  tag <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::match (:v::Svc/ping c (:v::Svc::PingRequest :tag tag))
    ((:wat::kernel::RecvOutcome::Message resp)
      (:wat::core::match resp
        ((:v::Svc::PingResponse::Ok got) (:wat::kernel::println (:wat::string::concat label "=Ok")))
        ((:v::Svc::PingResponse::RequestTooLarge b cap) (:wat::kernel::println (:wat::string::concat label "=TooLarge")))
        ((:v::Svc::PingResponse::RequestMalformed p e g) (:wat::kernel::println (:wat::string::concat label "=Malformed")))))
    ((:wat::kernel::RecvOutcome::Lost c2) (:wat::kernel::println (:wat::string::concat label "=Lost")))
    (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println (:wat::string::concat label "=Stopped")))
    (:wat::kernel::RecvOutcome::Closed (:wat::kernel::println (:wat::string::concat label "=Closed")))
    (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::println (:wat::string::concat label "=TimedOut")))
    ((:wat::kernel::RecvOutcome::Malformed c2) (:wat::kernel::println (:wat::string::concat label "=Malformed")))))

(:wat::core::defn :p::dial
  [h <- :v::svc::Handle  label <- :wat::core::String]
  -> (:wat::core::Option :- [(:wat::kernel::Peer :- [:v::Svc::Op :v::Svc::Reply])])
  (:wat::core::match (:wat::kernel::connect (:v::svc::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) (:wat::core::Some p))
    ((:wat::kernel::ConnectOutcome::Refused f)
      (:wat::core::let [_ (:wat::kernel::println (:wat::string::concat label "=connect-REFUSED"))] :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected f)
      (:wat::core::let [_ (:wat::kernel::println (:wat::string::concat label "=connect-REJECTED"))] :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed f)
      (:wat::core::let [_ (:wat::kernel::println (:wat::string::concat label "=connect-FAILED"))] :wat::core::None))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [vh (:v::svc/start :locus (:wat::spawn::process) :record (:v::svc::Record :served 0))
     ;; NON-VACUITY: the victim answers before anything dies
     a (:p::dial vh "a-before ")
     _ (:wat::core::match a ((:wat::core::Some p) (:p::ping p "a-before " 7)) (:wat::core::None nil))
     ;; the doomed client dials the victim, pings it, then arms its own death 50 ms out
     ch (:c::cli/start :locus (:wat::spawn::process/post-spawn
                                (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                                  (:wat::service::require-granted (:v::svc/grant vh (:wat::core::Vector :- [:wat::core::i64] (:wat::spawn::ProcessLaunch/pid pl))))))
          :record (:c::cli::Record :victim-addr (:v::svc::Handle/addr vh)))
     cp (:wat::core::match (:wat::kernel::connect (:c::cli::Handle/addr ch))
          ((:wat::kernel::ConnectOutcome::Connected p) (:wat::core::Some p))
          (_ (:wat::core::let [_ (:wat::kernel::println "cli-dial=FAILED")] :wat::core::None)))
     _ (:wat::core::match cp
         ((:wat::core::Some p)
           (:wat::core::match (:c::Cli/arm p (:c::Cli::ArmRequest :ms 50))
             ((:wat::kernel::RecvOutcome::Message _r) (:wat::kernel::println "cli-armed=yes"))
             (_ (:wat::kernel::println "cli-armed=NO"))))
         (:wat::core::None nil))
     ;; let the client die
     _ (:wat::core::match (:wat::kernel::recv
                            (:wat::kernel::after :wat::program::PeerKind::thread
                              (:wat::time::Milliseconds 1500) :done))
         (_ nil))
     ;; ⭑ THE MEASUREMENT — the victim never did anything wrong. Is it still serving?
     _ (:wat::core::match a ((:wat::core::Some p) (:p::ping p "a-after  " 8)) (:wat::core::None nil))
     b (:p::dial vh "b-fresh  ")
     _ (:wat::core::match b ((:wat::core::Some p) (:p::ping p "b-fresh  " 9)) (:wat::core::None nil))]
    nil))

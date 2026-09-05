;; wat-tests/service-signal-observer.wat — arc 278 "a service that measures itself" (A1+A2).
;;
;; THE FIRST SIGNAL CONSUMER. Grounded census (DESIGN-STONE-a-service-that-measures-itself.md):
;; no wat service, anywhere in the substrate, has ever observed a signal — `stopped?` appears
;; nowhere but a comment in wat/, and sigusr1?/sigusr2?/sighup? appear zero times. Handlers set
;; the flags; predicates read them; the actor layer has never asked. This defservice measures its
;; OWN process's signal flags AS IT SERVES: a durable field per observation, updated inside the
;; `observe` op body, read back through ordinary client ops as the sequence progresses. There is
;; no admin ask — querying as-you-go proves observation DURING real operation, which a final
;; tally delivered via `stop` cannot (and `stop` needs no state path at all: a killed service
;; delivers nothing, which is ruled correct — service.wat:1227 `ServiceEvent::Shutdown -> nil`,
;; untouched here).
;;
;; A1 — un-erasing the concrete locus: `start` builds a Handle whose `handle` field is typed
;; `(:wat::kernel::Peer :- [Admin Status])` (wat/spawn.wat:265 `Launched`'s deliberate erasure — it is
;; what keeps `stop` locus-agnostic). `:wat::kernel::signal` needs the concrete `(Process :- [I O])`.
;; The accessor is `:wat::kernel::peer-process` — mirrors the already-shipped `peer-pid` (arc 170
;; capability circuit stone 2), which reads the SAME RustOpaque type-path tag for a DIFFERENT
;; projection: `Some` on a process locus (the very same peer value, now nameable `(Process :- [I O])`),
;; `None` on a thread (a thread has no process to signal). Pure projection: no effect, no signal.
;;
;; A2 — the app: the builder's sequence, verbatim —
;;   (signal proc :sighup)    -> Delivered   (query client :sighup) -> true
;;   (signal proc :user1)     -> Delivered   (query client :user1)  -> true
;;   (signal proc :user2)     -> Delivered   (query client :user2)  -> true
;;   (signal proc :terminate) -> Delivered, the child dies, no notification
;;
;; The delivery asymmetry (expected, not a bug — `substrate_on_stop_signal` sets the flag AND
;; writes the wake pipe; `substrate_on_sigusr1`/`sigusr2`/`sighup` only set the flag): sighup/
;; user1/user2 are a bitflip observed on the NEXT op — a blocked service does not wake for them,
;; so `observe` is called once per signal to drive that op. `terminate` wakes the blocked `poll`
;; directly.
;;
;; Read alongside: wat-tests/service-admin-facet.wat (the start/connect/op exemplar this copies),
;; wat-tests/process/signal-user2-and-hangup-independent.wat (the discrimination-in-one-reply
;; discipline), wat/service.wat:1215-1230 (the poll'/ServiceEvent dispatch), wat/spawn.wat:258-266
;; (Launched, the erasure A1 undoes).

;; ── the surface: one query op, `observe`, returning the durable counters ─────────────────────
(:wat::core::defsurface :wat-tests::SignalObserver :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :wat-tests::SignalObserver::ObserveRequest [])
   (:wat::core::defenum :wat-tests::SignalObserver::ObserveResponse :wat::enum::Pure
     :Ok               [requests <- :wat::core::i64  sighup <- :wat::core::bool  user1 <- :wat::core::bool  user2 <- :wat::core::bool]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(observe [self <- :wat-tests::SignalObserver  req <- :wat-tests::SignalObserver::ObserveRequest] -> :wat-tests::SignalObserver::ObserveResponse :max-request-bytes 524288)])

;; ── the service: measures as it serves. Every `observe` call re-reads the process-global flags
;; (a bitflip set by handlers wat's control never touches — ruling 4 / STOP-1: never patched,
;; never reset here) and folds the current reading into the durable record it replies with. ─────
(:wat::service::defservice :wat-tests::signal-observer
  :satisfies :wat-tests::SignalObserver
  :durable [requests <- :wat::core::i64  sighup <- :wat::core::bool  user1 <- :wat::core::bool  user2 <- :wat::core::bool]
  :ephemeral []
  :impls
  [(observe [s ctx req]
     (:wat::core::let
       [d          (:wat-tests::signal-observer::State/durable s)
        new-reqs   (:wat::i64::+ (:wat-tests::signal-observer::Record/requests d) 1)
        new-sighup (:wat::kernel::sighup?)
        new-user1  (:wat::kernel::sigusr1?)
        new-user2  (:wat::kernel::sigusr2?)
        rec        (:wat-tests::signal-observer::Record :requests new-reqs :sighup new-sighup :user1 new-user1 :user2 new-user2)]
       (:wat::service::Outcome::Continue
         (:wat-tests::signal-observer::State :durable rec)
         (:wat::core::Some (:wat-tests::SignalObserver::Reply::Observe (:wat-tests::SignalObserver::ObserveResponse::Ok new-reqs new-sighup new-user1 new-user2))) (:wat::core::Vector :- [(:wat::service::Directed :- [:wat-tests::SignalObserver::Reply])]) (:wat::core::Vector :- [(:wat::service::Alarm :- [:wat-tests::signal-observer::Op])]))))])

;; ── a helper: drive one `observe` round trip, facing every RecvOutcome arm. ──────────────────
(:wat::core::defn :wat-tests::signal-observer::observe! [c <- :wat-tests::SignalObserver] -> :wat-tests::SignalObserver::ObserveResponse
  (:wat::core::match (:wat-tests::SignalObserver/observe c (:wat-tests::SignalObserver::ObserveRequest))
    ((:wat::kernel::RecvOutcome::Message m) m)
    ((:wat::kernel::RecvOutcome::Lost cause) (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::assertion-failed! "observe!: stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
    (:wat::kernel::RecvOutcome::Closed (:wat::kernel::assertion-failed! "observe!: peer closed" :wat::core::None :wat::core::None)) (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::assertion-failed! "recv: timed out — the peer is alive and silent" :wat::core::None :wat::core::None))))

;; ── the sequence: one process, all four handlers, three user signals discriminated ──────────
;; Every step is faced individually (RequestTooLarge/RequestMalformed each get their own
;; assertion-failed! naming which op saw the wire-breach) and folded into one (Vector bool),
;; compared structurally against the expected all-true vector — never a loose contains?.
(:wat::test::deftest :wat-tests::service::signal-observer-measures-itself

  (:wat::test::assert-eq
    (:wat::core::let
      [h    (:wat-tests::signal-observer/start :locus (:wat::spawn::process)
              :record (:wat-tests::signal-observer::Record :requests 0 :sighup false :user1 false :user2 false))
       c    (:wat::core::match (:wat::kernel::connect (:wat-tests::signal-observer::Handle/addr h))
              ((:wat::kernel::ConnectOutcome::Connected p) p)
              ((:wat::kernel::ConnectOutcome::Refused cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
              ((:wat::kernel::ConnectOutcome::Rejected cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None))
              ((:wat::kernel::ConnectOutcome::Failed cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None)))
       proc (:wat::core::match (:wat::kernel::peer-process (:wat-tests::signal-observer::Handle/handle h))
              ((:wat::core::Some p) p)
              (:wat::core::None (:wat::kernel::assertion-failed! "signal-observer-measures-itself: expected a process locus" :wat::core::None :wat::core::None)))

       ;; ── sighup: a bitflip only, no wake — drive `observe` to see it. ─────────────────────
       sighup-delivered
       (:wat::core::match (:wat::kernel::signal proc :wat::kernel::Signal::Hangup)
         (:wat::kernel::SignalOutcome::Delivered true)
         ((:wat::kernel::SignalOutcome::Failed cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None)))
       after-hangup
       (:wat::core::match (:wat-tests::signal-observer::observe! c)
         ((:wat-tests::SignalObserver::ObserveResponse::Ok reqs hup u1 u2)
           (:wat::core::Vector :- [:wat::core::bool]
             (:wat::core::= reqs 1) hup (:wat::core::not u1) (:wat::core::not u2)))
         ((:wat-tests::SignalObserver::ObserveResponse::RequestTooLarge bytes cap)
           (:wat::kernel::assertion-failed! "unexpected RequestTooLarge after sighup" :wat::core::None :wat::core::None))
         ((:wat-tests::SignalObserver::ObserveResponse::RequestMalformed mpath mexpected mgot)
           (:wat::kernel::assertion-failed! "unexpected RequestMalformed after sighup" :wat::core::None :wat::core::None)))

       ;; ── user1 — independent of sighup, which must stay observed true. ───────────────────
       user1-delivered
       (:wat::core::match (:wat::kernel::signal proc :wat::kernel::Signal::User1)
         (:wat::kernel::SignalOutcome::Delivered true)
         ((:wat::kernel::SignalOutcome::Failed cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None)))
       after-user1
       (:wat::core::match (:wat-tests::signal-observer::observe! c)
         ((:wat-tests::SignalObserver::ObserveResponse::Ok reqs hup u1 u2)
           (:wat::core::Vector :- [:wat::core::bool]
             (:wat::core::= reqs 2) hup u1 (:wat::core::not u2)))
         ((:wat-tests::SignalObserver::ObserveResponse::RequestTooLarge bytes cap)
           (:wat::kernel::assertion-failed! "unexpected RequestTooLarge after user1" :wat::core::None :wat::core::None))
         ((:wat-tests::SignalObserver::ObserveResponse::RequestMalformed mpath mexpected mgot)
           (:wat::kernel::assertion-failed! "unexpected RequestMalformed after user1" :wat::core::None :wat::core::None)))

       ;; ── user2 — independent of both sighup AND user1, which must stay observed true. ─────
       user2-delivered
       (:wat::core::match (:wat::kernel::signal proc :wat::kernel::Signal::User2)
         (:wat::kernel::SignalOutcome::Delivered true)
         ((:wat::kernel::SignalOutcome::Failed cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None)))
       after-user2
       (:wat::core::match (:wat-tests::signal-observer::observe! c)
         ((:wat-tests::SignalObserver::ObserveResponse::Ok reqs hup u1 u2)
           (:wat::core::Vector :- [:wat::core::bool]
             (:wat::core::= reqs 3) hup u1 u2))
         ((:wat-tests::SignalObserver::ObserveResponse::RequestTooLarge bytes cap)
           (:wat::kernel::assertion-failed! "unexpected RequestTooLarge after user2" :wat::core::None :wat::core::None))
         ((:wat-tests::SignalObserver::ObserveResponse::RequestMalformed mpath mexpected mgot)
           (:wat::kernel::assertion-failed! "unexpected RequestMalformed after user2" :wat::core::None :wat::core::None)))

       ;; ── terminate: the child dies, no notification — there is no admin ask (ruled). ─────
       terminate-delivered
       (:wat::core::match (:wat::kernel::signal proc :wat::kernel::Signal::Terminate)
         (:wat::kernel::SignalOutcome::Delivered true)
         ((:wat::kernel::SignalOutcome::Failed cause) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cause) :wat::core::None :wat::core::None)))]

      (:wat::core::concat
        (:wat::core::Vector :- [:wat::core::bool] sighup-delivered user1-delivered user2-delivered terminate-delivered)
        (:wat::core::concat after-hangup (:wat::core::concat after-user1 after-user2))))
    (:wat::core::Vector :- [:wat::core::bool]
      true true true true     ;; the four SignalOutcome::Delivered (sighup/user1/user2/terminate)
      true true true true     ;; after sighup:  requests=1, sighup,  !user1, !user2
      true true true true     ;; after user1:   requests=2, sighup,  user1,  !user2
      true true true true)))  ;; after user2:   requests=3, sighup,  user1,  user2

;; ── A1's other branch: a thread locus has no process to signal. ──────────────────────────────
(:wat::test::deftest :wat-tests::service::signal-observer-thread-has-no-process
  (:wat::test::assert-eq
    (:wat::core::let
      [h (:wat-tests::signal-observer/start :locus (:wat::spawn::thread)
           :record (:wat-tests::signal-observer::Record :requests 0 :sighup false :user1 false :user2 false))]
      (:wat::core::match (:wat::kernel::peer-process (:wat-tests::signal-observer::Handle/handle h))
        ((:wat::core::Some _p) false)
        (:wat::core::None true)))
    true))

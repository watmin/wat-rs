;; probe-self-sched-bisect.wat — BISECT for arc 278 item (c) (self-scheduling defservices).
;;
;; `tests/services/probe_arc278_self_scheduling.rs` is RED (both loci, #[ignore]'d): the client's
;; `poll` raises `recv': peer closed`. That message says the SERVICE's end of the client connection
;; went away — it does NOT say when, or why. This probe replaces every raise with a SENTINEL, so a
;; single run reports WHICH interleaving kills the client:
;;
;;   staged     — poll, arm, poll, wait out the ticks IDLE, poll .... client never talks during a tick
;;   tight-loop — arm, then poll every 5ms while ticks fire every 5ms ... the fixture's own shape
;;   no-arm     — the identical tight loop with NO timer ever armed ..... is the tight loop innocent?
;;   once       — target=1: ONE fire, taking NoReply (no re-arm fold) ... fire alone, or the re-arm?
;;
;; Sentinels: -10 connection Closed · -11 Lost · -12 Stopped · -20 attempts exhausted (client alive).

(:wat::core::defsurface :sched::Ticker :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :sched::Ticker::StartRequest [])
   (:wat::core::defenum :sched::Ticker::StartResponse :wat::enum::Pure
     :Ok              []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])
   (:wat::core::defrecord :sched::Ticker::PollRequest [])
   (:wat::core::defenum :sched::Ticker::PollResponse :wat::enum::Pure
     :Count           [n <- :wat::core::i64]
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])  expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(start [self <- :sched::Ticker  req <- :sched::Ticker::StartRequest] -> :sched::Ticker::StartResponse
     :max-request-bytes 524288)
   (poll  [self <- :sched::Ticker  req <- :sched::Ticker::PollRequest]  -> :sched::Ticker::PollResponse
     :max-request-bytes 524288)])

(:wat::service::defservice :sched::ticker
  :satisfies :sched::Ticker
  :durable   [count <- :wat::core::i64  target <- :wat::core::i64]
  :ephemeral []
  :init (:wat::core::fn [record <- :sched::ticker::Record] -> :sched::ticker::State
          (:sched::ticker::State :durable record))
  :impls
  [(start [s ctx req]
     (:wat::service::Outcome::ReplyAndArm s (:sched::Ticker::StartResponse::Ok)
       [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)]))

   (poll [s ctx req]
     (:wat::service::Outcome::Reply s
       (:sched::Ticker::PollResponse::Count
         (:sched::ticker::Record/count (:sched::ticker::State/durable s)))))

   (-tick [s ctx]
     (:wat::core::let
       [rec  (:sched::ticker::State/durable s)
        n    (:wat::i64::+ (:sched::ticker::Record/count rec) 1)
        rec' (:sched::ticker::Record :count n :target (:sched::ticker::Record/target rec))
        s'   (:sched::ticker::State :durable rec')]
       (:wat::core::if (:wat::i64::< n (:sched::ticker::Record/target rec))
         (:wat::service::Outcome::NoReplyAndArm s'
           [(:wat::service::Alarm :after (:wat::time::Millisecond 5) :op :-tick)])
         (:wat::service::Outcome::NoReply s'))))])

;; mora-honest wait — a one-shot `after` selected on, never a sleep.
(:wat::core::defn :sched::nap [ms <- :wat::core::i64] -> :wat::core::nil
  (:wat::core::match
    (:wat::kernel::select
      (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :wat::core::keyword])]
        (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond ms) :done)))
    ((:wat::spawn::ServiceEvent::Message _i _m) nil)
    ((:wat::spawn::ServiceEvent::Closed _i) nil)
    ((:wat::spawn::ServiceEvent::Lost _i _c) nil)
    ((:wat::spawn::ServiceEvent::Malformed _i _c) nil)
    ((:wat::spawn::ServiceEvent::Rejected _i _c) nil)
    (:wat::spawn::ServiceEvent::Shutdown nil)
    ((:wat::spawn::ServiceEvent::Connection _p) nil)
    ((:wat::spawn::ServiceEvent::Admin _m) nil)))

(:wat::core::defn :sched::safe-poll
  [c <- (:wat::kernel::Peer :- [:sched::Ticker::Op :sched::Ticker::Reply])] -> :wat::core::i64
  (:wat::core::match (:sched::Ticker/poll c (:sched::Ticker::PollRequest))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:sched::Ticker::PollResponse::Count n) n)
        ((:sched::Ticker::PollResponse::RequestTooLarge _b _cp) -1)
        ((:sched::Ticker::PollResponse::RequestMalformed _p _e _g) -2)))
    ((:wat::kernel::RecvOutcome::Lost __cause) -11)
    (:wat::kernel::RecvOutcome::Stopped -12)
    (:wat::kernel::RecvOutcome::Closed -10)))

(:wat::core::defn :sched::safe-start
  [c <- (:wat::kernel::Peer :- [:sched::Ticker::Op :sched::Ticker::Reply])] -> :wat::core::i64
  (:wat::core::match (:sched::Ticker/start c (:sched::Ticker::StartRequest))
    ((:wat::kernel::RecvOutcome::Message __recv)
      (:wat::core::match __recv
        ((:sched::Ticker::StartResponse::Ok) 0)
        ((:sched::Ticker::StartResponse::RequestTooLarge _b _cp) -1)
        ((:sched::Ticker::StartResponse::RequestMalformed _p _e _g) -2)))
    ((:wat::kernel::RecvOutcome::Lost __cause) -11)
    (:wat::kernel::RecvOutcome::Stopped -12)
    (:wat::kernel::RecvOutcome::Closed -10)))

;; the FIXTURE's drive shape: poll in a tight loop with a 5ms backoff — client traffic arriving
;; DURING the tick cadence. A negative return is the transport sentinel, propagated immediately.
(:wat::core::defn :sched::poll-until
  [c <- (:wat::kernel::Peer :- [:sched::Ticker::Op :sched::Ticker::Reply])
   target <- :wat::core::i64  attempts <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::i64::<= attempts 0)
    -20
    (:wat::core::let [n (:sched::safe-poll c)]
      (:wat::core::if (:wat::i64::< n 0)
        n
        (:wat::core::if (:wat::i64::>= n target)
          n
          (:wat::core::let [_ (:sched::nap 5)]
            (:sched::poll-until c target (:wat::i64::- attempts 1))))))))

;; connect to an ALREADY-STARTED ticker — the handle stays with the caller (see the warning below).
(:wat::core::defn :sched::conn
  [h <- :sched::ticker::Handle] -> (:wat::kernel::Peer :- [:sched::Ticker::Op :sched::Ticker::Reply])
  (:wat::core::match (:wat::kernel::connect (:sched::ticker::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p) p)
    ((:wat::kernel::ConnectOutcome::Refused cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Rejected cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))
    ((:wat::kernel::ConnectOutcome::Failed cc) (:wat::kernel::assertion-failed! (:wat::kernel::Failure/message cc) :wat::core::None :wat::core::None))))

;; ⚠ THE HANDLE OWNS THE SERVICE'S LIFETIME. An earlier revision of this probe factored the
;; start+connect into a `dial` helper that returned only the client peer — the `Handle` went out of
;; scope on return, the service died with it, and ALL FOUR drivers reported -10 (Closed), including
;; the no-timer control. Every driver below therefore binds `h` in its OWN let and holds it for the
;; whole drive. A -10 here is the substrate; a -10 there was the probe.

;; staged: the client is IDLE while the ticks fire.
(:wat::core::defn :sched::drive-staged [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::thread)
          :record (:sched::ticker::Record :count 0 :target 3))
     c  (:sched::conn h)
     _0 (:sched::safe-poll c)
     s1 (:sched::safe-start c)
     _2 (:sched::safe-poll c)
     _w (:sched::nap 60)
     n  (:sched::safe-poll c)]
    (:wat::core::if (:wat::i64::< s1 0) s1 n)))

;; tight: the fixture's exact drive — arm, then poll DURING the tick cadence.
(:wat::core::defn :sched::drive-tight [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::thread)
          :record (:sched::ticker::Record :count 0 :target 3))
     c  (:sched::conn h)
     s1 (:sched::safe-start c)
     n  (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40))]
    n))

;; CONTROL no-arm: the identical tight loop, but `start` is NEVER called, so no timer ever enters
;; the selectable set. `-20` (attempts exhausted, client alive) means the tight loop alone is innocent.
(:wat::core::defn :sched::drive-noarm [] -> :wat::core::i64
  (:wat::core::let
    [h (:sched::ticker/start :locus (:wat::spawn::thread)
         :record (:sched::ticker::Record :count 0 :target 3))
     c (:sched::conn h)
     n (:sched::poll-until c 3 40)]
    n))

;; CONTROL once: target=1 — ONE fire, taking the `NoReply` path (no re-arm fold). If this dies too,
;; the fault is the single fire + remove-at, not the re-arm.
(:wat::core::defn :sched::drive-once [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::thread)
          :record (:sched::ticker::Record :count 0 :target 1))
     c  (:sched::conn h)
     s1 (:sched::safe-start c)
     n  (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 1 40))]
    n))

;; poll-until call lives: inside the let's BINDING list (h still textually ahead of it) versus in
;; the let's BODY (h's last use is `Handle/addr`, in the first binding). If the body form dies while
;; the binding form lives, a Handle is being released at its last textual USE rather than at the end
;; of its lexical scope — and that, not the timer, is what kills `probe_arc278_self_scheduling`.
;; That fixture's `drive-ticker` takes `h` as a PARAMETER, uses it once for `Handle/addr` in the
;; first binding, and then does all its polling in the body — precisely this shape.

;; A — poll-until in a BINDING (h textually precedes the whole drive)
(:wat::core::defn :sched::hold-in-binding [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::thread)
          :record (:sched::ticker::Record :count 0 :target 3))
     c  (:sched::conn h)
     s1 (:sched::safe-start c)
     n  (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40))]
    n))

;; B — poll-until in the BODY (h's last use is the `conn` binding)
;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: this function is the
;; tail-escape the wall must name; the probe has to construct it to print the
;; discrimination table. Rune the instrument, never the acceptance criterion.
(:wat::core::defn :sched::hold-in-body [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::thread)
          :record (:sched::ticker::Record :count 0 :target 3))
     c  (:sched::conn h)
     s1 (:sched::safe-start c)]
    (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40))))

;; C — the fixture's ACTUAL shape: the handle arrives as a PARAMETER and the drive is in the body.
;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: this is road 3 (Handle param,
;; tail-escapes a peer). The probe must construct it to print C-param-tail. Rune the
;; instrument, never the acceptance criterion.
(:wat::core::defn :sched::drive-param [h <- :sched::ticker::Handle] -> :wat::core::i64
  (:wat::core::let
    [c  (:sched::conn h)
     s1 (:sched::safe-start c)]
    (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40))))

(:wat::core::defn :sched::hold-as-param [] -> :wat::core::i64
  (:sched::drive-param
    (:sched::ticker/start :locus (:wat::spawn::thread)
      :record (:sched::ticker::Record :count 0 :target 3))))

;; ── D / E — is the mechanism TAIL POSITION? ───────────────────────────────────────────────────
;; B differs from A in one more way than "binding vs body": B's drive call is in TAIL position, so
;; the let frame is a candidate to be POPPED (and its `h` released) before the call runs. A's drive
;; sits in a binding, which is never a tail call. These two separate the axes.

;; D — body, but NOT a tail call: the drive is an ARGUMENT, so the frame must outlive it.
;; If D is green while B is closed, tail position — not "body" — is the discriminator.
(:wat::core::defn :sched::hold-in-body-nontail [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::thread)
          :record (:sched::ticker::Record :count 0 :target 3))
     c  (:sched::conn h)
     s1 (:sched::safe-start c)]
    (:wat::i64::+ (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40)) 0)))

;; E — body, tail call, but `h` is TOUCHED after the drive would have started. If a live textual
;; reference in the body keeps the service alive, this is the workaround shape a caller must know.
(:wat::core::defn :sched::hold-in-body-touched [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::thread)
          :record (:sched::ticker::Record :count 0 :target 3))
     c  (:sched::conn h)
     s1 (:sched::safe-start c)
     n  (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40))
     _k (:sched::ticker::Handle/addr h)]
    n))


;; ── F — IS THE TIMER INVOLVED AT ALL? ─────────────────────────────────────────────────────────
;; `start` is NEVER called here, so no `Alarm` is ever armed and no timer ever enters the service's
;; selectable set. The ONLY thing this shares with B is: a Handle bound in a let, and a call in the
;; let's TAIL position. If F closes too, self-scheduling is a bystander and the defect is general to
;; every service handle in the language.
;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: F is a deliberate tail
;; escape (Handle bound in a let, user-fn call in the let's tail). The probe must
;; construct it to print the discrimination table.
(:wat::core::defn :sched::plain-service-tail [] -> :wat::core::i64
  (:wat::core::let
    [h (:sched::ticker/start :locus (:wat::spawn::thread)
         :record (:sched::ticker::Record :count 0 :target 3))
     c (:sched::conn h)]
    (:sched::poll-until c 3 6)))

;; G — F's twin, drive moved into a binding. Expect -20 (attempts exhausted, client ALIVE).
(:wat::core::defn :sched::plain-service-nontail [] -> :wat::core::i64
  (:wat::core::let
    [h (:sched::ticker/start :locus (:wat::spawn::thread)
         :record (:sched::ticker::Record :count 0 :target 3))
     c (:sched::conn h)
     n (:sched::poll-until c 3 6)]
    n))


;; ── H — DOES THE CLIENT GET THE REASON? ───────────────────────────────────────────────────────
;; The sentinel is only worth minting if the cause travels. This returns the Lost cause's own
;; message text, so the probe prints what a caller would actually see.
(:wat::core::defn :sched::poll-reason
  [c <- (:wat::kernel::Peer :- [:sched::Ticker::Op :sched::Ticker::Reply])] -> :wat::core::String
  (:wat::core::match (:sched::Ticker/poll c (:sched::Ticker::PollRequest))
    ((:wat::kernel::RecvOutcome::Message __recv) "(a reply arrived — not severed)")
    ((:wat::kernel::RecvOutcome::Lost __cause) (:wat::kernel::LociDiedError/message __cause))
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::Closed "Closed (MUTE — the reason was dropped)")))

;; rune:check(handle-lifetime-creation-escape) — INSTRUMENT: same tail-escape shape as
;; plain-service-tail; this function prints the Lost cause a caller would see.
(:wat::core::defn :sched::severed-reason [] -> :wat::core::String
  (:wat::core::let
    [h (:sched::ticker/start :locus (:wat::spawn::thread)
         :record (:sched::ticker::Record :count 0 :target 3))
     c (:sched::conn h)]
    (:sched::poll-reason c)))


;; ── P — THE PROCESS LOCUS, held handle. ───────────────────────────────────────────────────────
;; The fixture asserts BOTH loci. Everything above is thread-tier, so the process tick (env-grab
;; arms the timer at the service's own tier) is still unproven. Same held-handle shape as
;; `drive-tight`; only `:locus` differs.
(:wat::core::defn :sched::drive-tight-process [] -> :wat::core::i64
  (:wat::core::let
    [h  (:sched::ticker/start :locus (:wat::spawn::process)
          :record (:sched::ticker::Record :count 0 :target 3))
     c  (:sched::conn h)
     s1 (:sched::safe-start c)
     n  (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40))]
    n))


;; ── Q — the FIXTURE'S REPAIR SHAPE, tested before it is proposed. ─────────────────────────────
;; The fixture's `drive-ticker` takes the handle as a PARAMETER and drives in the body (= C, which
;; dies). The minimal repair is to move the drive into a BINDING and return it. Whether a PARAM
;; survives that is a separate question from whether a let-BOUND value does (A), and it is not
;; something to assume: test it.
(:wat::core::defn :sched::drive-param-binding [h <- :sched::ticker::Handle] -> :wat::core::i64
  (:wat::core::let
    [c  (:sched::conn h)
     s1 (:sched::safe-start c)
     n  (:wat::core::if (:wat::i64::< s1 0) s1 (:sched::poll-until c 3 40))]
    n))

(:wat::core::defn :sched::param-binding-thread [] -> :wat::core::i64
  (:sched::drive-param-binding
    (:sched::ticker/start :locus (:wat::spawn::thread)
      :record (:sched::ticker::Record :count 0 :target 3))))

(:wat::core::defn :sched::param-binding-process [] -> :wat::core::i64
  (:sched::drive-param-binding
    (:sched::ticker/start :locus (:wat::spawn::process)
      :record (:sched::ticker::Record :count 0 :target 3))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::do
    (:wat::kernel::println
      (:wat::string::interpolate
        "A-binding={a} B-body-tail={b} C-param-tail={c} D-body-nontail={d} E-body-touched={e} || F-noTimer-tail={f} G-noTimer-nontail={g}"
        :a (:sched::hold-in-binding) :b (:sched::hold-in-body) :c (:sched::hold-as-param)
        :d (:sched::hold-in-body-nontail) :e (:sched::hold-in-body-touched)
        :f (:sched::plain-service-tail) :g (:sched::plain-service-nontail)))
    (:wat::kernel::println (:sched::severed-reason))
    (:wat::kernel::println
      (:wat::string::interpolate
        "held: THREAD-tick={t} PROCESS-tick={p} || REPAIR-SHAPE param+binding: thread={q} process={r}"
        :t (:sched::drive-tight) :p (:sched::drive-tight-process)
        :q (:sched::param-binding-thread) :r (:sched::param-binding-process)))))

;; probe-m1-fix-revoke.wat — the FIXED revoked circuit: probe-m1-fix-norevoke.wat with the
;; `echo'/revoke` line restored.
;;
;; The prober sends dial #2's reply UP, so a regression (dial #2 admitted after the revoke) would be
;; OBSERVABLE as a print. With the revoke, dial #2 is bounced: the prober's echo recv' EOFs and it
;; dies, so compute raises — the discriminating twin of probe-m1-fix-norevoke.wat.
;;
;; EXPECT: a raise, `recv': peer closed` — and no "NOREVOKE-REACHED-END" print.
;;
;; DISPOSITION (255.75) — negative, by design: the raise IS the proof (revoke is load-bearing —
;; its discriminating twin, `probe-m1-fix-norevoke.wat`, stays a plain `.wat` and prints
;; "NOREVOKE-REACHED-END: echo:hi" clean, per `DESIGN-STONE-M1-TEETH-revoke-refusal.md`'s "the
;; vacuity trap" section). a plain `.wat` (AMEND: this file starts up clean — `startup_from_file` succeeds, the
;; failure is runtime-only — and `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids
;; `.wat.bad` for a startup-clean file; per that gate's remedy #1, moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` instead, so the "every probe runs, exit
;; 0" gate does not require it), driven by
;; `tests/process/probe_arc255_75_negative_probes.rs`, asserting the raised message is either
;; `recv': peer closed` (the documented bounce path) OR the kernel's own `LociDiedError::Disconnected`
;; ("disconnected" — measured once, full-floor concurrent load only, never in isolation: the
;; unmatched `echo'/revoke` call below raises directly if the echo service's own control channel
;; disconnects first under heavy process-spawn pressure, before the documented path is reached —
;; both are non-regression failures, see that test's own AMEND comment), and that
;; "NOREVOKE-REACHED-END" never prints either way.

(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- wat.type/String])
   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- wat.type/String] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                                      :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])

(:wat::service::defservice :probe::echo
  :satisfies :probe::Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req]
            (:wat::service::Outcome.Reply {:state s
              :reply (:probe::Echo::EchoResponse.Ok {:reply (:wat::string::concat "echo:" (:probe::Echo::EchoRequest/msg req))})}))])

(:wat::core::defn :user::main [] -> wat.type/nil
  (:wat::core::let
    [eh  (:probe::echo/start :locus (:wat::spawn::process) :record (:probe::echo::Record))
     ea  (:probe::echo::Handle/addr eh)
     prober (:wat::test::spawn-peer (:wat::spawn::process)
              (:wat::core::forms
                (:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer
                  :messages
                  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- wat.type/String])
                   (:wat::core::defenum :probe::Echo::EchoResponse :wat::enum::Pure :Ok [reply <- wat.type/String] :RequestTooLarge [bytes <- wat.type/i64  cap <- wat.type/i64]
                                                                                                                      :RequestMalformed [path <- (wat.type/Vector :- [wat.type/String])  expected <- wat.type/String  got <- wat.type/String])]
                  :features
                  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse :max-request-bytes 524288)])
                (:wat::core::defn :user::main [] -> wat.type/nil
                  (:wat::core::let
                    [self (:wat::program::self-peer :wat::core::String
                             (:wat::kernel::Address :- [:probe::Echo::Op :probe::Echo::Reply]))
                     addr (:wat::core::match (:wat::kernel::recv self) [:wat::kernel::RecvOutcome.Message {:msg __d} __d] [:wat::kernel::RecvOutcome.Lost {:cause __c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __c))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
                     c1   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome.Connected {:peer p} p] [:wat::kernel::ConnectOutcome.Closed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Undialable {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
                     er1  (:probe::Echo/echo c1 (:probe::Echo::EchoRequest :msg "hi"))
                     _    (:wat::core::match (:wat::kernel::send self (:wat::core::match er1 [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv [:probe::Echo::EchoResponse.Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil] [:wat::kernel::SendOutcome.Stopped {} nil])
                     _sig (:wat::core::match (:wat::kernel::recv self) [:wat::kernel::RecvOutcome.Message {:msg __d} __d] [:wat::kernel::RecvOutcome.Lost {:cause __c} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __c))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")])
                     c2   (:wat::core::match (:wat::kernel::connect addr) [:wat::kernel::ConnectOutcome.Connected {:peer p} p] [:wat::kernel::ConnectOutcome.Closed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Undialable {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.WrongPeer {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))] [:wat::kernel::ConnectOutcome.Failed {:cause c} (:wat::kernel::assertion-failed! :message (:wat::kernel::Failure/message c))])
                     er2  (:probe::Echo/echo c2 (:probe::Echo::EchoRequest :msg "hi"))
                     _2   (:wat::kernel::send self (:wat::core::match er2 [:wat::kernel::RecvOutcome.Message {:msg __recv} (:wat::core::match __recv [:probe::Echo::EchoResponse.Ok {:reply reply} reply]
  [:probe::Echo::EchoResponse.RequestTooLarge {:bytes bytes :cap cap}
    (:wat::kernel::assertion-failed! :message "unexpected RequestTooLarge")]
  [:probe::Echo::EchoResponse.RequestMalformed {:path mpath :expected mexpected :got mgot}
    (:wat::kernel::assertion-failed! :message "unexpected RequestMalformed")])] [:wat::kernel::RecvOutcome.Lost {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message __cause))] [:wat::kernel::RecvOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")] [:wat::kernel::RecvOutcome.Closed {} (:wat::kernel::assertion-failed! :message "recv': peer closed")]))]
                    nil))))
     _   (:wat::core::match (:wat::kernel::peer-pid prober) 
           [:wat::core::Option.Some {:value p}
             (:wat::core::let
               [_  (:probe::echo/grant  eh (wat.type/Vector :- [wat.type/i64] p))
                _  (:wat::core::match (:wat::kernel::send prober ea) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
                r1 (:wat::kernel::recv prober)
                _r (:probe::echo/revoke eh (wat.type/Vector :- [wat.type/i64] p))
                ;; <<< the echo'/revoke line is REMOVED here (the counterfactual) >>>
                _  (:wat::core::match (:wat::kernel::send prober ea) [:wat::kernel::SendOutcome.Sent {} nil] [:wat::kernel::SendOutcome.HandleClosed {} nil] [:wat::kernel::SendOutcome.Stopped {} nil] [:wat::kernel::SendOutcome.Closed {:cause _c} nil] [:wat::kernel::SendOutcome.Failed {:cause _c} nil])
                rr2 (:wat::kernel::recv prober)
                r2 (:wat::core::match rr2
                     [:wat::kernel::RecvOutcome.Message {:msg m} m]
                     [:wat::kernel::RecvOutcome.Lost {:cause cause}
                       (:wat::kernel::assertion-failed! :message (:wat::kernel::LociDiedError/message cause))]
                     [:wat::kernel::RecvOutcome.Stopped {}
                       (:wat::kernel::assertion-failed! :message "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open")]
                     [:wat::kernel::RecvOutcome.Closed {}
                       (:wat::kernel::assertion-failed! :message "recv': prober closed unexpectedly")])]
               (:wat::kernel::println (:wat::string::concat "NOREVOKE-REACHED-END: " r2)))]
           [:wat::core::Option.None {}
             (:wat::kernel::assertion-failed! :message "peer-pid None on process prober")])]
    nil))

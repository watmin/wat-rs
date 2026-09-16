;; probe-two-faults-before-stop.wat — the DRIVEN witness for excursus 001
;; `every-status-arm-is-named`, EXPECTATIONS row 9.
;;
;; THE CLAIM UNDER TEST: two queued `Status::Faulted` used to kill the owner.
;; D1-a added a ONE-LEVEL drain to `<svc>/stop`: the FIRST recv handles Faulted
;; and re-recvs. The SECOND recv did not — its `_` wildcard raised
;;   "defservice stop: expected Status::Stopped"
;; which is a LIE (a Faulted DID arrive; the message names the thing that did
;; not) and a crash on a RECOVERABLE error. After the stone the second recv
;; returns a faced `StopOutcome::GaveUp` naming what actually happened.
;;
;; HOW TWO FAULTS ARE DRIVEN: a public op handler that raises produces exactly
;; one `Status::Faulted` on the lineage peer and the service KEEPS SERVING
;; (wat/service.wat, the `Outcome::Faulted` arm). So two boom calls queue two
;; faults ahead of the Admin::Stop ack. The raise severs the calling conn
;; (measured: the client sees `Lost`), so each boom dials its own conn.
;; Shape copied from probe-a-handler-raise-kills-the-service.wat.
;;
;; EXPECTED (post-stone):  boom1=Lost  boom2=Lost  stop=GaveUp waited-ms=… last=a second Status::Faulted arrived …
;; EXPECTED (pre-stone):   boom1=Lost  boom2=Lost  then the owner DIES with
;;                         "defservice stop: expected Status::Stopped".
;;
;; Run:
;;   ./target/release/wat wat-scripts/scratch-pad/probe-two-faults-before-stop.wat
;;
;; ⛔ Every observation PRINTS rather than raising — a probe that dies on the
;; thing it is measuring reports nothing.

(:wat::core::defsurface :tf::Svc :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :tf::Svc::GoRequest [n <- :wat::core::i64])
   (:wat::core::defenum :tf::Svc::GoResponse :wat::enum::Pure
     :Ok               [got <- :wat::core::i64]
     :RequestTooLarge  [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String
                        got <- :wat::core::String])]
  :features
  [(go [self <- :tf::Svc  req <- :tf::Svc::GoRequest] -> :tf::Svc::GoResponse :max-request-bytes 4096)])

(:wat::service::defservice :tf::svc
  :satisfies :tf::Svc
  :durable   [hits <- :wat::core::i64]
  :ephemeral []
  :impls
  [(go [s ctx req]
     ;; n = 0 → the handler raises → Status::Faulted up the lineage, service lives on.
     (:wat::core::if (:wat::i64::= (:tf::Svc::GoRequest/n req) 0)
       (:wat::kernel::assertion-failed!
         "tf: the handler raised — this is what drives Status::Faulted"
         :wat::core::None :wat::core::None)
       (:wat::service::Outcome::Continue s
         (:wat::core::Some (:tf::Svc::Reply::Go (:tf::Svc::GoResponse::Ok (:tf::Svc::GoRequest/n req))))
         (:wat::core::Vector :- [(:wat::service::Directed :- [:tf::Svc::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:tf::svc::Op])]))))]
  :stop (:wat::core::fn [s <- :tf::svc::State] -> :wat::core::i64
          (:tf::svc::Record/hits (:tf::svc::State/durable s))))

;; Dial a FRESH conn and send one raising request. Reports, never raises.
(:wat::core::defn :tf::boom
  [h <- :tf::svc::Handle  label <- :wat::core::String] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::connect (:tf::svc::Handle/addr h))
    ((:wat::kernel::ConnectOutcome::Connected p)
      (:wat::core::match (:tf::Svc/go p (:tf::Svc::GoRequest :n 0))
        ((:wat::kernel::RecvOutcome::Message _resp)
          (:wat::kernel::println (:wat::core::format "{l}=Message" :l label)))
        ((:wat::kernel::RecvOutcome::Lost _c)
          (:wat::kernel::println (:wat::core::format "{l}=Lost" :l label)))
        (:wat::kernel::RecvOutcome::Stopped
          (:wat::kernel::println (:wat::core::format "{l}=Stopped" :l label)))
        (:wat::kernel::RecvOutcome::Closed
          (:wat::kernel::println (:wat::core::format "{l}=Closed" :l label)))
        (:wat::kernel::RecvOutcome::TimedOut
          (:wat::kernel::println (:wat::core::format "{l}=TimedOut" :l label)))
        ((:wat::kernel::RecvOutcome::Malformed _c)
          (:wat::kernel::println (:wat::core::format "{l}=Malformed" :l label)))))
    ((:wat::kernel::ConnectOutcome::Refused _f)
      (:wat::kernel::println (:wat::core::format "{l}=connect-REFUSED" :l label)))
    ((:wat::kernel::ConnectOutcome::Rejected _f)
      (:wat::kernel::println (:wat::core::format "{l}=connect-REJECTED" :l label)))
    ((:wat::kernel::ConnectOutcome::Failed _f)
      (:wat::kernel::println (:wat::core::format "{l}=connect-FAILED" :l label)))))

;; Queue TWO faults on the lineage, ahead of whatever ack the owner asks for next.
(:wat::core::defn :tf::two-faults [h <- :tf::svc::Handle  tag <- :wat::core::String] -> :wat::core::nil
  (:wat::core::let
    [_ (:tf::boom h (:wat::core::format "{t}-boom1" :t tag))
     _ (:tf::boom h (:wat::core::format "{t}-boom2" :t tag))]
    nil))

;; Parametric so ONE reporter covers stop (StopOutcome :- [i64]) and
;; hibernate (StopOutcome :- [Record]). Prints the variant, never raises.
(:wat::core::defn :tf::say-stop :- [T]
  [label <- :wat::core::String  o <- (:wat::service::StopOutcome :- [:T])] -> :wat::core::nil
  (:wat::core::match o
    ((:wat::service::StopOutcome::Stopped _s)
      (:wat::kernel::println (:wat::core::format "{l}=Stopped" :l label)))
    ((:wat::service::StopOutcome::Gone _c)
      (:wat::kernel::println (:wat::core::format "{l}=Gone" :l label)))
    ((:wat::service::StopOutcome::GaveUp w l)
      (:wat::kernel::println (:wat::core::format "{lb}=GaveUp waited-ms={w} last={l}" :lb label :w w :l l)))))

(:wat::core::defn :tf::say-gate
  [label <- :wat::core::String  o <- :wat::service::GateOutcome] -> :wat::core::nil
  (:wat::core::match o
    ((:wat::service::GateOutcome::Applied)
      (:wat::kernel::println (:wat::core::format "{l}=Applied" :l label)))
    ((:wat::service::GateOutcome::Gone _c)
      (:wat::kernel::println (:wat::core::format "{l}=Gone" :l label)))
    ((:wat::service::GateOutcome::GaveUp w l)
      (:wat::kernel::println (:wat::core::format "{lb}=GaveUp waited-ms={w} last={l}" :lb label :w w :l l)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:tf::svc/start :locus (:wat::spawn::process) :record (:tf::svc::Record :hits 0))
     ;; NON-VACUITY first: the service answers a normal request, so an
     ;; all-failures run cannot be read as the wall.
     _ (:wat::core::match (:wat::kernel::connect (:tf::svc::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected p)
           (:wat::core::match (:tf::Svc/go p (:tf::Svc::GoRequest :n 7))
             ((:wat::kernel::RecvOutcome::Message _resp) (:wat::kernel::println "control=Message"))
             ((:wat::kernel::RecvOutcome::Lost _c) (:wat::kernel::println "control=Lost"))
             (:wat::kernel::RecvOutcome::Stopped (:wat::kernel::println "control=Stopped"))
             (:wat::kernel::RecvOutcome::Closed (:wat::kernel::println "control=Closed"))
             (:wat::kernel::RecvOutcome::TimedOut (:wat::kernel::println "control=TimedOut"))
             ((:wat::kernel::RecvOutcome::Malformed _c) (:wat::kernel::println "control=Malformed"))))
         ((:wat::kernel::ConnectOutcome::Refused _f) (:wat::kernel::println "control=connect-REFUSED"))
         ((:wat::kernel::ConnectOutcome::Rejected _f) (:wat::kernel::println "control=connect-REJECTED"))
         ((:wat::kernel::ConnectOutcome::Failed _f) (:wat::kernel::println "control=connect-FAILED")))
     ;; ── site 3 of the DESIGN: stop's SECOND recv ──────────────────────────
     _ (:tf::two-faults h "stop")
     _ (:tf::say-stop "stop     " (:tf::svc/stop h))
     ;; ── site 5: hibernate's SECOND recv (fresh service; stop terminated the last) ──
     h2 (:tf::svc/start :locus (:wat::spawn::process) :record (:tf::svc::Record :hits 0))
     _ (:tf::two-faults h2 "hib")
     _ (:tf::say-stop "hibernate" (:tf::svc/hibernate h2))
     ;; ── site 7: grant's SECOND recv ───────────────────────────────────────
     h3 (:tf::svc/start :locus (:wat::spawn::process) :record (:tf::svc::Record :hits 0))
     _ (:tf::two-faults h3 "grant")
     _ (:tf::say-gate "grant    " (:tf::svc/grant h3 (:wat::core::Vector :- [:wat::core::i64] 4242)))
     ;; ⭐ NO CLEANUP STOP HERE, and the reason is a FINDING. A first draft of this
     ;; probe called `(stop-faced (:tf::svc/stop h3))` to avoid leaking the child.
     ;; It DIED — and what it printed is the stone working:
     ;;   "defservice stop: got Status::PeersAllowed (expected Stopped)"
     ;; The grant above gave up on the ack, so the surplus `Status::PeersAllowed`
     ;; was still queued on the lineage when stop re-recv'd. `wat/service.wat`'s
     ;; own grant comment PREDICTED exactly this — "a surplus PeersAllowed ack left
     ;; on the lineage peer is read by a later stop as Message(other) → 'expected
     ;; Status::Stopped'" — and pre-stone that is the lie it printed. The variant is
     ;; now NAMED, so the prediction and the message finally agree. The raise itself
     ;; is correct behaviour (rule 4: a genuine protocol violation still raises) and
     ;; it is NOT faceable by the caller, so the probe simply does not ask. The two
     ;; leaked children are reaped when this program exits.
     ;; ── site 9: revoke's SECOND recv ──────────────────────────────────────
     h4 (:tf::svc/start :locus (:wat::spawn::process) :record (:tf::svc::Record :hits 0))
     _ (:tf::two-faults h4 "revoke")
     _ (:tf::say-gate "revoke   " (:tf::svc/revoke h4 (:wat::core::Vector :- [:wat::core::i64] 4242)))]
    nil))

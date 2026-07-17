;; Co-located fixture for probe_arc278_s2s_peer_on_process.rs — arc 278 S4d run+assert gate (FORK).
;;
;; PROMOTED from wat-scripts/probes/arc-278/s2s-process-probe.wat (only TYPE-CHECKED by the
;; wat-scripts load gate, never RUN). This lifts the s2s peer-holding into a run-and-assert SYSTEM
;; TEST on a PROCESS locus — the FORK path `journal'` inherits when process-hosted.
;;
;; The delta from the thread sibling is the crossing block: BOTH services fork to PROCESSES, and
;; caller' is born with a `post-spawn` hook that grants caller's child pid to echo's accept-gate
;; BEFORE caller''s :init dials echo' (grant-before-dial ordering — the hook fires owner-side with
;; the child ProcessLaunch{pid} after the fork, before :init ships). This proves:
;;   (1) the :peers manifest concat ships (:probe::Echo::surface-forms) into caller''s child bundle
;;       so the forked child resolves Echo/echo + its messages (else StartupError), AND
;;   (2) the held peer actually dials + round-trips across the process boundary.
;; Expect "echo:hi".

;; ── ECHO: the dialed surface + its service ──────────────────────────────────────
(:wat::core::defsurface :probe::Echo :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Echo::EchoRequest  [msg   <- :wat::core::String])
   (:wat::core::defrecord :probe::Echo::EchoResponse [reply <- :wat::core::String])]
  :features
  [(echo [self <- :probe::Echo  req <- :probe::Echo::EchoRequest] -> :probe::Echo::EchoResponse)])

(:wat::service::defservice :probe::echo'
  :satisfies :probe::Echo
  :durable   []
  :ephemeral []
  :impls
  [(echo [s req]
     (:wat::service::Outcome::Reply s
       (:probe::Echo::EchoResponse :reply
         (:wat::core::string::concat "echo:" (:probe::Echo::EchoRequest/msg req)))))])

;; ── CALLER: a surface + a service that DIALS echo' (the s2s peer) ───────────────
(:wat::core::defsurface :probe::Caller :nature :wat::kernel::Peer'
  :messages
  [(:wat::core::defrecord :probe::Caller::RunRequest  [])
   (:wat::core::defrecord :probe::Caller::RunResponse [out <- :wat::core::String])]
  :features
  [(run [self <- :probe::Caller  req <- :probe::Caller::RunRequest] -> :probe::Caller::RunResponse)])

(:wat::service::defservice :probe::caller'
  :satisfies :probe::Caller
  :durable   []
  ;; the dialed peer — a client Peer'<Echo::Op,Echo::Reply>, held as a ROOT ephemeral field
  :ephemeral [echo <- :wat::kernel::Peer'<probe::Echo::Op,probe::Echo::Reply>]
  ;; the explicit s2s dependency DAG — set-equal to the ephemeral peer field's surface
  :peers     [:probe::Echo]
  ;; :init connects to echo' (its Address' crosses the fork as an operating-input cap record)
  :init (:wat::core::fn
          [record    <- :probe::caller'::Record
           echo-addr <- :wat::kernel::Address'<probe::Echo::Op,probe::Echo::Reply>]
          -> :probe::caller'::State
          (:probe::caller'::State :durable record :echo (:wat::kernel::connect' echo-addr)))
  :impls
  [(run [s req]
     (:wat::core::let
       [echo (:probe::caller'::State/echo s)
        er   (:probe::Echo/echo echo (:probe::Echo::EchoRequest :msg "hi"))
        out  (:probe::Echo::EchoResponse/reply er)]
       (:wat::service::Outcome::Reply s (:probe::Caller::RunResponse :out out))))])

;; ── the crossing: start both on PROCESSES; grant-before-dial via post-spawn hook. Return reply. ──
(:wat::core::defn :user::compute [] -> :wat::core::String
  (:wat::core::let
    [eh  (:probe::echo'/start   :locus (:wat::spawn::process) :record (:probe::echo'::Record))
     ea  (:probe::echo'::Handle/addr eh)
     ch  (:probe::caller'/start
           :locus (:wat::spawn::process/post-spawn
                    (:wat::core::fn [pl <- :wat::spawn::ProcessLaunch] -> :wat::core::nil
                      (:probe::echo'/grant eh
                        (:wat::core::Vector :wat::core::i64 (:wat::spawn::ProcessLaunch/pid pl)))))
           :record (:probe::caller'::Record) :echo-addr ea)
     cc  (:wat::kernel::connect' (:probe::caller'::Handle/addr ch))
     rr  (:probe::Caller/run cc (:probe::Caller::RunRequest))]
    (:probe::Caller::RunResponse/out rr)))

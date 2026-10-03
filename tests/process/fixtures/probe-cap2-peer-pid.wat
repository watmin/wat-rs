;; probe-cap2-peer-pid.wat — arc 170 capability circuit, stone 2 DISCONFIRMING PROBE.
;;
;; Isolate the ONE gap: does (:wat::kernel::peer-pid p) lift the pid off a peer?
;;   process peer (connect' to a process service) -> (Some pid)
;;   thread  peer (connect' to a thread  service) -> :None
;; Pre-strike this fails on EXACTLY ":wat::kernel::peer-pid is undefined" — everything
;; around it (start the services, connect' → peers) is already green, so the whole
;; strike is "define :wat::kernel::peer-pid". EXPECT (post-strike): "(Some <pid>)" then ":None".
;;
;; DISPOSITION (255.75) — negative, stale claim: `DESIGN-STONE-CAP-2-BRACKET-GRANTS.md`'s own
;; "GROUNDED FINDING (strike A, 2026-07-09)" settled this differently — `peer-pid` is for
;; SPAWN-derived peers only (`Process'`/`Thread'`); a unified `Peer'` from `connect'` (what `pc`
;; below is) gets "an honest error", not `(Some pid)`. This probe dials via `connect'`, so its
;; EXPECT above is the question the grounded finding already answered NO to — the positive case
;; (spawn-derived) is covered by `wat-scripts/probes/arc-170/probe-cap2-spawnrunner-pid.wat`
;; (already green, untouched). a plain `.wat` (AMEND: this file starts up clean — `startup_from_file` succeeds, the
;; failure is runtime-only — and `tests/lint/every_wat_bad_fixture_actually_fails.rs` forbids
;; `.wat.bad` for a startup-clean file; per that gate's remedy #1, moved out of
;; `wat-scripts/probes/` to `tests/process/fixtures/` instead, so the "every probe runs, exit
;; 0" gate does not require it), driven by
;; `tests/process/probe_arc255_75_negative_probes.rs`, asserting the TypeMismatch names
;; `:wat::kernel::peer-pid` and the expected-peer union type.

(wat.core/defsurface probe/Echo :nature wat.kernel/Peer
  :messages
  [(wat.core/defrecord probe.Echo/EchoRequest  [msg   :- wat.type/String])
   (wat.core/defenum probe.Echo/EchoResponse wat.enum/Pure :Ok [reply :- wat.type/String] :RequestTooLarge [bytes :- wat.type/i64  cap :- wat.type/i64]
                                                                                                      :RequestMalformed [path :- (wat.type/Vector :- [wat.type/String])  expected :- wat.type/String  got :- wat.type/String])]
  :features
  [(echo [self :- probe/Echo  req :- probe.Echo/EchoRequest] :- probe.Echo/EchoResponse :max-request-bytes 524288)])

(wat.service/defservice probe/echo
  :satisfies probe/Echo  :durable [] :ephemeral []
  :impls [(echo [s ctx req] (wat.service/Outcome.Reply {:state s
                          :reply (probe.Echo/EchoResponse.Ok {:reply (probe.Echo.EchoRequest/msg req)})}))])

(wat.core/defn user/main [] :- wat.type/nil
  (wat.core/let
    [;; ── a PROCESS peer: its far end is a forked child → peer-pid should be (Some pid) ──
     ph  (probe.echo/start :locus (wat.spawn/process) :record (probe.echo/Record))
     pc  (wat.core/match (wat.kernel/connect (probe.echo.Handle/addr ph)) [wat.kernel/ConnectOutcome.Connected {:peer p} p] [wat.kernel/ConnectOutcome.Closed {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))] [wat.kernel/ConnectOutcome.Undialable {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))] [wat.kernel/ConnectOutcome.WrongPeer {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))] [wat.kernel/ConnectOutcome.Failed {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))])
     _   (wat.kernel/println "process-peer peer-pid:")
     _   (wat.kernel/println (wat.kernel/peer-pid pc))   ; ← THE GAP (undefined pre-strike)
     ;; ── a THREAD peer: its far end is a cell in THIS process → peer-pid should be :None ──
     th  (probe.echo/start :locus (wat.spawn/thread) :record (probe.echo/Record))
     tc  (wat.core/match (wat.kernel/connect (probe.echo.Handle/addr th)) [wat.kernel/ConnectOutcome.Connected {:peer p} p] [wat.kernel/ConnectOutcome.Closed {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))] [wat.kernel/ConnectOutcome.Undialable {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))] [wat.kernel/ConnectOutcome.WrongPeer {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))] [wat.kernel/ConnectOutcome.Failed {:cause c} (wat.kernel/assertion-failed! :message (wat.kernel.Failure/message c))])
     _   (wat.kernel/println "thread-peer peer-pid:")
     _   (wat.kernel/println (wat.kernel/peer-pid tc))]
    nil))

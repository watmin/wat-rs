;; probe-closed-is-recoverable.wat — THE DISCONFIRMING PROBE for "Closed is not always death".
;;
;; probe-frame-cap-severs-one-conn.wat established the mechanism:
;;   a-small=ok ; a-big=lost ; b-other=ok ; a-again=closed
;; The FIRST failure on a severed connection is Lost; every SUBSEQUENT touch is CLOSED,
;; and B's separate connection kept working — the SERVICE WAS ALIVE THE WHOLE TIME.
;;
;; The corpus treats that Closed as death: `assertion-failed! "ack closed"`. Chaos produces
;; exactly this state, so today a client-side drop would kill workers instead of exercising
;; recovery.
;;
;; ★ THE ONE THING THE FIX RESTS ON, AND NOTHING HAS TESTED IT:
;;   after a connection is severed AND its stale handle has returned Closed,
;;   does re-dialing the SAME address produce a WORKING connection?
;;
;; If yes, `Closed` joins `Lost` on the recovery path and the stone is a uniform change.
;; If no, the recovery arms are unreachable from Closed and the stone must not be drawn.
;;
;;   expect: a-small=ok;a-big=lost;a-again=closed;a-REDIAL=ok;b-still=ok

(:wat::config::set-redef! true)
(:wat::load-file! "probe-frame-cap-severs-one-conn.wat")

(:wat::core::defn :cr::run [] -> :wat::core::String
  (:wat::core::let
    [h  (:fc::echo/start :locus (:wat::spawn::process) :record (:fc::echo::Record))
     a  (:fc::dial (:fc::echo::Handle/addr h))
     b  (:fc::dial (:fc::echo::Handle/addr h))
     small (:fc::hit a "")                       ;; sanity: A works
     big   (:fc::hit a (:fc::pad 200))           ;; over the 256B frame cap — severs A
     again (:fc::hit a "")                       ;; stale handle -> Closed
     a2    (:fc::dial (:fc::echo::Handle/addr h)) ;; ★ RE-DIAL THE SAME ADDRESS
     redial (:fc::hit a2 "")                     ;; ★ does the fresh connection work?
     still (:fc::hit b "")]                      ;; B untouched throughout
    (:wat::core::format "a-small={s};a-big={g};a-again={r};a-REDIAL={d};b-still={o};verdict={v}"
      :s small :g big :r again :d redial :o still
      :v (:wat::core::if (:wat::core::= redial "ok")
           "CLOSED-IS-RECOVERABLE"
           "closed-is-terminal-DO-NOT-DRAW"))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println (:cr::run)))

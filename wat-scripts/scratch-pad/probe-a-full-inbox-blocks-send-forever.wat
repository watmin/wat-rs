;; MEASUREMENT for the send half of "both sides must be able to time out and recover".
;;
;; THE CLAIM UNDER TEST: `:wat::kernel::send` into a peer whose receiver is not
;; draining BLOCKS, and `SendOutcome` (Sent | Closed | Lost | Stopped) has **no
;; variant that can report it** — so the site cannot even write a losing arm. The
;; recv half always had an unreachable `TimedOut` arm to make reachable; the send
;; half has nothing to make reachable. That asymmetry is the finding, and it must
;; be measured rather than read off the variant list.
;;
;; SHAPE: copied from probe-crash-surface-try-send-wouldblock.wat (the service
;; never reads — its `ping` impl returns Continue with no reply), so the inbox
;; fills. Flood with `try-send` until `WouldBlock` proves it is full, print that
;; boundary, then issue ONE plain `send` and print the outcome.
;;
;; ⛔ HOW TO READ IT — the two worlds are distinguished by the LAST LINE, never by
;; the exit code:
;;   `full-at=<n>` then NOTHING more, killed by `timeout`  → the send BLOCKED. The
;;      claim holds: an unbounded send exists and is unreportable.
;;   `full-at=<n>` then `plain-send=<variant>`             → the claim is FALSE and
;;      this probe refutes the stone it was drawn for. Report the variant.
;; Run it deliberately, never in a suite, and ALWAYS with -k (a blocked wat ignores
;; SIGTERM — 125 s measured 2026-09-15):
;;   timeout -k 2 40 ./target/release/wat \
;;     wat-scripts/scratch-pad/probe-a-full-inbox-blocks-send-forever.wat > /tmp/sb.log 2>&1 &
;; ⛔ DO NOT PIPE IT TO `tail`/`head` while it runs. The first attempt did, saw NOTHING,
;; and could not tell "still filling" from "blocked in the send" — the pager was holding
;; the line. Redirect to a FILE and read the file WHILE IT RUNS; that is what separated
;; the two worlds.
;;
;; MEASURED 2026-09-15, process tier, fixture 2 (the parking handler):
;;   t≈2 s   `full-at=283`                  — the inbox is durably full
;;   t≈5/10/20 s  still ONE line             — ⭑ the plain `send` is BLOCKED, ≥38 s
;;   at the kill  `plain-send=Stopped`       — it returned ONLY because `timeout`'s SIGTERM
;;                                             killed the service and the peer died
;; ⭐⭐ THE FINDING, and it is worse than the recv half: `SendOutcome` is
;; `Sent | Closed | Lost | Stopped` — **no variant describes "blocked / not draining"**.
;; The value this site finally observed names the peer's DEATH, so a caller cannot
;; distinguish "I blocked 38 s and then it died" from "it was already stopped". The recv
;; half always had an unreachable `TimedOut` arm to make reachable (arc 109's painted
;; brick); the send half has no arm at all, and no bounded send primitive exists anywhere
;; in the corpus — `try-send`'s bound is ZERO, not a deadline.
;; ⚠ One asymmetry in the other direction, worth keeping: this blocked `send` DID return
;; when its peer died, where a blocked bare `recv` ignored SIGTERM for 125 s.

(:wat::core::defsurface :hold::Hold :nature :wat::kernel::Peer
  :messages
  [(:wat::core::defrecord :hold::Hold::PingRequest [])
   (:wat::core::defenum :hold::Hold::PingResponse :wat::enum::Pure
     :Ok []
     :RequestTooLarge [bytes <- :wat::core::i64  cap <- :wat::core::i64]
     :RequestMalformed [path <- (:wat::core::Vector :- [:wat::core::String])
                        expected <- :wat::core::String  got <- :wat::core::String])]
  :features
  [(ping [self <- :hold::Hold  req <- :hold::Hold::PingRequest] -> :hold::Hold::PingResponse
     :max-request-bytes 65536)])

(:wat::service::defservice :hold::hold
  :satisfies :hold::Hold
  :durable []
  :ephemeral []
  :impls
  ;; ⛔ THE HANDLER PARKS 120 s. Fixture 1 of this probe had a handler that returned
  ;; immediately, and it REFUTED ITSELF: `try-send` reported WouldBlock at n=376 and a
  ;; plain `send` one microsecond later returned `Sent`. The reason is that a
  ;; `defservice` serve loop DOES drain — it recvs every Op and dispatches it; what
  ;; fixture 1's handler never did was REPLY. So WouldBlock there was transient socket
  ;; pressure, not a full inbox. ⭑ Recorded because it is a load-bearing negative:
  ;; **`TrySendOutcome::WouldBlock` does NOT imply the receiver has stopped draining.**
  ;; A handler that parks stops the loop, so the inbox stays full.
  [(ping [s ctx req]
     (:wat::core::let
       [_ (:wat::core::match (:wat::kernel::recv
              (:wat::kernel::after :wat::program::PeerKind::thread
                (:wat::time::Milliseconds 120000) :done))
            ((:wat::kernel::RecvOutcome::Message _m) nil)
            (_ nil))]
       (:wat::service::Outcome::Continue s
         :wat::core::None
         (:wat::core::Vector :- [(:wat::service::Directed :- [:hold::Hold::Reply])])
         (:wat::core::Vector :- [(:wat::service::Alarm :- [:hold::hold::Op])]))))])

;; Fill until the inbox refuses. `cap` is a give-up bound so a NON-filling world
;; reports "never-full" instead of running forever — the probe must not hang for
;; the reason it is testing for.
(:wat::core::defn :sb::fill
  [p <- (:wat::kernel::Peer :- [:hold::Hold::Op :hold::Hold::Reply])
   n <- :wat::core::i64
   cap <- :wat::core::i64] -> :wat::core::String
  (:wat::core::if (:wat::i64::>= n cap)
    (:wat::core::format "never-full-after={n}" :n n)
    (:wat::core::match
      (:wat::kernel::try-send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
      (:wat::kernel::TrySendOutcome::Sent (:sb::fill p (:wat::i64::+ n 1) cap))
      (:wat::kernel::TrySendOutcome::WouldBlock
        (:wat::core::format "full-at={n}" :n n))
      (:wat::kernel::TrySendOutcome::Closed
        (:wat::core::format "closed-at={n}" :n n))
      ((:wat::kernel::TrySendOutcome::Lost _)
        (:wat::core::format "lost-at={n}" :n n)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [h (:hold::hold/start :locus (:wat::spawn::process) :record (:hold::hold::Record))
     p (:wat::core::match (:wat::kernel::connect (:hold::hold::Handle/addr h))
         ((:wat::kernel::ConnectOutcome::Connected c) c)
         (_ (:wat::kernel::assertion-failed! "probe: connect failed" :wat::core::None :wat::core::None)))
     _ (:wat::kernel::println (:sb::fill p 0 100000))
     ;; ⛔ THE MEASUREMENT. One plain send into an inbox that just refused a
     ;; try-send. Every SendOutcome variant is named so a returning world is
     ;; reported rather than crashing the probe — a match that raised here would
     ;; look identical to a block in the output.
     _ (:wat::kernel::println
         (:wat::core::match (:wat::kernel::send p (:hold::Hold::Op::Ping (:hold::Hold::PingRequest)))
           (:wat::kernel::SendOutcome::Sent "plain-send=Sent")
           (:wat::kernel::SendOutcome::Closed "plain-send=Closed")
           (:wat::kernel::SendOutcome::Stopped "plain-send=Stopped")
           ((:wat::kernel::SendOutcome::Lost _) "plain-send=Lost")))
     _ (:wat::service::stop-faced (:hold::hold/stop h))]
    nil))

;; probe-self-scheduling-loop.wat — DISCONFIRMING PROBE (arc 278 item (c) — self-scheduling).
;;
;; Prove the LOAD-BEARING mechanism the self-scheduling stone will encode into the generated serve
;; loop: a hand-rolled TCO loop that (1) select's over a MUTATING peer-set delivering a SHARED Op
;; enum, (2) dispatches by variant, (3) buffers items via threaded state, (4) INSERTS a timer into
;; the set mid-loop (the "arm" — the builder's "insert into its reactor"), (5) flushes on the
;; flush-variant. Thread-tier here (feasibility); process rides the identical shape (env-grab,
;; arc-292 both-loci-proven). The {real-client-peer + timer} MIX is grounded (arc-292 REALIZATIONS
;; l.63-64 "select' unchanged", + the bracket collect-loop select's real peers) — here the sources
;; are timers so the proof is self-contained + deterministic (counts re-arms, not wall-clock).
;;
;; GREEN = the mechanism composes; the flushed batch survived to the tick → the stone is feasible,
;; this loop is the exemplar the shadowdancer transcribes. RED = the composition is wrong; the
;; checker names exactly where, before a shadowdancer is spent.

;; The shared select' message — a client "item" OR the internal flush tick (ONE homogeneous O):
(:wat::core::defenum :probe::SinkSig :wat::enum::Pure
  :Item      [v <- :wat::core::i64]
  :FlushTick [])

;; the loop: threads the live timer-peer set + the buffer + whether a flush is already armed.
;; returns the flushed batch on the tick (proving items buffered BEFORE the tick survive to it).
(:wat::core::defn :probe::sink-loop
  [peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [:wat::core::nil :probe::SinkSig])])
   buf   <- (:wat::core::Vector :- [:wat::core::i64])
   armed <- :wat::core::bool]
  -> (:wat::core::Vector :- [:wat::core::i64])
  (:wat::core::match (:wat::kernel::select peers) 
    ((:wat::spawn::ServiceEvent::Message idx sig)
      (:wat::core::match sig 
        ;; :Item → buffer it; if no flush armed, ARM one (INSERT a fresh Timer' into the set):
        ((:probe::SinkSig::Item v)
          (:wat::core::let
            [peers0 (:wat::std::list::remove-at peers idx)       ;; drop the fired one-shot item-timer
             buf'   (:wat::core::conj buf v)]
            (:wat::core::if armed
              (:probe::sink-loop peers0 buf' true)
              (:probe::sink-loop
                (:wat::core::conj peers0                          ;; <-- arm: insert a FlushTick timer
                  (:wat::kernel::after :wat::program::PeerKind::thread
                    (:wat::time::Millisecond 50) (:probe::SinkSig::FlushTick)))
                buf' true))))
        ;; :FlushTick → flush: return the buffered batch (survived to the tick):
        ((:probe::SinkSig::FlushTick) buf)))
    ;; timers never Close/Lost/etc — but the match is exhaustive (no-hidden-failures):
    ((:wat::spawn::ServiceEvent::Closed _idx) buf)
    ((:wat::spawn::ServiceEvent::Lost _idx _c) buf)
    ((:wat::spawn::ServiceEvent::Malformed _idx _c) buf)
    ((:wat::spawn::ServiceEvent::Rejected _idx _c) buf)
    (:wat::spawn::ServiceEvent::Shutdown buf)
    ((:wat::spawn::ServiceEvent::Connection _p) buf)
    ((:wat::spawn::ServiceEvent::Admin _m) buf)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; seed 3 staggered item-timers (1/2/3 ms) — simulate 3 client pushes before the flush:
     items (:wat::core::Vector (:wat::kernel::Peer :- [:wat::core::nil :probe::SinkSig])
             (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond 1) (:probe::SinkSig::Item 10))
             (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond 2) (:probe::SinkSig::Item 20))
             (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Millisecond 3) (:probe::SinkSig::Item 30)))
     flushed (:probe::sink-loop items (:wat::core::Vector :wat::core::i64) false)]
    (:wat::kernel::println flushed)))     ;; EXPECT: [10 20 30] — all 3 buffered, flushed on the tick

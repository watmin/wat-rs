;; PROBE — a-peer-wait-can-be-bounded-too. DIFFERENTIAL + negative control.
;;
;; CONTROL: recv-by-deadline on a silent Peer must return TimedOut (the refusal
;;   "expected owner handle … got :wat::kernel::Peer" is gone; the variant is
;;   producible on a Peer).
;; NEGATIVE: a peer that sends inside the deadline must return Message, not
;;   TimedOut. A bound that always fires is a broken clock.
;; SUBJECT: a BARE recv on the same silent peer still blocks. If both halves
;;   return, the probe has stopped discriminating.

(:wat::core::defn :br::silent-peer [] -> (:wat::kernel::Peer :- [:wat::core::keyword :wat::core::keyword])
  (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds 3600000) :never))

(:wat::core::defn :br::name [o <- (:wat::kernel::RecvOutcome :- [:wat::core::keyword])] -> :wat::core::String
  (:wat::core::match o
    ((:wat::kernel::RecvOutcome::Message _m) "Message")
    ((:wat::kernel::RecvOutcome::Lost _c) "Lost")
    (:wat::kernel::RecvOutcome::Stopped "Stopped")
    (:wat::kernel::RecvOutcome::Closed "Closed")
    (:wat::kernel::RecvOutcome::TimedOut "TimedOut")
    ((:wat::kernel::RecvOutcome::Malformed _c) "Malformed")))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; CONTROL — recv-by-deadline on a Peer that will not send inside the window.
     c (:wat::kernel::recv-by-deadline (:br::silent-peer) 300)
     _ (:wat::kernel::println (:wat::string::concat "control=" (:br::name c)))
     ;; NEGATIVE — a peer that DOES send inside the deadline.
     n (:wat::kernel::recv-by-deadline
          (:wat::kernel::after :wat::program::PeerKind::thread (:wat::time::Milliseconds 1) :tick)
          300)
     _ (:wat::kernel::println (:wat::string::concat "negative=" (:br::name n)))
     _ (:wat::kernel::println "subject-starting")
     ;; SUBJECT — the same wait, BARE. Must still block.
     s (:wat::kernel::recv (:br::silent-peer))
     _ (:wat::kernel::println (:wat::string::concat "subject=" (:br::name s)))]
    nil))

;; tests/comms/probe_arc293_W2c_controls.wat
;; Control fixture for probe_arc293_W2c_compile_time_send.rs.
;;
;; Arc 293.W.2c — guards against over-rejection by the send' wire-wall.
;;
;; Two controls that MUST type-check (world loads without error):
;;
;;   1. struct over a THREAD peer — exempt (in-locus, same address space,
;;      crossbeam channel; no serialization). The gate must NOT fire for Thread'.
;;
;;   2. record over a PROCESS peer — portable (records are wire-serializable).
;;      The gate must NOT fire for portable payload types.

(:wat::core::defstruct :w2c_ctrl::S [val <- :wat::core::i64])
(:wat::core::defrecord :w2c_ctrl::R [val <- :wat::core::i64])

;; Thread control: parent spawns a thread child that echoes structs back via
;; its Peer' self-handle. The gate must not fire — Thread' is in-locus.
(:wat::core::defn :w2c_ctrl::probe-send-struct-thread [] -> :wat::core::i64
  (:wat::core::let
    [peer (:wat::test::spawn-peer (:wat::spawn::thread)
            (:wat::core::fn [self <- (:wat::kernel::ThreadSelfPeer :- [:w2c_ctrl::S :w2c_ctrl::S])] -> :wat::core::nil
              (:wat::core::match
                (:wat::kernel::send self
                  (:wat::core::match (:wat::kernel::recv self)
                    ((:wat::kernel::RecvOutcome::Message m) m)
                    ((:wat::kernel::RecvOutcome::Lost cause)
                      (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
                    (:wat::kernel::RecvOutcome::Stopped
                      (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
                    (:wat::kernel::RecvOutcome::Closed
                      (:wat::kernel::assertion-failed! "recv': self closed unexpectedly" :wat::core::None :wat::core::None))))
                (:wat::kernel::SendOutcome::Sent nil)
                (:wat::kernel::SendOutcome::Closed nil)
                ((:wat::kernel::SendOutcome::Lost _c) nil)
                (:wat::kernel::SendOutcome::Stopped nil))))  ;; arc 278 #73 — fire-and-forget echo; outcome ignored uniformly regardless of cause
     _   (:wat::core::match (:wat::kernel::send peer (:w2c_ctrl::S :val 99))
           (:wat::kernel::SendOutcome::Sent nil)
           (:wat::kernel::SendOutcome::Closed nil)
           ((:wat::kernel::SendOutcome::Lost _c) nil)
           (:wat::kernel::SendOutcome::Stopped nil)) ;; arc 278 #73 — fire-and-forget request; outcome ignored uniformly regardless of cause
     got (:wat::core::match (:wat::kernel::recv peer)
           ((:wat::kernel::RecvOutcome::Message m) m)
           ((:wat::kernel::RecvOutcome::Lost cause)
             (:wat::kernel::assertion-failed! (:wat::kernel::LociDiedError/message cause) :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Stopped
             (:wat::kernel::assertion-failed! "recv': stopped — the substrate was asked to stop; the peer was ALIVE and the channel open" :wat::core::None :wat::core::None))
           (:wat::kernel::RecvOutcome::Closed
             (:wat::kernel::assertion-failed! "recv': peer closed unexpectedly" :wat::core::None :wat::core::None)))]
    (:w2c_ctrl::S/val got)))

;; Record control: parent sends a portable record to a PROCESS child.
;; Records are wire-serializable; the gate must not fire.
(:wat::core::defn :w2c_ctrl::probe-send-record-to-process [] -> :wat::core::nil
  (:wat::core::let
    [p (:wat::test::spawn-peer (:wat::spawn::process)
         (:wat::core::forms
           (:wat::core::defrecord :w2c_ctrl::R [val <- :wat::core::i64])
           (:wat::core::defn :user::main [] -> :wat::core::nil (:wat::kernel::println "spawned child"))))]
    (:wat::core::match (:wat::kernel::send p (:w2c_ctrl::R :val 42))
      (:wat::kernel::SendOutcome::Sent nil)
      (:wat::kernel::SendOutcome::Closed nil)
      ((:wat::kernel::SendOutcome::Lost _c) nil)
      (:wat::kernel::SendOutcome::Stopped nil)))) ;; arc 278 #73 — fire-and-forget record send; outcome ignored uniformly regardless of cause

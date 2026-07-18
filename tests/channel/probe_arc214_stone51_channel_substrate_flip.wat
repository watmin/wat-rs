;; tests/channel/probe_arc214_stone51_channel_substrate_flip.wat — co-located fixture
;; (just-eval rubric, docs/CONVENTIONS.md § Test idioms: a VALUE/TYPE claim — make-channel's
;; returned Sender/Receiver, inspected for its comms-backed Debug fingerprint). No process
;; boundary participates.
;;
;; Two zero-arg entries, one per half of the pair — avoids needing a Tuple-of-opaque-handles
;; return type; each just returns the half the corresponding Rust test inspects.
(:wat::core::defn :user::compute-receiver [] -> :wat::kernel::Receiver<wat::core::i64>
  (:wat::core::let [[_tx rx] (:wat::kernel::make-channel :wat::core::i64)] rx))

(:wat::core::defn :user::compute-sender [] -> :wat::kernel::Sender<wat::core::i64>
  (:wat::core::let [[tx _rx] (:wat::kernel::make-channel :wat::core::i64)] tx))

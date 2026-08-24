;; probe-s3a-select-peer.wat — DISCONFIRMING probe for 259 S3a (add Peer' to select').
;;
;; A fn that holds a HOMOGENEOUS (Vector :- [(Peer' :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])]) and calls select' on it.
;; select''s runtime (eval_peer_select_prime, runtime.rs:26070) already dispatches on the
;; concrete value's type_path (Thread'/Process'), so the ONLY gap is the TYPE CHECKER:
;; infer_select_prime (check.rs:12098-12105) hardcodes the element head to Thread'/Process'
;; and REJECTS the abstract Peer'.
;;
;; RED at HEAD: freezing :probe::sel errors with
;;   TypeMismatch select' peers expected "Vector of Thread'<I,O> | Process'<I,O> peers"
;;   got Peer'<(i64,i64),(i64,i64)>
;; on EXACTLY that gap — everything else is clean (the fn body just binds + discards).
;;
;; GREEN after S3a: :probe::sel type-checks (select' accepts the Peer' element, returns
;; (ServiceEvent :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) ?])), main prints "ok".

;; peers held at the ABSTRACT Peer' type — the shape the bracket's collect-loop wants
;; ((Vector :- [(Peer' :- [(:wat::core::Tuple :- [:wat::core::i64 I]) (:wat::core::Tuple :- [:wat::core::i64 O])])])). Positionally identical to (Thread' :- [(:wat::core::Tuple :- [:wat::core::i64 I]) (:wat::core::Tuple :- [:wat::core::i64 O])]).
(:wat::core::defn :probe::sel
  [peers <- (:wat::core::Vector :- [(:wat::kernel::Peer :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64]) (:wat::core::Tuple :- [:wat::core::i64 :wat::core::i64])])])]
  -> :wat::core::nil
  (:wat::core::let [_ev (:wat::kernel::select peers)]
    nil))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::kernel::println "ok"))

;; tests/channel/wat_arc170_channel_pipes.wat — co-located fixture for the wat-level-verb
;; dispatch tests (just-eval rubric, docs/CONVENTIONS.md § Test idioms). Each verb's channel
;; argument (Sender/Receiver, tier-1 crossbeam or tier-2 PipeFd — the wat type is tier-blind)
;; is a Rust-native handle minted dynamically by the `.rs` driver — impure/non-EDN by
;; construction, so it can't cross a process boundary; these fns take it as an argument and
;; are driven via `apply_function`, not the zero-arg `call_beside` convenience.

;; Arc 212 stone δ-comm-positions law: a bare send/recv as a defn body is
;; CommCallOutOfPosition (Forbidden position). The comm call must be the
;; scrutinee of a match (or the value-position of Result/Option `expect`).
;; Each fn below wraps its comm call as a match scrutinee and reconstructs
;; the identical Result shape on both arms — this satisfies the position
;; law while returning EXACTLY what the bare call would have returned, so
;; the .rs assertions (which inspect Ok/Err and the inner Option) are
;; unchanged.
(:wat::core::defn :user::do-send-7
  [tx <- :wat::kernel::Sender<wat::core::i64>]
  -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>
  (:wat::core::match (:wat::kernel::send tx 7) -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>
    ((:wat::core::Ok v) (:wat::core::Ok v))
    ((:wat::core::Err e) (:wat::core::Err e))))

(:wat::core::defn :user::do-send-99
  [tx <- :wat::kernel::Sender<wat::core::i64>]
  -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>
  (:wat::core::match (:wat::kernel::send tx 99) -> :wat::core::Result<wat::core::nil,wat::core::Vector<wat::kernel::ThreadDiedError>>
    ((:wat::core::Ok v) (:wat::core::Ok v))
    ((:wat::core::Err e) (:wat::core::Err e))))

(:wat::core::defn :user::do-recv
  [rx <- :wat::kernel::Receiver<wat::core::i64>]
  -> :wat::core::Result<wat::core::Option<wat::core::i64>,wat::core::Vector<wat::kernel::ThreadDiedError>>
  (:wat::core::match (:wat::kernel::recv rx) -> :wat::core::Result<wat::core::Option<wat::core::i64>,wat::core::Vector<wat::kernel::ThreadDiedError>>
    ((:wat::core::Ok (:wat::core::Some v)) (:wat::core::Ok (:wat::core::Some v)))
    ((:wat::core::Ok :wat::core::None)    (:wat::core::Ok :wat::core::None))
    ((:wat::core::Err e)                  (:wat::core::Err e))))

(:wat::core::defn :user::do-select
  [rxs <- :wat::core::Vector<wat::kernel::Receiver<wat::core::i64>>]
  -> :(wat::core::i64,wat::core::Result<wat::core::Option<wat::core::i64>,wat::core::Vector<wat::kernel::ThreadDiedError>>)
  (:wat::kernel::select rxs))

(:wat::core::defn :user::do-sender-close
  [tx <- :wat::kernel::Sender<wat::core::i64>]
  -> :wat::core::nil
  (:wat::kernel::Sender/close tx))

;; wat-scripts/fixes/retarget-peer-purity-probes.wat — arc 278: the §7 wire-peer purity
;; assertion outlives its host.
;;
;; `peer-pair'` is annihilated (builder-ruled: "i don't think we need this thing"). Three
;; arc-293 probes used it purely as a HOST for the purity wall — they bind the result to `_`
;; and discard it; only the CHECK matters. The wall itself is alive and enforced by three
;; surviving producers (`connect'`, `accept'`, `:wat::program::self-peer`).
;;
;; `:wat::program::self-peer` is the drop-in: same shape as the dead verb — two type-keyword
;; args — and it runs the same `check_wire_peer_purity` on both. Proven by a run before this
;; codemod was written:
;;   (:wat::program::self-peer :x::S :wat::core::i64)   => EXIT 1, MalformedForm,
;;       :reason "a wire peer (Peer'<I,O>) type arg must be a PURE type …"
;;   (:wat::program::self-peer :wat::core::i64 :wat::core::i64) => EXIT 0
;; So the negative probes stay RED-for-the-right-reason and the positive stays green.
;;
;; Exact FQDN swap. Idempotent (the old name no longer exists after the pass).
;;
;; Usage:
;;   printf '["tests/comms/probe_arc293_W2d_peer_purity.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/retarget-peer-purity-probes.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::kernel::peer-pair'" ":wat::program::self-peer"
    src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

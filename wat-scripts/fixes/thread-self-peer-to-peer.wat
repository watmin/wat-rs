;; wat-scripts/fixes/thread-self-peer-to-peer.wat — stone 255.30.
;; SCOPE: corpus
;;
;; Self-hosted, comment-faithful. ThreadSelfPeer is deleted. Every code
;; keyword `:wat::kernel::ThreadSelfPeer` becomes `:wat::kernel::Peer`.
;; The one-way derive
;; `(:wat::core::derive :wat::kernel::Peer :wat::kernel::ThreadSelfPeer)`
;; is removed, not rewritten into a self-derive. There is no alias.
;; A respelling is `(:wat::kernel::Peer :- [S R])`.
;;
;; Comments are not keywords, so they keep the old name. String literals
;; keep it too (the recorded prime-name table in reclaim-ipc-prime-names).
;;
;; Idempotent: after one run the derive form and the keyword are gone.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["pathA" "pathB"]\n' | ./target/release/wat ./wat-scripts/fixes/thread-self-peer-to-peer.wat

(:wat::core::defn :user::literal-replace
  [src <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String] -> :wat::core::String
  (:wat::string::join new (:wat::string::split src old)))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [src1 (:user::literal-replace src
            "(:wat::core::derive :wat::kernel::Peer :wat::kernel::ThreadSelfPeer)"
            "")
     src2 (:wat::fix::rename-keyword-exact
            ":wat::kernel::ThreadSelfPeer" ":wat::kernel::Peer" src1)]
    src2))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[thread-self-peer->peer] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v paths} paths]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])))

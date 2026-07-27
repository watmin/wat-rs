;; wat-scripts/fixes/reclaim-ipc-prime-names.wat — arc 278 "0z": the IPC primes reclaim their
;; plain names. Every non-prime IPC name is annihilated and proven dead (UnknownFunction on a
;; real run; whole-tree sweep at zero), so the `'` comes off.
;;
;; Exact FQDN symbol swaps. Idempotent by construction (a `'` removal cannot match twice).
;;
;; NOT here, each for its own reason:
;;   peer-pair'   builder-ruled DEAD — an annihilation, not a reclamation
;;   readln'      a live macro/verb pair (readln the defmacro expands to readln')
;;   Frame'       the positional-constructor idiom, not a successor
;;   fire-rules' / fire-once' / fire-rules-explain' / step-payload'
;;                the rete dual-impl — unprimed is the wat ORACLE, primed is the native
;;                kernel, differential-tested against each other (R9/R22). Never collapse.
;;
;; Usage (one EDN vector of EVERY path on stdin — generate the list, never hand-type it):
;;   printf '["a.wat" "b.wat" …]\n' | ./target/release/wat ./wat-scripts/fixes/reclaim-ipc-prime-names.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-exact ":wat::kernel::spawn-program'" ":wat::kernel::spawn-program"
    (:wat::fix::rename-keyword-exact ":wat::kernel::spawn-thread'" ":wat::kernel::spawn-thread"
      (:wat::fix::rename-keyword-exact ":wat::kernel::spawn-process'" ":wat::kernel::spawn-process"
        (:wat::fix::rename-keyword-exact ":wat::kernel::try-send'" ":wat::kernel::try-send"
          (:wat::fix::rename-keyword-exact ":wat::kernel::send'" ":wat::kernel::send"
            (:wat::fix::rename-keyword-exact ":wat::kernel::recv'" ":wat::kernel::recv"
              (:wat::fix::rename-keyword-exact ":wat::kernel::select'" ":wat::kernel::select"
                (:wat::fix::rename-keyword-exact ":wat::kernel::poll'" ":wat::kernel::poll"
                  (:wat::fix::rename-keyword-exact ":wat::kernel::close'" ":wat::kernel::close"
                    (:wat::fix::rename-keyword-exact ":wat::kernel::connect'" ":wat::kernel::connect"
                      (:wat::fix::rename-keyword-exact ":wat::kernel::accept'" ":wat::kernel::accept"
                        (:wat::fix::rename-keyword-exact ":wat::kernel::listener'" ":wat::kernel::listener"
                          (:wat::fix::rename-keyword-exact ":wat::kernel::Listener'" ":wat::kernel::Listener"
                            (:wat::fix::rename-keyword-exact ":wat::kernel::Address'" ":wat::kernel::Address"
                              (:wat::fix::rename-keyword-exact ":wat::kernel::Peer'" ":wat::kernel::Peer"
                                (:wat::fix::rename-keyword-exact ":wat::kernel::Thread'" ":wat::kernel::Thread"
                                  (:wat::fix::rename-keyword-exact ":wat::kernel::Process'" ":wat::kernel::Process"
                                    src))))))))))))))))))

(:wat::core::defn :user::apply-each
  [paths <- :wat::core::Vector<wat::core::String>] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::kernel::readln)))

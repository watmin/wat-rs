;; wat-scripts/fixes/rete-oracle-sigil.wat — rete dual-impl names: public = native,
;; oracle = `$oracle` sigil. Self-hosted fix-wat: no hand-edit of the corpus.
;;
;;   :wat::rete::fire-rules-spec     -> :wat::rete::fire-rules$oracle
;;   :wat::rete::insert-spec         -> :wat::rete::insert$oracle
;;   :wat::rete::insert-all-spec     -> :wat::rete::insert-all$oracle
;;   :wat::rete::fire-once           -> :wat::rete::fire-once$oracle   (was the wat oracle)
;;   :wat::rete::fire-once'          -> :wat::rete::fire-once          (native)
;;   :wat::rete::fire-rules'         -> :wat::rete::fire-rules
;;   :wat::rete::fire-rules-explain' -> :wat::rete::fire-rules-explain
;;   :wat::rete::insert'             -> :wat::rete::insert
;;   :wat::rete::insert-all'         -> :wat::rete::insert-all
;;   :wat::rete::arm-session'        -> :wat::rete::arm-session
;;   :wat::rete::step-payload'       -> :wat::rete::step-payload
;;
;; Exact whole-token (`rename-keyword-exact`): `fire-once` does not touch `fire-once'`;
;; `insert` does not touch `insert-all`. Order: oracle suffixes first, then unprimed
;; `fire-once` → `$oracle`, then drop the native `'`.
;;
;; Prime `'` stays the language native/IPC marker; rete no longer uses it for the
;; kernel. `$oracle` is the odd name a differential must type on purpose.
;;
;; The wat wrappers that only re-called the primed native (`fire-rules`,
;; `fire-rules-explain`, `insert-all`, `insert` defclause, `step-payload`) are
;; a definition seam — rust owns those names. Strip them BEFORE this rewrite
;; or freeze sees two bindings.
;;
;; Usage (one EDN vector of paths on stdin — list EVERY file with a dual-impl keyword):
;;   printf '["wat/rete/oracle/fire.wat" "tests/rete/….wat" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/rete-oracle-sigil.wat
;;
;; Idempotent by construction: after a pass the old tokens are gone.
;; Dry-run on a /tmp copy and `diff` before touching the corpus.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  ;; Innermost applies first. Oracle names, then fire-once (unprimed oracle),
  ;; then native primes drop their `'`.
  (:wat::fix::rename-keyword-exact ":wat::rete::step-payload'" ":wat::rete::step-payload"
    (:wat::fix::rename-keyword-exact ":wat::rete::arm-session'" ":wat::rete::arm-session"
      (:wat::fix::rename-keyword-exact ":wat::rete::insert'" ":wat::rete::insert"
        (:wat::fix::rename-keyword-exact ":wat::rete::insert-all'" ":wat::rete::insert-all"
          (:wat::fix::rename-keyword-exact ":wat::rete::fire-rules'" ":wat::rete::fire-rules"
            (:wat::fix::rename-keyword-exact ":wat::rete::fire-rules-explain'" ":wat::rete::fire-rules-explain"
              (:wat::fix::rename-keyword-exact ":wat::rete::fire-once'" ":wat::rete::fire-once"
                (:wat::fix::rename-keyword-exact ":wat::rete::fire-once" ":wat::rete::fire-once$oracle"
                  (:wat::fix::rename-keyword-exact ":wat::rete::insert-spec" ":wat::rete::insert$oracle"
                    (:wat::fix::rename-keyword-exact ":wat::rete::insert-all-spec" ":wat::rete::insert-all$oracle"
                      (:wat::fix::rename-keyword-exact ":wat::rete::fire-rules-spec" ":wat::rete::fire-rules$oracle"
                        src))))))))))))

(:wat::core::defn :user::apply-each
  [paths <- :wat::core::Vector<wat::core::String>] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[oracle-sigil] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

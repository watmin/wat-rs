;; wat-scripts/fixes/reclaim-hologram-find-name.wat — arc 278, the `Hologram/find'` name reclamation,
;; run over real wat source files IN WAT, through the wat CLI. Self-hosted: no Rust harness, no
;; hand-edit of wat source (use-the-tool, not hand-fix).
;;
;; The tuple-returning non-prime `:wat::holon::Hologram/find` is ANNIHILATED (commit 9410ac02) —
;; its only caller was the oracle crate `crates/wat-holon-lru`, itself annihilated at cache Stone 5
;; (83093431). So the prime reclaims the plain name — drop the trailing `'`, one boundary-aware
;; whole-name PREFIX rename:
;;   :wat::holon::Hologram/find'  ->  :wat::holon::Hologram/find
;;
;; SURGICAL: the prefix match uses the FULL old name INCLUDING the `'`, so the siblings are
;; UNTOUCHED — `:wat::holon::Hologram/get`, `/remove`, `:wat::holon::Hologram` (the TYPE), and any
;; bare `:wat::holon::Hologram/find` already present in prose-adjacent forms. The `'` is what makes
;; the match boundary unambiguous.
;;
;; Idempotent BY CONSTRUCTION: this DROPS a trailing `'` (a removal), so after the rewrite the old
;; `…find'` prefix is gone and a re-run matches nothing.
;;
;; Usage (one EDN vector of EVERY path holding the `'`-name on stdin — list them ALL):
;;   printf '["wat/cache.wat" "wat-tests/cache/HolographicLru.wat"]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/reclaim-hologram-find-name.wat
;;
;; The seams the codemod cannot touch — the Rust registration/dispatch/doc comments in
;; src/{check,runtime,hologram}.rs, and the `;;` PROSE in the wat headers that explains why a prime
;; existed at all — are the manual tail; the load-bearing wat CODE is this rewrite.

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::holon::Hologram/find'" ":wat::holon::Hologram/find"
    src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[reclaimed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

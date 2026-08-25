;; wat-scripts/fixes/rename-seq-fold-aliases-to-core-reduce.wat — arc 118.2a.
;;
;; The 118.2a NOMINA NOTA, MACHINA TACITA decisions retire the `:wat::seq::` namespace:
;; its two aliases (`:wat::seq::reduce` / `:wat::seq::fold`, both -> `:wat::core::foldl`)
;; promote to the single new clojure-surface name `:wat::core::reduce` (the proper 2/3-arity
;; reduce built over `foldl`, added this arc). Both old names collapse onto the SAME new name
;; — this is an exact whole-keyword rename (not a prefix-preserving one, since the suffix
;; ALSO changes for `fold` -> `reduce`), so two `rename-keyword-prefix` calls (each with the
;; OLD name as the full "prefix") do the job — the boundary-aware matcher (arc 283.1
;; hardening) treats a full match as the degenerate case of a prefix match.
;;
;; Usage (one EDN vector of EVERY path holding either old name on stdin):
;;   printf '["wat-tests/core/seq-fold-aliases.wat" "crates/wat-holon-lru/wat/holon/lru/HologramCacheService.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-seq-fold-aliases-to-core-reduce.wat
;;
;; Idempotent: re-running yields zero changes (the old names are gone).

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [step1 (:wat::fix::rename-keyword-prefix ":wat::seq::reduce" ":wat::core::reduce" src)
     step2 (:wat::fix::rename-keyword-prefix ":wat::seq::fold" ":wat::core::reduce" step1)]
    step2))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[renamed] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

;; wat-scripts/fixes/to-faithful-clojure.wat — faithful-Clojure corpus converter.
;;
;; Converts each file in the supplied path list from rust-scheme surface to the
;; faithful-Clojure dialect via :wat::fix::fix-text (fix.wat:339), the comment-
;; faithful text-edit codemod:
;;   - ::-namespaced call-head keyword → faithful-Clojure symbol
;;     (:wat::core::if … → (wat.core/if …)
;;   - bare annotation arrow symbol <- / -> → :-
;;   - type-shaped keyword (parametric Head<…> or tuple (…)) → list type-form
;;   - post-arrow keyword (return/param type annotation) → type-form
;;   - redundant `-> :T` return annotation on `if` → stripped
;;
;; Rides fix-text's span-edit engine: original whitespace, comments, and
;; blank lines survive byte-identical. Idempotent: re-running yields zero changes.
;;
;; BOOTSTRAP NOTE (read fix.wat lines 22-53 before running at scale): when this
;; codemod ships alongside a Rust checker change, perform the stash-dance. For a
;; pure codemod with no Rust change, just `cargo build --release` first.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat/source.wat"]\n' | cargo wat ./wat-scripts/fixes/to-faithful-clojure.wat
;;
;; Dry-run on a /tmp copy (MANDATORY before corpus sweep):
;;   cp wat/source.wat /tmp/pilot-orig.wat
;;   printf '["/tmp/pilot-orig.wat"]\n' | cargo wat ./wat-scripts/fixes/to-faithful-clojure.wat
;;   diff wat/source.wat /tmp/pilot-orig.wat

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:wat::fix::fix-text (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[to-faithful-clojure] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

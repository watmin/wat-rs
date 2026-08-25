;; wat-scripts/fixes/drop-deftest-prelude.wat — arc 278, the prelude annihilation (class-1).
;; Self-hosted fix-wat codemod: no hand-editing of .wat — wat rewrites wat.
;;
;; Drops the now-EMPTY `()` prelude slot from every deftest-family call, span-faithfully:
;;   (:wat::test::deftest'? :name () BODY)  ->  (:wat::test::deftest'? :name BODY)
;;
;; The deletion covers [prelude-start .. body-start) — the `()` AND the whitespace up to the
;; body — so the body slides cleanly up to where the prelude sat (no dangling blank line).
;; Comment/formatting elsewhere survives byte-identical (rides fix-text-apply's span-splice).
;;
;; PRECONDITION: NON-empty preludes must already be lifted to file top-level (by hand/fleet);
;; this rule fires ONLY when the prelude is `()`. A non-empty prelude is LEFT UNTOUCHED (it
;; would then break the macro flip — caught by the pre-flip grep). Idempotent: a 2-arg deftest
;; has no `()` child to drop, so a re-run is a no-op.
;;
;; HEAD-GATED to the four deftest variants ONLY. make-deftest/make-deftest-hermetic are handled
;; separately (their whole default-prelude is dropped, not left as `()`); the alias-generated
;; `(:deftest :name body)` calls are already 2-arg and never match this exact-FQDN head set.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["tests/..." ...]\n' | ./target/release/wat ./wat-scripts/fixes/drop-deftest-prelude.wat

;; deftest-head? — a List whose head keyword name is one of the four deftest variants.
(:wat::core::defn :user::deftest-head? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::empty? ch)
        false
        (:wat::core::let [head (:wat::core::first ch)]
          (:wat::core::if (:wat::core::= (:wat::core::ast-kind head) "keyword")
            (:wat::fix::str-in? (:wat::core::ast-name head)
              (:wat::core::Vector :wat::core::String
                ":wat::test::deftest"
                ":wat::test::deftest'"
                ":wat::test::deftest-hermetic"
                ":wat::test::deftest-hermetic'"))
            false))))
    false))

;; empty-list? — an empty `()` list node (the prelude slot to drop).
(:wat::core::defn :user::empty-list? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::empty? (:wat::core::ast->children node))
    false))

;; form-edits — 0-or-1 deletion edit for one top-level form: fires only on a deftest head
;; with a 4-child shape (head name prelude body) whose prelude child is `()`.
;; old-text = fix-text-span-text over the `()` prelude node's OWN span (arc 282) —
;; sanctioned: empty-list? already verified the node's identity structurally (its
;; ast->children is empty), so the deletion's subject genuinely IS the span, whatever
;; whitespace it does or doesn't contain between the parens.
(:wat::core::defn :user::form-edits
  [node  <- :wat::WatAST
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::deftest-head? node)
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::count ch) 4)
        (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
        (:wat::core::let [prelude (:wat::core::nth ch 2)]
          (:wat::core::if (:user::empty-list? prelude)
            ;; delete ONLY the `()` token span (prelude-start .. prelude-end); surrounding
            ;; whitespace + any body doc-comment between `()` and the body survive intact
            ;; (the residual blank line is wat-fmt's job — never eat a comment).
            (:wat::core::let [off      (:wat::fix::fix-text-offset-of (:wat::core::ast-span prelude) lines)
                              old-text (:wat::fix::fix-text-span-text
                                         (:wat::core::ast-span prelude)
                                         (:wat::core::ast-end-span prelude)
                                         lines src)]
              (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
                (:wat::core::Tuple off old-text "")))
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))))))
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))))

;; scan — collect edits across every top-level form (ascending offset order).
(:wat::core::defn :user::scan
  [forms <- (:wat::core::Vector :- [:wat::WatAST])
   src   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? forms)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::concat
      (:user::form-edits (:wat::core::first forms) src lines)
      (:user::scan (:wat::core::rest forms) src lines))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::scan forms src lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse all-edits))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[drop-prelude] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

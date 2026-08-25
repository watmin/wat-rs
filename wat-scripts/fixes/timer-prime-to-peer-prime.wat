;; wat-scripts/fixes/timer-prime-to-peer-prime.wat — arc 278 Stone 1 corpus migration.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; The relocation makes `(:wat::kernel::after …)` build a UNIFIED `(Peer' :- [nil O])` instead of
;; the retired tier-open `Timer'<O>`, so every `Timer'<X>` TYPE ANNOTATION in the corpus must
;; become `(Peer' :- [nil X])` (a timer has no input → I = nil; O = the delivered message type X).
;;
;; This is an INTERIOR substring rewrite (prepend a type arg), NOT a prefix rename:
;;   :wat::kernel::Timer'<X>                         → :wat::kernel::Peer'<wat::core::nil,X>
;;   :wat::core::Vector<wat::kernel::Timer'<X>>      → :wat::core::Vector<wat::kernel::Peer'<wat::core::nil,X>>
;; `rename-keyword-prefix` is boundary-aware (right-context must be a non-ident char), so it
;; CANNOT match `Timer'<` followed by an ident — hence this dedicated substring rule. The token
;; `wat::kernel::Timer'<` is unambiguous (only ever a timer annotation), so the swap is exact.
;;
;; Comment/formatting-faithful (rides fix-text-apply's span-splice) and idempotent (re-running
;; finds no `Timer'<` left → zero edits).
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat-tests/timer-after.wat" …]\n' | cargo wat ./wat-scripts/fixes/timer-prime-to-peer-prime.wat

;; ── subst-all — replace EVERY occurrence of `old` with `new` in `s` (raw, no boundary check) ──
(:wat::core::defn :user::subst-walk
  [s <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String
   old-len <- :wat::core::i64  s-len <- :wat::core::i64
   i <- :wat::core::i64  acc <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::if (:wat::core::>= i s-len)
    acc
    (:wat::core::if (:wat::core::> (:wat::core::+ i old-len) s-len)
      ;; not enough chars left to match old — emit the rest one char at a time
      (:user::subst-emit-char s old new old-len s-len i acc)
      (:wat::core::if (:wat::core::= (:wat::string::subs s i (:wat::core::+ i old-len)) old)
        ;; match — emit `new`, advance past `old`
        (:user::subst-walk s old new old-len s-len
          (:wat::core::+ i old-len) (:wat::string::concat acc new))
        ;; no match — emit one char, advance by 1
        (:user::subst-emit-char s old new old-len s-len i acc)))))

(:wat::core::defn :user::subst-emit-char
  [s <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String
   old-len <- :wat::core::i64  s-len <- :wat::core::i64
   i <- :wat::core::i64  acc <- :wat::core::String]
  -> :wat::core::String
  (:user::subst-walk s old new old-len s-len
    (:wat::core::+ i 1)
    (:wat::string::concat acc (:wat::string::subs s i (:wat::core::+ i 1)))))

(:wat::core::defn :user::subst-all
  [s <- :wat::core::String  old <- :wat::core::String  new <- :wat::core::String]
  -> :wat::core::String
  (:user::subst-walk s old new
    (:wat::string::length old) (:wat::string::length s) 0 ""))

;; ── the token rewrite: :wat::kernel::Timer'< → :wat::kernel::Peer'<wat::core::nil, ──────────
(:wat::core::defn :user::rewrite-name [name <- :wat::core::String] -> :wat::core::String
  (:user::subst-all name "wat::kernel::Timer'<" "wat::kernel::Peer'<wat::core::nil,"))

;; ── leaf-edit collection: for every keyword whose name changes, emit a whole-token edit ──────
(:wat::core::defn :user::edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
    (:wat::core::concat
      (:user::edits (:wat::core::first items) lines)
      (:user::edits-walk (:wat::core::rest items) lines))))

(:wat::core::defn :user::edits
  [node <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:user::edits-walk (:wat::core::ast->children node) lines)
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "keyword")
      (:wat::core::let [name     (:wat::core::ast-name node)
                        new-name (:user::rewrite-name name)]
        (:wat::core::if (:wat::core::= new-name name)
          (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String]))
          (:wat::core::let [off (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)]
            (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])
              (:wat::core::Tuple off name new-name)))))
      (:wat::core::Vector (:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])))))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let [lines     (:wat::string::split src "\n")
                    tree      (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None)))
                    forms     (:wat::core::ast->children tree)
                    all-edits (:user::edits-walk forms lines)
                    rev-edits (:wat::core::reverse all-edits)]
    (:wat::fix::fix-text-apply src rev-edits)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[timer->peer] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

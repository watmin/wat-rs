;; wat-scripts/fixes/unwrap-recvoutcome-false-positive.wat — RECOVERY: reverse a false-positive
;; RecvOutcome wrap.
;;
;; The wrap codemod's "Resp" matcher over-fired on matches whose scrutinee is a BARE response from
;; the OLD raw `recv` (e.g. `AdminResp`, which contains "Resp"), NOT a `recv'` client-method result.
;; Those were wrongly wrapped `(match SCRUT ((RecvOutcome::Message __recv) (match __recv IA…)) (Lost
;; __cause …) (Closed …))` — a type error (a bare response matched against RecvOutcome). This reverses
;; it back to `(match SCRUT IA…)`. Run ONLY on the confirmed false-positive files (NOT the legit
;; 119 — those also use `__recv` and MUST keep their wrap).
;;
;; Reverse (span-faithful, surgical): for a match whose FIRST arm is `((RecvOutcome::Message __recv)
;; (match __recv IA…))`, delete [scrut.end, IA1.start) → " " and delete [IAn.end, node.end-1) → "".
;;
;; Usage: printf '["pathA" …]\n' | ./target/release/wat ./wat-scripts/fixes/unwrap-recvoutcome-false-positive.wat

(:wat::core::defn :user::start-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))
(:wat::core::defn :user::end-off [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-end-span n) lines))
(:wat::core::defn :user::kw-name [n <- :wat::WatAST] -> :wat::core::String
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "keyword") (:wat::core::ast-name n) ""))

;; is-inner-match? — a node `(:wat::core::match __recv …)` (>=3 children, scrutinee symbol `__recv`).
(:wat::core::defn :user::is-inner-match? [n <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "list")
    (:wat::core::let [ch (:wat::core::ast->children n)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::match")
          (:wat::core::let [s (:wat::core::Option/expect (:wat::core::get ch 1) "s")]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind s) "symbol")
              (:wat::core::= (:wat::core::ast-name s) "__recv") false))
          false)))
    false))

;; codemod-wrapped? — outer match whose first arm (child[2]) is `((RecvOutcome::Message __recv) INNER)`
;; with INNER an is-inner-match?.
(:wat::core::defn :user::codemod-wrapped? [node <- :wat::WatAST] -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "list")
    (:wat::core::let [ch (:wat::core::ast->children node)]
      (:wat::core::if (:wat::core::< (:wat::core::length ch) 3)
        false
        (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first ch)) ":wat::core::match")
          (:wat::core::let [arm (:wat::core::Option/expect (:wat::core::get ch 2) "arm")]
            (:wat::core::if (:wat::core::= (:wat::core::ast-kind arm) "list")
              (:wat::core::let [ach (:wat::core::ast->children arm)]
                (:wat::core::if (:wat::core::< (:wat::core::length ach) 2)
                  false
                  (:wat::core::let [pat (:wat::core::first ach)
                                    body (:wat::core::Option/expect (:wat::core::get ach 1) "body")]
                    (:wat::core::if (:wat::core::= (:wat::core::ast-kind pat) "list")
                      (:wat::core::let [pch (:wat::core::ast->children pat)]
                        (:wat::core::if (:wat::core::empty? pch)
                          false
                          (:wat::core::if (:wat::core::= (:user::kw-name (:wat::core::first pch)) ":wat::kernel::RecvOutcome::Message")
                            (:user::is-inner-match? body)
                            false)))
                      false))))
              false))
          false)))
    false))

;; unwrap-edits — delete the two inserted regions, leaving (match SCRUT IA…).
(:wat::core::defn :user::unwrap-edits
  [ch <- (:wat::core::Vector :- [:wat::WatAST])  node <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [scrut  (:wat::core::Option/expect (:wat::core::get ch 1) "scrut")
     arm    (:wat::core::Option/expect (:wat::core::get ch 2) "arm")
     inner  (:wat::core::Option/expect (:wat::core::get (:wat::core::ast->children arm) 1) "inner")
     inner-ch (:wat::core::ast->children inner)
     ia1    (:wat::core::Option/expect (:wat::core::get inner-ch 2) "ia1")
     ian    (:wat::core::Option/expect (:wat::core::get inner-ch (:wat::core::- (:wat::core::length inner-ch) 1)) "ian")
     s-end  (:user::end-off scrut lines)
     ia1-s  (:user::start-off ia1 lines)
     ian-e  (:user::end-off ian lines)
     node-e (:user::end-off node lines)
     ;; both edits are gap deletions between two independently-located node boundaries
     ;; (arc 282) — sanctioned: no name-based claim about that whitespace/punctuation
     ;; exists to diverge from it; the span IS the whole belief. Sliced directly by flat
     ;; offset (both endpoints are already flat ints here; fix-text-span-text's span-map
     ;; form is unneeded — same subs-of-src semantics).
     gap1   (:wat::string::subs src s-end ia1-s)
     gap2   (:wat::string::subs src ian-e (:wat::i64::- node-e 1))]
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
      (:wat::core::Tuple s-end gap1 " ")
      (:wat::core::Tuple ian-e gap2 ""))))

(:wat::core::defn :user::node-edits
  [node <- :wat::WatAST  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::let
    [this (:wat::core::if (:user::codemod-wrapped? node)
            (:user::unwrap-edits (:wat::core::ast->children node) node src lines)
            (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))]
    (:wat::core::if (:wat::fix::structural? node)
      (:wat::core::concat this (:user::seq-edits (:wat::core::ast->children node) src lines))
      this)))

(:wat::core::defn :user::seq-edits
  [items <- (:wat::core::Vector :- [:wat::WatAST])  src <- :wat::core::String  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::foldl
    (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]) it <- :wat::WatAST]
      -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
      (:wat::core::concat acc (:user::node-edits it src lines)))
    (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
    items))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::let
    [lines (:wat::string::split src "\n")
     forms (:wat::core::ast->children (:wat::core::match (:wat::core::read-string src) ((:wat::core::ReadOutcome::Forms __forms) __forms) ((:wat::core::ReadOutcome::Malformed __cause) (:wat::kernel::assertion-failed! (:wat::core::Error/message __cause) :wat::core::None :wat::core::None))))
     eds   (:user::seq-edits forms src lines)
     rev   (:wat::core::reverse (:wat::core::sort eds))]
    (:wat::fix::fix-text-apply src rev)))

(:wat::core::defn :user::apply-each [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[unwrap-fp] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

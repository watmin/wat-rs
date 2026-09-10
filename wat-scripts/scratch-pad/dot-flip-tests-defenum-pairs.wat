;; wat-scripts/scratch-pad/dot-flip-tests-defenum-pairs.wat — arc 255 ③b-ii class ⑦ phase A.
;;
;; Text-parses every `defenum` form under `tests/` (any nesting depth — a `defenum` can sit
;; inside a `defn` body, per probe_closure_body_prelude_lift_t3/t4.wat) and composes each
;; declared variant's OLD (`::`) and NEW (`.`) spelling from the SAME structural rule
;; `wat/fix.wat`'s `rename-exact-edits-defenum-variants` already encodes for the codemod itself:
;; a defenum's children, after `defenum-variant-start` skips head/name/`:-`-params/purity/meta,
;; alternate variant-keyword / optional-fields-vector. The variant-name keyword IS the leaf; no
;; shape/case predicate is applied — every keyword found at a variant slot is emitted, whatever
;; its case (NOTE-character-case-carries-no-meaning.md).
;;
;; This is READ-ONLY: `read-string` on each file's source text, no `write-file`, no
;; `load-file!` — so a file that only fails to TYPE-CHECK (a `.wat.bad` fixture) still parses
;; fine here; only a file that fails to even READ (malformed syntax) would abort, and none did
;; in this corpus (verified: full run, zero Malformed).
;;
;; Reads a vector of paths on stdin (rooted at repo root); prints one line per declared variant:
;;   :old::spelling :new.spelling
;; Union with the existing 382-pair census (dot-flip-phase1-pairs.txt), sort, dedupe.

(:wat::core::defn :user::variant-leaf-names
  [ch <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? ch)
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::let [h (:wat::core::first ch) tl (:wat::core::rest ch)]
      (:wat::core::if (:wat::core::= (:wat::core::ast-kind h) "keyword")
        (:wat::core::if (:wat::core::if (:wat::core::not (:wat::core::empty? tl))
                            (:wat::core::= (:wat::core::ast-kind (:wat::core::first tl)) "vector")
                            false)
          (:wat::core::concat
            (:wat::core::Vector :- [:wat::core::String] (:wat::core::ast-name h))
            (:user::variant-leaf-names (:wat::core::rest tl)))
          (:wat::core::concat
            (:wat::core::Vector :- [:wat::core::String] (:wat::core::ast-name h))
            (:user::variant-leaf-names tl)))
        (:user::variant-leaf-names tl)))))

(:wat::core::defn :user::pair-line
  [parent <- :wat::core::String leaf <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [bare (:wat::string::subs leaf 1 (:wat::string::length leaf))]
    (:wat::string::concat parent "::" (:wat::string::concat bare (:wat::string::concat " " (:wat::string::concat parent (:wat::string::concat "." bare)))))))

(:wat::core::defn :user::pairs-for-leaves
  [parent <- :wat::core::String
   leaves <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? leaves)
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::concat
      (:wat::core::Vector :- [:wat::core::String] (:user::pair-line parent (:wat::core::first leaves)))
      (:user::pairs-for-leaves parent (:wat::core::rest leaves)))))

;; A defenum's NAME slot (ch[1]) is a literal keyword for a real declaration but can be an
;; `~unquote` splice inside a `defmacro` TEMPLATE (the name is computed at expand time from a
;; macro argument, e.g. `(:wat::core::defenum ~op-name :wat::enum::Pure :Go [...])`). That is
;; exactly the macro-generated bucket the design brief calls out as NOT text-parseable from
;; source — skip it here (print a marker) rather than crash on `ast-name`; it is phase C's ask
;; to resolve, not phase A's parse.
(:wat::core::defn :user::pairs-from-defenum
  [node <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [ch       (:wat::core::ast->children node)
     name-node (:wat::core::Option/expect (:wat::core::get ch 1) "defenum parent")]
    (:wat::core::if (:wat::core::= (:wat::core::ast-kind name-node) "keyword")
      (:wat::core::let
        [start  (:wat::fix::defenum-variant-start ch)
         parent (:wat::core::ast-name name-node)
         body   (:wat::core::into [] (:wat::core::drop ch start))
         leaves (:user::variant-leaf-names body)]
        (:user::pairs-for-leaves parent leaves))
      (:wat::core::do
        (:wat::kernel::println "SKIP-MACRO-TEMPLATE defenum name is not a literal keyword")
        (:wat::core::Vector :- [:wat::core::String])))))

(:wat::core::defn :user::walk-list
  [items <- (:wat::core::Vector :- [:wat::WatAST])]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::core::empty? items)
    (:wat::core::Vector :- [:wat::core::String])
    (:wat::core::concat
      (:user::walk-node (:wat::core::first items))
      (:user::walk-list (:wat::core::rest items)))))

(:wat::core::defn :user::walk-node
  [node <- :wat::WatAST]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::if (:wat::fix::calls-to? node ":wat::core::defenum")
    (:user::pairs-from-defenum node)
    (:wat::core::if (:wat::fix::structural? node)
      (:user::walk-list (:wat::core::ast->children node))
      (:wat::core::Vector :- [:wat::core::String]))))

(:wat::core::defn :user::pairs-for-file
  [path <- :wat::core::String]
  -> (:wat::core::Vector :- [:wat::core::String])
  (:wat::core::let
    [src   (:wat::io::read-file path)
     outcome (:wat::core::read-string src)]
    (:wat::core::match outcome
      [:wat::core::ReadOutcome.Forms {:forms __forms}
        (:user::walk-list (:wat::core::ast->children __forms))]
      [:wat::core::ReadOutcome.Malformed {:cause __cause}
        (:wat::core::do
          (:wat::kernel::println
            (:wat::string::concat "MALFORMED " (:wat::string::concat path (:wat::string::concat ": " (:wat::core::Error/message __cause)))))
          (:wat::core::Vector :- [:wat::core::String]))])))

(:wat::core::defn :user::print-each
  [lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? lines)
    nil
    (:wat::core::do
      (:wat::kernel::println (:wat::core::first lines))
      (:user::print-each (:wat::core::rest lines)))))

(:wat::core::defn :user::process-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::do
      (:user::print-each (:user::pairs-for-file (:wat::core::first paths)))
      (:user::process-each (:wat::core::rest paths)))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln)
    [:wat::kernel::ReadlnOutcome.Datum {:v paths} (:user::process-each paths)]
    [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
    [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")]))

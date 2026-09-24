;; wat-scripts/fixes/locus-methods-on-the-waist.wat — arc 255 Stone 255.19,
;; "per-locus behaviour lives on the waist".
;; SCOPE: corpus
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; `:wat::spawn::runner-count` and `:wat::spawn::with-label` were DEFCLAUSES keyed on the
;; concrete loci (ThreadOpts | ProcessOpts) — a per-locus list outside the `Locus` surface,
;; which a generic `(Locus :- [T])` could not narrow into, and whose `with-label` returned the
;; BARE `Locus` (erasing T). Both are now methods of the `(:wat::spawn::Locus :- [T])`
;; surface, implemented in each locus's `extend-type` (wat/spawn.wat). The defclauses are
;; RETIRED with no alias, so every caller moves to the surface call.
;;
;; THE RULES — whole-token, exact-name (never a prefix, so no sibling can move):
;;   1. a KEYWORD leaf `:wat::spawn::with-label`   → `:wat::spawn::Locus/with-label`
;;   2. a KEYWORD leaf `:wat::spawn::runner-count` → `:wat::spawn::Locus/runner-count`
;;   3. a STRING leaf  ":wat::spawn::with-label"   → ":wat::spawn::Locus/with-label"
;;      (defservice's `start`/`resume` and bracket's `map`/`each` macros pick the per-locus
;;      impl by comparing the `:locus` argument's HEAD NAME against this string; the head the
;;      user now writes is the surface call, and `ast-name` renders it in the colon form —
;;      measured in wat-scripts/scratch-pad/255-19-head-name-of-a-surface-call.wat).
;; Spec rows (the ORACLE quotes these):
;;   (:wat::spawn::with-label l r)  =>  (:wat::spawn::Locus/with-label l r)
;;   (:wat::spawn::runner-count l)  =>  (:wat::spawn::Locus/runner-count l)
;;   (= h ":wat::spawn::with-label")  =>  (= h ":wat::spawn::Locus/with-label")
;; Idempotent by construction: after the rewrite no leaf's full name equals an old name.
;;
;; ⛔ COMMENTS AND PROSE ARE NOT REWRITTEN — every edit is a leaf span splice through
;; `fix-text-apply` (each edit carries the old text it replaces and is verified before the
;; splice). Prose naming the retired defclauses is updated by hand in the same commit;
;; `docs/arc/**` is history and is not rewritten at all.
;;
;; Usage (ONE EDN vector of EVERY path on stdin):
;;   printf '[…EVERY path…]\n' | ./target/release/wat ./wat-scripts/fixes/locus-methods-on-the-waist.wat

(:wat::core::typealias :user::Edits (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :user::no-edits [] -> :user::Edits
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

;; string-edits — for a STRING leaf whose value EQUALS old, one whole-token replace edit on the
;; quoted literal (the span starts at the opening quote). Structural nodes recurse.
(:wat::core::defn :user::string-edits
  [node  <- :wat::WatAST
   old   <- :wat::core::String
   new   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :user::Edits
  (:wat::core::if (:wat::fix::structural? node)
    (:user::string-edits-walk (:wat::core::ast->children node) old new lines)
    (:wat::core::if (:wat::core::if (:wat::core::= (:wat::core::ast-kind node) "string")
                      (:wat::core::= (:wat::core::ast-name node) old)
                      false)
      (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
        (:wat::core::Tuple (:wat::fix::fix-text-offset-of (:wat::core::ast-span node) lines)
          (:wat::string::concat "\"" (:wat::string::concat old "\""))
          (:wat::string::concat "\"" (:wat::string::concat new "\""))))
      (:user::no-edits))))

(:wat::core::defn :user::string-edits-walk
  [items <- (:wat::core::Vector :- [:wat::WatAST])
   old   <- :wat::core::String
   new   <- :wat::core::String
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :user::Edits
  (:wat::core::if (:wat::core::empty? items)
    (:user::no-edits)
    (:wat::core::concat
      (:user::string-edits (:wat::core::first items) old new lines)
      (:user::string-edits-walk (:wat::core::rest items) old new lines))))

(:wat::core::defn :user::rename-string-exact
  [old <- :wat::core::String
   new <- :wat::core::String
   src <- :wat::core::String]
  -> :wat::core::String
  (:wat::core::let [lines (:wat::string::split src "\n")
                    tree  (:wat::core::match (:wat::core::read-string src) [:wat::core::ReadOutcome.Forms {:forms __forms} __forms] [:wat::core::ReadOutcome.Malformed {:cause __cause} (:wat::kernel::assertion-failed! :message (:wat::core::Error/message __cause))])
                    edits (:user::string-edits-walk (:wat::core::ast->children tree) old new lines)]
    (:wat::fix::fix-text-apply src (:wat::core::reverse edits))))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:user::rename-string-exact ":wat::spawn::with-label" ":wat::spawn::Locus/with-label"
    (:wat::fix::rename-keyword-exact ":wat::spawn::runner-count" ":wat::spawn::Locus/runner-count"
      (:wat::fix::rename-keyword-exact ":wat::spawn::with-label" ":wat::spawn::Locus/with-label"
        src))))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[locus-methods-on-the-waist] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) [:wat::kernel::ReadlnOutcome.Datum {:v __datum} __datum] [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")] [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")])))

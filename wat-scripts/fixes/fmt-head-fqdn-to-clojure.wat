;; wat-scripts/fixes/fmt-head-fqdn-to-clojure.wat — arc 277: three spellings, one seam.
;; SCOPE: wat-scripts/fmt/rules/*.wat wat-scripts/grep/*.wat
;;
;; Retarget fmt-rule string comparisons from the FQDN head spelling to the
;; clojure-target spelling that `:wat::grep::canonical-name` produces.
;;   ":wat::core::defn"  ->  "wat.core/defn"
;;
;; Only STRING nodes whose value contains `::` and starts with `:wat::` are
;; rewritten (the 33 head comparisons). Comments are not string nodes.
;; Idempotent: a second run finds no such strings.
;;
;; Usage:
;;   printf '["wat-scripts/fmt/rules/defn.wat" …]\n' \
;;     | cargo wat ./wat-scripts/fixes/fmt-head-fqdn-to-clojure.wat

(:wat::core::defn :user::empty-edits []
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]))

(:wat::core::defn :user::one-edit
  [off <- :wat::core::i64  old <- :wat::core::String  new <- :wat::core::String]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])]
    (:wat::core::Tuple off old new)))

(:wat::core::defn :user::start-off
  [n <- :wat::WatAST  lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> :wat::core::i64
  (:wat::fix::fix-text-offset-of (:wat::core::ast-span n) lines))

(:wat::core::defn :user::fqdn-string?
  [n <- :wat::WatAST]
  -> :wat::core::bool
  (:wat::core::if (:wat::core::= (:wat::core::ast-kind n) "string")
    (:wat::core::let [s (:wat::core::ast-name n)]
      (:wat::core::if (:wat::string::starts-with? s ":wat::")
        (:wat::string::contains? s "::")
        false))
    false))

(:wat::core::defn :user::string-edit
  [n     <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:user::fqdn-string? n)
    (:wat::core::let [s (:wat::core::ast-name n)
                      c (:wat::grep::canonical-name s)]
      (:wat::core::if (:wat::core::= s c)
        (:user::empty-edits)
        (:user::one-edit
          (:user::start-off n lines)
          (:wat::core::ast->source n)
          (:wat::string::concat "\"" (:wat::string::concat c "\"")))))
    (:user::empty-edits)))

(:wat::core::defn :user::node-edits
  [node  <- :wat::WatAST
   lines <- (:wat::core::Vector :- [:wat::core::String])]
  -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
  (:wat::core::if (:wat::fix::structural? node)
    (:wat::core::foldl
      (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                       c   <- :wat::WatAST]
        -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
        (:wat::core::concat acc (:user::node-edits c lines)))
      (:user::string-edit node lines)
      (:wat::core::ast->children node))
    (:user::string-edit node lines)))

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::match (:wat::core::read-string src)
    [:wat::core::ReadOutcome.Forms {:forms forms}
      (:wat::core::let
        [lines (:wat::string::split src "\n")
         kids  (:wat::core::ast->children forms)
         eds   (:wat::core::foldl
                 (:wat::core::fn [acc <- (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                                  n   <- :wat::WatAST]
                   -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::i64 :wat::core::String :wat::core::String])])
                   (:wat::core::concat acc (:user::node-edits n lines)))
                 (:user::empty-edits)
                 kids)
         rev   (:wat::core::reverse (:wat::core::sort eds))]
        (:wat::fix::fix-text-apply src rev))]
    [:wat::core::ReadOutcome.Malformed {:cause cause}
      (:wat::kernel::assertion-failed! :message (:wat::core::Error/message cause))]))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])
   n     <- :wat::core::i64]
  -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    (:wat::kernel::println
      (:wat::string::interpolate "CHANGED={n}" :n (:wat::i64::to-string n)))
    (:wat::core::let
      [path   (:wat::core::first paths)
       before (:wat::io::read-file path)
       after  (:user::migrate before)
       hit?   (:wat::core::not (:wat::core::= before after))]
      (:wat::core::do
        (:wat::core::if hit? (:wat::io::write-file path after) nil)
        (:wat::kernel::println
          (:wat::string::concat
            (:wat::core::if hit? "[changed] " "[unchanged] ")
            path))
        (:user::apply-each (:wat::core::rest paths)
          (:wat::core::if hit? (:wat::i64::+ n 1) n))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln)
      [:wat::kernel::ReadlnOutcome.Datum {:v d} d]
      [:wat::kernel::ReadlnOutcome.Eof {}
        (:wat::kernel::assertion-failed! :message "readln: end of input")]
      [:wat::kernel::ReadlnOutcome.Stopped {}
        (:wat::kernel::assertion-failed! :message "readln: stop requested")])
    0))

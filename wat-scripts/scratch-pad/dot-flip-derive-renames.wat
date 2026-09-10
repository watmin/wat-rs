;; wat-scripts/scratch-pad/dot-flip-derive-renames.wat — the dot flip's DERIVATION.
;;
;; Reads a vector of BARE namespaced names on stdin (no leading colon — that is what
;; `keyword::from-string` accepts) and prints one line per CONFIRMED variant:
;;
;;     :old::spelling :new.spelling
;;
;; ⛔ IT ASKS; IT DOES NOT MATCH. `:wat::program::PeerKind::thread` is a variant and
;; `:wat::core::Record::def` is a surface method — textually identical, `::Capitalized::lowercase`
;; both. Only `:wat::runtime::variant-parent-of` separates them, and a regex that renamed the
;; method would not fail; it would just mean something else.
;;
;; The new spelling is COMPOSED from the answer, never edited out of the old string: the parent
;; the substrate hands back IS the left half, so the dot lands where the substrate says the
;; variant boundary is rather than where a `rfind` guesses.
;;
;; Over-generation upstream is free — every distinct namespaced keyword in the corpus can be
;; offered, because the ask is the filter. That is the whole point: no predicate to be wrong about.

(:wat::core::defn :user::pair-for [bare <- :wat::core::String] -> :wat::core::String
  (:wat::core::match
    (:wat::runtime::variant-parent-of (:wat::keyword::from-string bare))
    [:wat::core::Option.Some {:value parent}
      (:wat::core::let
        [p (:wat::keyword::to-string parent)
         start (:wat::i64::+ (:wat::string::length p) 2)
         leaf (:wat::string::subs bare start (:wat::string::length bare))]
        (:wat::string::concat ":" bare " :" p "." leaf))]
    [:wat::core::Option.None {} ""]))

(:wat::core::defn :user::emit-each [names <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? names)
    nil
    (:wat::core::let [line (:user::pair-for (:wat::core::first names))]
      (:wat::core::do
        (:wat::core::if (:wat::core::= line "") nil (:wat::kernel::println line))
        (:user::emit-each (:wat::core::rest names))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::match (:wat::kernel::readln)
    [:wat::kernel::ReadlnOutcome.Datum {:v names} (:user::emit-each names)]
    [:wat::kernel::ReadlnOutcome.Eof {} (:wat::kernel::assertion-failed! :message "readln: end of input")]
    [:wat::kernel::ReadlnOutcome.Stopped {} (:wat::kernel::assertion-failed! :message "readln: stop requested")]))

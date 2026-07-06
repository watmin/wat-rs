;; Arc 293 item-2a (negative) — a STRUCT must NOT satisfy a `:nature :wat::core::Record` surface:
;; the nature bound is a hard categorical reject (Struct is non-portable, nature Struct < Record).
;; startup must Err. (After the strike the rejection is the nature bound; at HEAD it errs earlier
;; at the `:nature :wat::core::Record` parse — either way the struct never satisfies the portable surface.)
(:wat::core::defstruct :env::Stru [host <- :wat::core::String])
(:wat::core::defsurface :env::Portable :nature :wat::core::Record :features [])
(:wat::core::defn :env::take [p <- :env::Portable] -> :wat::core::i64 42)
(:wat::core::defn :env::feed-struct [] -> :wat::core::i64 (:env::take (:env::Stru "h")))

;; wat-scripts/fixes/rename-sort-prime-to-native.wat — arc 255 STONE: the last verb wearing `'`
;; as a native-impl marker adopts the `$native` convention already applied to the five
;; `:wat::rete::` firing verbs (DESIGN-STONE-sort-prime-becomes-sort-native.md).
;;
;; ⚠ RENAME, NOT ALIAS — pinned in the design. `:wat::core::sort'` stops dispatching entirely;
;; the retired spelling becomes a RETIREMENT_TABLE hit (src/remedy/retirement.rs). The public
;; `sort` / `sort-by` defclauses do not move — this only touches their INTERNAL caller.
;;
;; `rename-keyword-prefix` matches from the START of the keyword and is boundary-aware
;; (wat/fix.wat's Stone 269 vehicle). The rename table below carries the trailing `'` as part
;; of `old`, so the literal apostrophe must be present in the matched substring — this cannot
;; touch `:wat::core::sort` or `:wat::core::sort-by` (neither contains a `'`).
;;
;; Idempotent by construction: this DROPS the trailing `'` and appends `$native`, so after one
;; run no keyword begins with `:wat::core::sort'` and a re-run matches nothing.
;;
;; `fix-source`/`rename-keyword-prefix` walk the FORM tree, so `;;` comment prose is invisible
;; to this codemod — the two prose lines in wat/core.wat naming `sort'` (:1513-1514) are updated
;; by hand as prose, not by this rule.
;;
;; Usage:
;;   printf '["wat/core.wat" "wat-scripts/scratch-pad/255-probe-can-a-user-make-sort-effectful.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-sort-prime-to-native.wat

;; The migration as DATA — one row, for symmetry with the multi-row recorded migrations
;; (reclaim-ipc-prime-names.wat is the shape this mirrors).
(:wat::core::defn :user::renames [] -> (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])])
  (:wat::core::Vector :- [(:wat::core::Tuple :- [:wat::core::String :wat::core::String])]
    (:wat::core::Tuple ":wat::core::sort'" ":wat::core::sort$native")))

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::core::foldl
    (:wat::core::fn [acc <- :wat::core::String
                     pr  <- (:wat::core::Tuple :- [:wat::core::String :wat::core::String])] -> :wat::core::String
      (:wat::fix::rename-keyword-prefix (:wat::core::first pr) (:wat::core::second pr) acc))
    src
    (:user::renames)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

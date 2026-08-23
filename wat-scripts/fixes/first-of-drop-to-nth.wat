;; wat-scripts/fixes/first-of-drop-to-nth.wat — stone 118.B4-ii: the `nth` fold.
;; Self-hosted fix-wat codemod: no hand-editing of .wat — wat rewrites wat.
;;
;; B4-i widened `nth` to Seqable<T> (arc 118) so it now covers every receiver the old
;; two-verb idiom reached. This codemod folds the corpus onto the new door:
;;
;;   (:wat::core::first (:wat::core::drop X n))  ->  (:wat::core::nth X n)
;;
;; X and n carry across as their ORIGINAL SOURCE TEXT, byte for byte (span-faithful edit
;; via wat/fix.wat's `first-of-drop-to-nth` / `first-of-drop-edits` — a structural collapse,
;; not a rename, since the outer head changes AND the inner call disappears AND one paren
;; goes away). Comment-faithful; idempotent (a migrated site no longer matches the
;; `first`-headed-with-`drop`-arg shape, so a re-run is a no-op).
;;
;; ⚠ NOT semantically neutral at the edges: out of range, `(nth v i)` RAISES
;; ("nth: index out of range") where `(first (drop v i))` returned `nil` SILENTLY. Every
;; site in the recorded worklist was checked (BRIEF-STONE-118.B4-ii, STOP-1) — each is
;; guarded by a prior length/count check or draws from a grammatically-fixed-shape parsed
;; form, and none tests the extracted value for nil. That closes a tracked hole
;; (wat/seq.wat ~:597) rather than opening one; see DESIGN-STONE-118.B4.
;;
;; Worklist (the recorded census, `wat-scripts/scratch-pad/census-first-of-drop.wat` — a
;; form-tree walk, not a grep): 44 hits / 13 files / 0 malformed at c90647d4.
;;   wat/service.wat (10) · wat/lint.wat (6) · wat/fix.wat (5, rewrites ITSELF — expected,
;;   see fix.wat's own header) · wat-scripts/probes/arc-170/probe-m1-argcount.wat (5) ·
;;   wat/bracket.wat (4) · probe-s3b-extract.wat (3) · probe-s3b-astsplice.wat (3) ·
;;   probe-c1-plain-fnforms-shape.wat (3) · wat/deporder.wat (1) ·
;;   probe-m1-dump-forms.wat (1) · wat-scripts/fixes/drop-deftest-prelude.wat (1, an
;;   earlier arc's own recorded migration — this run touches its text too) ·
;;   census-parametric-surface-bindings.wat (1) · census-defclause-arm-overlap.wat (1)
;;
;; Usage (one EDN vector of EVERY path on stdin):
;;   printf '["wat/service.wat" "wat/lint.wat" …]\n' \
;;     | ./target/release/wat ./wat-scripts/fixes/first-of-drop-to-nth.wat

(:wat::core::defn :user::migrate [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::first-of-drop-to-nth src))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::core::string::concat "[first-of-drop->nth] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln )
      ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum)
      (:wat::kernel::ReadlnOutcome::Eof
        (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None))
      (:wat::kernel::ReadlnOutcome::Stopped
        (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

;; PROBE — what does brackets/map ACTUALLY do when a runner dies mid-item?
;;
;; ⭐ THIS FILE IS RUN BY THE FLOOR. Row `a_dead_runner_names_the_item_it_orphaned` in
;; `tests/probes/scratch_pad_manifest.rs` (excursus 001 `the-probes-run-in-the-floor`) runs it
;; as a subprocess and asserts exit 2, `clean=[0 2 4 6 8 10 12 14]`, and `crashed holding
;; item 3:`. ⛔ So a change here can redden the floor: run
;; `cargo nextest run --release -E 'binary_id(wat::probes)'` after editing.
;; ⚠ The row deliberately does NOT assert the RUNNER index. `a-dead-runner-names-its-orphan`'s
;; SCORE recorded `runner 0` from five uncontended runs; measured since, 12 concurrent runs
;; give 6× `runner 0` and 6× `runner 1` (the ITEM is 3 in all of them). Which worker takes the
;; item is a scheduling statistic; the item is the invariant.
;;
;; THE CLAIM UNDER TEST (a READ of wat/bracket.wat:626/632/642, not a measurement):
;;   `collect-loop`'s ServiceEvent::{Closed,Lost,Malformed} arms all `assertion-failed!`, so a
;;   dead runner fails the whole map — and the item it held is UNIDENTIFIABLE, because the loop's
;;   state is [peers items pairs-acc cursor collected m] with NO peer→item mapping. The dispatch
;;   comment says the cursor advances "regardless of outcome".
;;
;; ⛔ WHY A PROBE FIRST: this orchestrator has had SIX structural claims die on contact today, one
;;   of them a "a dying client kills the service" that a probe refuted outright. "It raises" is a
;;   reading. Raise / hang / wrong-answer are three different stones.
;;
;; MECHANISM: userland cannot kill a lineage (arc 259 S2d, `close` is kernel-restricted), so the
;; runner is made to die BY ITS OWN WORK — the work fn raises on one item. That is a real runner
;; death holding a real item, which is exactly the remote-host case the builder is preparing for.
;;
;; EXPECTED OBSERVATIONS, all printed, none raised by this probe itself:
;;   control  a clean map returns
;;   subject  a map whose item 3 kills its runner: does it raise? hang? return short?
(:wat::core::defn :bp::double [x <- :wat::core::i64] -> :wat::core::i64 (:wat::i64::* x 2))

;; The work fn raises on ONE item. The runner executing it dies holding that item.
(:wat::core::defn :bp::double-or-die [x <- :wat::core::i64] -> :wat::core::i64
  (:wat::core::if (:wat::core::= x 13)
    (:wat::kernel::assertion-failed! "runner: dying on VALUE 13, which is INDEX 3" :wat::core::None :wat::core::None)
    (:wat::i64::* x 2)))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [;; CONTROL — a clean map must return, or nothing below means anything
     ok (:wat::bracket::map (:wat::spawn::thread/runner-count 2)
          (:wat::core::range 0 8) :bp::double)
     _  (:wat::kernel::println (:wat::string::concat "clean=" (:wat::edn::write ok)))
     _  (:wat::kernel::println "subject-starting")
     ;; SUBJECT — item 3 kills its runner. Raise? Hang? Short answer?
     bad (:wat::bracket::map (:wat::spawn::thread/runner-count 2)
           (:wat::core::range 10 18) :bp::double-or-die)
     _  (:wat::kernel::println (:wat::string::concat "doomed=" (:wat::edn::write bad)))]
    nil))

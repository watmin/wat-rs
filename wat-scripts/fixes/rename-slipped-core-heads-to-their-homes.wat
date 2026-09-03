;; wat-scripts/fixes/rename-slipped-core-heads-to-their-homes.wat — arc 255 Stone 1c-0a.
;; Self-hosted fix-wat codemod: no hand-editing of .wat files — use the tool.
;;
;; DESIGN: docs/arc/2026/06/255-builtin-registry/DESIGN-STONE-1c-0a-five-call-sites-name-nothing.md
;; BRIEF:  docs/arc/2026/06/255-builtin-registry/BRIEF-STONE-1c-0a-five-call-sites-name-nothing.md
;;
;; Two of the five names this stone found are namespace SLIPS with a registered target —
;; the call site landed under `:wat::core::` when the verb actually lives elsewhere:
;;   :wat::core::println     -> :wat::kernel::println   (REGISTERED, src/intrinsic/kernel/stdio.rs)
;;   :wat::core::edn::write  -> :wat::edn::write         (REGISTERED, src/edn/render.rs)
;;
;; TWO full-name renames via `rename-keyword-prefix` (wat/fix.wat:828), each given the WHOLE
;; head as old-prefix/new-prefix — never a bare `:wat::core::` -> `:wat::` style prefix rule,
;; which would also rewrite every OTHER `:wat::core::` head in the corpus. Order is
;; irrelevant: neither old-bare is a substring of the other ("core::println" vs.
;; "core::edn::write"), so the two rewrites are disjoint by construction, unlike the
;; SPECIFIC-FIRST ordering deprime-telemetry-sqlite.wat needed for its primed/family pair.
;;
;; Comment-faithful and idempotent by construction (rename-keyword-prefix's boundary rule):
;; after applying, the old prefixes are gone, so a second run produces zero edits.
;;
;; Usage (one EDN vector of paths on stdin):
;;   printf '["wat-scripts/scratch-pad/probe-stone-2a-bracket-mechanics.wat" "wat-scripts/scratch-pad/t-bare.wat" "wat-scripts/probes/arc-170/probe-process-only.wat" "wat-scripts/probes/arc-170/probe-edn.wat"]\n' \
;;     | cargo wat ./wat-scripts/fixes/rename-slipped-core-heads-to-their-homes.wat

(:wat::core::defn :user::migrate
  [src <- :wat::core::String] -> :wat::core::String
  (:wat::fix::rename-keyword-prefix ":wat::core::edn::write" ":wat::edn::write"
    (:wat::fix::rename-keyword-prefix ":wat::core::println" ":wat::kernel::println"
      src)))

(:wat::core::defn :user::apply-each
  [paths <- (:wat::core::Vector :- [:wat::core::String])] -> :wat::core::nil
  (:wat::core::if (:wat::core::empty? paths)
    nil
    (:wat::core::let [path (:wat::core::first paths)]
      (:wat::core::do
        (:wat::io::write-file path
          (:user::migrate (:wat::io::read-file path)))
        (:wat::kernel::println (:wat::string::concat "[renamed-slipped-core-heads] " path))
        (:user::apply-each (:wat::core::rest paths))))))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:user::apply-each
    (:wat::core::match (:wat::kernel::readln ) ((:wat::kernel::ReadlnOutcome::Datum __datum) __datum) (:wat::kernel::ReadlnOutcome::Eof (:wat::kernel::assertion-failed! "readln: end of input" :wat::core::None :wat::core::None)) (:wat::kernel::ReadlnOutcome::Stopped (:wat::kernel::assertion-failed! "readln: stop requested" :wat::core::None :wat::core::None)))))

;; wat-scripts/scratch-pad/277-lint-recount.wat — arc 277: report `(:wat::lint::lint-stdlib)`
;; findings after the sweep-lint-fixes.wat auto-fix sweep, one line per finding
;; (rule | severity | file:line), so the shell can tally rule counts with sort|uniq -c.
;; Scratch, per holon/CLAUDE.md's `.wat` scratch convention (not the ephemeral session tmp).

(:wat::core::defn :user::print-finding
  [f <- :wat::lint::Finding] -> :wat::core::i64
  (:wat::core::do
    (:wat::kernel::println
      (:wat::string::interpolate "{rule} | {sev} | {file}:{line}"
        :rule (:wat::lint::Finding/rule f)
        :sev  (:wat::lint::Finding/severity f)
        :file (:wat::lint::Finding/file f)
        :line (:wat::i64::to-string (:wat::lint::Finding/line f))))
    0))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let [findings (:wat::lint::lint-stdlib)]
    (:wat::core::do
      (:wat::kernel::println
        (:wat::string::interpolate "TOTAL {n}" :n (:wat::i64::to-string (:wat::core::length findings))))
      (:wat::core::foldl
        (:wat::core::fn [acc <- :wat::core::i64 f <- :wat::lint::Finding] -> :wat::core::i64
          (:user::print-finding f))
        0
        findings)
      nil)))

;; Excursus 003 step 4 (D4) — G0, the reserved-stdlib-label wall (builder ruling).
;;
;; This entry program loads a file at the EXACT same label as the real baked stdlib's
;; own `wat/core.wat` — served here by an `InMemoryLoader` the driver seeds with
;; DIFFERENT content, so the collision is unambiguous: whatever answers to this label
;; is NOT the real stdlib, and must be refused rather than silently accepted as one.
(:wat::load-file! "wat/core.wat")

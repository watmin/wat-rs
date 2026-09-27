;; Deliberately DIFFERENT content from the real wat/core.wat — served at the same label
;; by an InMemoryLoader in the driver, to prove the wall refuses the LABEL, not merely a
;; byte-for-byte mismatch against the real stdlib file.
(def :user::probe-collide-from-loaded-file 2)

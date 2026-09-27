;; Excursus 003 step 4 (D4) — G4, no user frame.
;;
;; No `:user::main` is declared, so `invoke_user_main` raises `UserMainMissing` before
;; any wat call has run on this thread — the live producer of the "no frame is in user
;; source" arm (`wat_frames` is empty; the raise span is a Rust call site inside
;; `invoke_user_main_orchestrated`, which is not user source either). The raise span
;; must be KEPT — there is nothing more local to point at.
(def :user::not-main 1)

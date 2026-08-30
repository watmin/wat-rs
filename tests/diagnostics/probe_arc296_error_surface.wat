;; tests/diagnostics/probe_arc296_error_surface.wat — co-located fixture
;;
;; Arc 296 — :wat::core::Error stdlib surface.
;;
;; RED at HEAD: :wat::core::Error is unknown → startup fails when
;; :probe::BadInput references it as a field type and when :probe::describe
;; references it as a param type.
;;
;; GREEN after wat/core.wat adds:
;;   (:wat::core::defsurface :wat::core::Error
;;     :nature :wat::core::Record
;;     :features [message  <- :wat::core::String
;;                location <- :wat::kernel::Location
;;                causes   <- (:wat::core::Vector :- [wat::core::Error])])
;;
;; Three things proved:
;; (a) startup boots — :wat::core::Error is registered
;; (b) :probe::BadInput structurally satisfies :wat::core::Error as a
;;     [e <- :wat::core::Error] param; field accessed via :wat::core::Error/message
;; (c) edn::write → edn::read round-trips the record without error

(:wat::core::defrecord :probe::BadInput
  [message  <- :wat::core::String
   location <- :wat::kernel::Location
   causes   <- (:wat::core::Vector :- [:wat::core::Error])
   field    <- :wat::core::String])

;; Surface-typed param: the checker verifies :probe::BadInput satisfies
;; :wat::core::Error structurally (it has message, location, causes).
;; Body: arc 293.4d field accessor — :S/field-name on a surface-typed receiver.
(:wat::core::defn :probe::describe [e <- :wat::core::Error] -> :wat::core::String
  (:wat::core::Error/message e))

(:wat::core::defn :user::main [] -> :wat::core::nil
  (:wat::core::let
    [e    (:probe::BadInput :message "port must be > 0" :location (:wat::kernel::here)
             :causes (:wat::core::Vector :- [:wat::core::Error]) :field "port")
     msg  (:probe::describe e)
     s    (:wat::edn::write e)
     back (:wat::edn::read s)]
    (:wat::test::assert-eq msg "port must be > 0")))

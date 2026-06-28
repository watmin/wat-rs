;; Contract 04: defenum with :variant-metadata.
(:wat::core::defenum :app::Status
  {:variant-metadata {:Error {:doc "raised when the operation fails"}}}
  :Ok
  :Pending
  :Error)

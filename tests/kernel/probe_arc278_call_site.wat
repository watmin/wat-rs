;; Co-located fixture for probe_arc278_call_site.rs — call_site_returns_caller_frame.
;; Arc 278 "caller.1" — `(:wat::kernel::call-site)`: a native nullary verb returning the
;; caller's `:wat::kernel::Frame {file, line, symbol}` — the wat equivalent of Ruby's
;; `caller` / Rust's `Location::caller()`.
;;
;; :probe::here's ENTIRE body is `(:wat::kernel::call-site)`. A native verb pushes no wat
;; frame of its own (only wat fn-calls push, via FrameGuard), so the top of the wat call
;; stack at that point is the frame pushed for the call TO :probe::here — i.e. the CALLER's
;; site: where the deftest' body below wrote `(:probe::here)`.
;;
;; RED at HEAD: :wat::kernel::call-site is unknown to the type checker → startup fails.
;; GREEN after: startup succeeds; the returned Frame's file/line/symbol are all Some and
;; describe the caller (this file, a positive line, and the "probe::here" symbol).

(:wat::core::defn :probe::here [] -> :wat::kernel::Frame
  (:wat::kernel::call-site))

(:wat::test::deftest' :user::call-site-returns-caller-frame ()
  (:wat::core::let
    [frame     (:probe::here)
     file      (:wat::kernel::Frame/file frame)
     line      (:wat::kernel::Frame/line frame)
     symbol    (:wat::kernel::Frame/symbol frame)
     file-ok   (:wat::core::match file -> :wat::core::bool
                 ((:wat::core::Some f) (:wat::core::string::contains? f "probe_arc278_call_site"))
                 (:wat::core::None     false))
     line-ok   (:wat::core::match line -> :wat::core::bool
                 ((:wat::core::Some l) (:wat::core::> l 0))
                 (:wat::core::None     false))
     symbol-ok (:wat::core::match symbol -> :wat::core::bool
                 ((:wat::core::Some s) (:wat::core::string::contains? s "probe::here"))
                 (:wat::core::None     false))]
    (:wat::core::do
      (:wat::test::assert-true file-ok)
      (:wat::test::assert-true line-ok)
      (:wat::test::assert-true symbol-ok))))

//! Sandbox loader resolution.
//!
//! Pre-arc-105 this module hosted `eval_kernel_run_sandboxed` /
//! `eval_kernel_run_sandboxed_ast` — substrate Rust impls that
//! collected stdio as `Vec<String>` buffers. Arc 105c retired
//! both. The wat-level reimplementation in `wat/kernel/sandbox.wat`
//! (atop arc 105a's spawn-program-returns-Result and arc 105b's
//! `:wat::kernel::ThreadDiedError/message` accessor) is now
//! canonical; `Vec<String>` survives only inside that wat-level
//! helper where it's the test-assertion target.
//!
//! `resolve_sandbox_loader` (and the corresponding `src/spawn.rs` callers)
//! were retired in arc 298. The module remains as a namespace anchor.

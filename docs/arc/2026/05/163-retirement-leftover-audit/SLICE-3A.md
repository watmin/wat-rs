# Arc 163 Slice 3a — Service paths verification

**Verified 2026-05-07 by orchestrator.** Audit confirmed all 5
service-path retirement surfaces are HARD-retired. Zero source
edits needed.

## Surfaces verified

| Surface | Walker fn | Variant | Live keyword usage | Runtime arm | Status |
|---|---|---|---|---|---|
| `:wat::std::service::Console::*` (arc 109 K.console) | `validate_legacy_console_service_path` | `BareLegacyConsolePath` | 0 (14 sites all Bucket C/D — diagnostic / walker / doc) | none | HARD ✓ |
| `:wat::std::service::Telemetry::*` (arc 109 K.telemetry) | `validate_legacy_telemetry_service_path` | `BareLegacyTelemetryServicePath` | 0 sites | none | HARD ✓ |
| `:wat::std::service::LruCache::*` (arc 109 K.lru) | `validate_legacy_lru_cache_service_path` | `BareLegacyLruCacheServicePath` | 0 sites | none | HARD ✓ |
| `:wat::std::stream::*` (arc 109 slice 9d) | `validate_legacy_stream_path` | `BareLegacyStreamPath` | 0 (6 sites all Bucket C/D — `LEGACY_STREAM_PREFIX` const + walker + diagnostic) | none | HARD ✓ |
| `:wat::kernel::Queue*` (arc 109 K.kernel-channel) | `validate_legacy_kernel_queue_path` | `BareLegacyKernelQueuePath` | 0 (14 sites all Bucket C/D — `LEGACY_KERNEL_QUEUE_NAMES` const + walker + diagnostic + dead-code scope-deadlock comment) | none | HARD ✓ |

## Audit method (per surface)

1. Confirm walker fn exists in `src/check.rs` — emits Pattern 2
   poison with redirect-to-canonical hint
2. Confirm Display impl present (variant has user-facing message)
3. `grep -rEn "<legacy_prefix>" --include="*.rs" --include="*.wat" .`
   excluding `complected/` archive — sample residuals to classify
4. Confirm no `":wat::std::*"` runtime alias arm in `src/runtime.rs`

## Verdict

**5 surfaces hard-retired; no slice 3a source work required.** The
sweep done earlier (slice 2 cleared consumer wat-scripts/ leftovers
for stream + queue family) plus slice 1 (let* sweep) plus arc 162
(lambda sweep) covered consumer code. The remaining residual in
each is exactly the Bucket C/D scaffolding (variant + Display +
walker fn name + LEGACY_*_PREFIX const + retirement comment) per
arc 113 precedent — preserved by design.

## Next slice

Slice 3b — Unit name + type retirement verification
(`BareLegacyUnitName`, `BareLegacyUnitType`). Same pattern.

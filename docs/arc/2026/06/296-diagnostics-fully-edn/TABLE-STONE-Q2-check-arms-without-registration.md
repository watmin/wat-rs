# TABLE — STONE Q2: check.rs arms with no intrinsic registration

Committed before the registration. The brief sampled four of 143 and asked
whether `subtype?` is an omission or 255's special-forms bucket.

Instrument: every `":wat::…" =>` match arm in `src/check.rs`, unique FQDN,
against every `#[wat_intrinsic("…")]` / `#[wat_special_form("…")]` under `src/`.

```
arm occurrences          144
unique arm FQDNs         133
registered (attr)        577   (whole src/, not just arms)
unique arms NOT registered  23
```

23 is SMALL. Not 143. Not the special-forms bucket. `subtype?` is an omission;
the other 22 are a short list, not this stone.

## The 23

| FQDN | kind | this stone? |
|---|---|---|
| `:wat::core::subtype?` | type-level predicate, twin of registered `conforms?` | **yes** |
| `:wat::core::defn` | declaration macro | no |
| `:wat::core::defn-restricted` | declaration | no |
| `:wat::core::def-restricted` | declaration | no |
| `:wat::core::define` | declaration | no |
| `:wat::core::define-dispatch` | declaration | no |
| `:wat::core::enum` | declaration | no |
| `:wat::core::struct` | declaration | no |
| `:wat::core::struct-restricted` | declaration | no |
| `:wat::runtime::define-alias` | declaration | no |
| `:wat::core::vec` | retired constructor spelling (`Vector` is registered) | no |
| `:wat::core::list` | retired constructor spelling (`List` is registered) | no |
| `:wat::core::tuple` | retired constructor spelling (`Tuple` is registered) | no |
| `:wat::core::concat` | collection op, pre-registry dispatch | no |
| `:wat::core::foldr` | collection op, pre-registry dispatch | no |
| `:wat::core::reduce` | collection op, pre-registry dispatch | no |
| `:wat::core::seqable->stream` | collection op, pre-registry dispatch | no |
| `:wat::core::try` | alias of registered `Option/try` / `Result/try` | no |
| `:wat::core::option::expect` | alias of registered `Option/expect` | no |
| `:wat::core::result::expect` | alias of registered `Result/expect` | no |
| `:wat::kernel::Process/join-result` | kernel | no |
| `:wat::kernel::Process/stdin` | kernel | no |
| `:wat::program::self-peer` | kernel | no |

`if` / `let` / `fn` / `match` / `def` / `quote` / `conforms?` already have a
registration. They are in the 133, not the 23.

## Verdict

Register `subtype?`. Report the 22. Do not register 143 rows.

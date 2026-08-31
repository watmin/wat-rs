# HANDOFF → grok — excursus 001 stone WO-OPT: the opts arg becomes OPTIONAL

Same branch, `sns-sqs`. Read in full:

- `docs/excursus/2026/08/001-sns-sqs/BRIEF-stone-write-opts-optional.md`
- `docs/excursus/2026/08/001-sns-sqs/EXPECTATIONS-stone-write-opts-optional.md`

**This corrects MY brief, not your work.** WRITE-OPTS shipped `write-json` at a required arity
of 2 because my sketch showed `(:wat::edn::write-json v (:wat::edn::opts))` everywhere. The
builder's intent was: *"if you omit it, you get the defaults; if you want to change it, you pass
the config ops you want for your call."* You built what I wrote; the specification was wrong.

`(:wat::edn::write-json v)` must type-check and mean the default.

**The exemplar is exact — `:wat::io::IOReader/read-frame`**, which already accepts 1 or 2 args:
a Variadic handler (`src/intrinsic/io/reader.rs:410`), the arity guard in a named `infer_` fn in
the checker (`src/check.rs:9281`), and a dispatch arm that intercepts it (`src/check.rs:2977`).
`reader.rs:80` calls it out as the one exception of ten — do the same for the JSON verbs.

⚠ **Do NOT add a `Range` arity to the intrinsic registry.** `src/intrinsic/mod.rs:142` says
Range/AtLeast are deliberately out of scope. That is a registry-shape change and it belongs to
arc 255. If Variadic-plus-checker-guard is unworkable, STOP and report.

★ **Row 2 is the real gate:** `(write-json v)` and `(write-json v (:wat::edn::opts))` must
produce **byte-identical** output. "1-arg works" would pass with any default.

`:wat::edn::write` / `write-pretty` stay `Exact(1)` — sort-key path, unchanged. `wat/edn.wat`
and `crates/wat-edn/` are untouched.

`.contains(` on a deterministic string trips `no_loose_string_assert` — it has caught two stones
in this excursus already. Use `assert_eq!` on the whole string from the start.

Verify in the FOREGROUND; read the Summary line. Floor is **5113 with ONE known failure**
(the journal key-collision arm) — expected, not yours.

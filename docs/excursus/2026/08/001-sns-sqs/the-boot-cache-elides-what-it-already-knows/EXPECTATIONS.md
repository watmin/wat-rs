# EXPECTATIONS — the boot cache elides what it already knows (Tier A)

| # | what | how it is judged |
|---|---|---|
| 1 | ⭑⭑ **Boot drops, measured with the census** | before/after, **cold AND warm**, via `WAT_BOOT_CENSUS=phases`. Target is the 404.9 → ~154.7 ms shape; report what you actually got, including the load cost of building the real spine. |
| 2 | ⭑⭑ **Spans survive** | byte-exact fixpoint `encode(decode(encode(x))) == encode(x)`. ⛔ `assert_eq!` is worthless here — `Span::eq` returns `true` unconditionally. Say which test you used and why. |
| 3 | ⭑⭑ **Behaviour identical** | circuit byte-identical: `distinct=8000;dup=0` **and** `timeout=yes;discarded=yes;redial=Connected;retry-on=fresh`. |
| 4 | ⭑⭑ **Staleness is DETECTED** | driven: change a stdlib file, show the cache is rejected and boot still produces correct results. Report detection cost. |
| 5 | ⭑⭑ **Absent / corrupt cache falls back silently and correctly** | driven: delete it, truncate it, corrupt a byte. Boot must work in all three. |
| 6 | ⭑ **Natives are re-registered, not serialised** | confirm none are in the payload. |
| 7 | ⭑ **`ScopeId` is re-minted and sharing is preserved** | the remap, driven. |
| 8 | ⭑⭑ **Floor** | Summary verbatim, `.floor/<stamp>/`, `ARM.txt` or not. |
| 9 | clippy + `--no-run` | `--release --workspace --all-targets`. |
| 10 | ⭑ **Build-time vs first-run: which, and what the other would cost** | the decision and its reasoning. |
| 11 | ⛔ **Tier B untouched; `.config/nextest.toml` untouched** | `git diff` and a sha256. |
| 12 | ⭑ **What got slower** | any cost the cache adds — build time, binary size, first run. Report it; a win stated without its bill is half a measurement. |

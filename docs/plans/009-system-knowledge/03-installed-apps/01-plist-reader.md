# 009/03/01 — The `Info.plist` reader

**Status:** ✔️ done · **Version:** 0.5.5 · **Date:** 2026-11-16 · **Modules:** system

## Files

- `crates/system/src/plist/{mod,xml,binary}.rs` (new) — `read_strings(bytes) -> Result<BTreeMap<String, String>,
  PlistError>`: the string values of the top-level dictionary of an XML or binary (`bplist00`) property list; other
  values skipped; at most 1 MiB; bounded depth and object counts.

## Checks

- Negative (first): empty input, a dictionary that is not at the top, a truncated binary trailer, an offset outside
  the data, a reference cycle, an XML with an unclosed tag, more than 1 MiB; property tests: no input panics.
- Positive: XML with entities, nested dictionaries and arrays skipped; binary with ASCII and UTF-16 strings and
  integers of every width; both forms of the same file give the same map.

## Conclusions

- Built as written by a subagent in a scratch crate (the lints and `rustfmt.toml` of the workspace), then moved into
  `crates/system/src/plist/` (`mod`, `xml`, `binary`, `tests` with a test-only binary writer); committed with 03/03,
  its first user, so no step leaves dead code behind.
- Rules: only string values of the top-level dictionary; any other value or a non-string key drops its entry; a
  string that cannot be decoded (non-ASCII bytes in a binary ASCII string, a lone UTF-16 surrogate) drops its entry
  rather than failing the read; the last of a repeated key wins; skipped XML values at most 64 levels deep; `<plist>`
  is optional; CDATA is read; `&#0;`, surrogates and code points past U+10FFFF are malformed.
- Checked against the system's own writer: a property list written by hand and converted to the binary form with
  `plutil` is a hex fixture (`reads_what_the_system_writer_produced`). 50 tests, 99.7 % of its lines.

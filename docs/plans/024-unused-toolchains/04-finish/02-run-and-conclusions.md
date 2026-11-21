# 024/04/02 — The runs by hand and the conclusions

**Status:** 🔨 in progress · **Version:** 0.1.0 · **Date:** 2026-11-21 · **Modules:** docs, modules

A checklist for the user, then the conclusions.

## In the sandbox

- [ ] `rm -rf sandbox/unused-toolchains && npm run dev:sandbox -- unused-toolchains`. Scan; walk `EXPECTED.md` line by
  line, negatives first: nothing a pointer keeps is offered (rustup's default and override, the moving channels, the
  toolchains pinned in `Documents/dev/pinned`, `Code/legacy` and `Code/asdf`, the link, `my-custom`, nvm's default and
  newest, the alias chain, the `.nvmrc` in `Documents/dev/web`, the `.node-version`); the socket case reads as kept with
  its reason; each positive is offered in its category at its danger (Rust Low, Node Medium); titles read `Rust
  1.70.0-aarch64-apple-darwin`, `Node v12.22.12`; each reason names a date and the command.
- [ ] Settings: the Node versions switch is on (the case file's settings). Turn it off, scan again: no Node version is
  offered, the toolchains still are. Turn it back on.
- [ ] The storage map: the offered versions in the explained part; the pins' folders still open in your folders.
- [ ] Select everything; Review names Medium; confirm to the Trash: the version folders land in `home/.Trash`;
  `settings.toml`, the alias files and every pin untouched. Cleanups → put back: the folders are back.
- [ ] `npm run dev:sandbox -- unused-toolchains-unknown`: the toolchain is kept "could not read which version is the
  default"; the Node version is kept "a version file could not be read".
- [ ] `npm run dev:sandbox -- unused-toolchains-off`: the switch is off in Settings; no Node version; the toolchain goes.
- [ ] `npm run dev:sandbox -- unused-toolchains-documents-denied`: the module's two categories on Scan say they need
  Documents (the "Let Fresher look there" button on their Details screen); nothing is offered.
- [ ] Ignore the Rust category (from its card's menu on Scan), then the module; restore both in Settings, Ignored.

## On the developer's Mac (a dry run)

`npm run dev`, a scan of `~` with Full Disk Access (a development build on the real Mac makes every cleanup a dry
run); turn the Node switch on and scan again. Compare with these read-only commands:

```sh
rustup toolchain list; cat ~/.rustup/settings.toml
ls ~/.rustup/toolchains ~/.nvm/versions/node 2>/dev/null; ls -R ~/.nvm/alias 2>/dev/null
du -sh ~/.rustup/toolchains/* ~/.nvm/versions/node/* 2>/dev/null
find ~ -xdev \( -name rust-toolchain.toml -o -name rust-toolchain -o -name .nvmrc -o -name .node-version \
  -o -name .tool-versions \) -not -path '*/node_modules/*' -not -path '*/Library/*' 2>/dev/null
find ~ -xdev \( -name .nvmrc -o -name .node-version \) -type l 2>/dev/null
```

- [ ] Every offered version checked by hand: not the default, not an override or alias, not named by any pin the
  `find` lists; its size against `du`.
- [ ] Nothing offered outside the two places; no moving channel, no newest Node version.
- [ ] The facts "to verify":
  - whether the walk is complete on this Mac (`is_walk_complete()`): if every version is left out `partial-scan`, the
    numbers and the follow-up are written here and a deferred item is opened;
  - a removed toolchain comes back with `rustup toolchain install <name>` while `update-hashes/<name>` remains (in a
    throwaway `RUSTUP_HOME`, never the real one);
  - linked pins (the second `find`): how many, and whether they block nvm's offers;
  - no scan timing is due: the walk and the scanner are unchanged.
  The table, texts and category docs are corrected where they differ.

## Files

- `docs/plans/024-unused-toolchains/readme.md`, `00-design.md` and every plan's readme — the conclusions: what was
  built, what changed from the design and why, the numbers (tests, coverage, what the module offered
  on the developer's Mac by category, how many files it read), the calls taken while building, what the real Mac
  showed.
- `modules/unused-toolchains/toolchains.toml`, `locales/en.json`, `docs/categories/unused-toolchains-*.md` — corrected
  where the real Mac differs.
- `docs/plans/README.md` — the row ✔️ done once the user has looked at the result.

## Checks

- `npm run pre-push` on the branch's last commit; the user has looked at the result.

## Conclusions

The agent's part is done: the residual long lines rewrapped, then the checks below on the branch's last code
commit. Every box above is the user's and is **pending**: the sandbox walk, the dry run on the real Mac and the facts
"to verify". The scan-time check is not part of this plan: the user's rule of 2026-11-21 (the walk and the scanner are
unchanged), so no time is due and no deferred item was opened for it. The plan stays 🔨 in progress until the user has
looked at the result. The conclusions of the whole plan are in the [global readme](../readme.md#conclusions).

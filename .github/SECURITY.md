# Security policy

## Supported versions

The latest release, 0.1.x, and the latest `main`. A fix lands on `main` and ships in the next release.

## What counts

Anything that:

- removes a file outside a confirmed cleanup;
- reaches outside the scanned folders or into a protected place;
- follows a link out of a scanned folder;
- sends anything over the network;
- asks for more access than it needs.

The rules are in [`deletion-safety.md`](../docs/restrictions/deletion-safety.md) and
[`privacy.md`](../docs/restrictions/privacy.md).

## How to report

Privately, through the repository's Security tab, "Report a vulnerability":
<https://github.com/AlexSergey/fresher/security/advisories/new>. Do not open a public issue.

Include the macOS version, Fresher's version (Settings → About) or the commit, the steps, and what was removed or
reached.

You get an answer, a fix on `main` and in the next release, and credit if you want it.

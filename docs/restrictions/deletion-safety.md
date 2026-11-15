# Deletion safety

The app deletes files from a real person's disk. These rules hold in every version.

## Explicit confirmation only

- **Rule:** nothing is deleted unless the user confirmed that exact set of items in the current session, after seeing
  their total size and the highest danger level among them. No background, scheduled or automatic deletion.
- **Why:** the user asked for it explicitly; a cleaner that deletes on its own cannot be trusted.
- **Enforced by:** the cleaner accepts only a `CleanPlan` built from a confirmation (planned for the cleaner's plan);
  review until then.

## Scanning is read-only

- **Rule:** the scanner and the rules never write to, move or delete anything in a scanned root.
- **Why:** a scan must be safe to run at any time.
- **Enforced by:** Clippy `disallowed-methods` forbids deleting `std::fs` functions outside `fresher-cleaner` (plan
  001); the crate layout.

## Trash first

- **Rule:** items are moved to the Trash by default; a permanent delete is a separate, explicit choice per confirmation.
- **Why:** a mistake must be recoverable.
- **Enforced by:** the cleaner's API (planned).

## Re-check before delete

- **Rule:** right before an item is deleted, the cleaner checks it is still what was confirmed (same path, same kind, not
  replaced by a symlink, size not grown past the confirmed one by a large margin); otherwise it skips the item and
  reports it.
- **Why:** the disk can change between the scan and the confirmation.
- **Enforced by:** the cleaner (planned).

## Never follow symlinks out, never touch protected paths

- **Rule:** the scanner and the cleaner follow no links — neither symlinks nor Finder aliases — so a link can never
  point them somewhere unexpected; nothing under `/System`, `/usr` (except `/usr/local`), `/bin`, `/sbin`,
  `/private/var/db`, SIP-protected locations, or another user's home is ever offered for deletion. Docker objects are
  removed only through the Docker CLI, never by deleting its files.
- **Why:** deleting there breaks macOS or another user's data; Docker's files are its own database.
- **Enforced by:** the scanner's and the cleaner's guards and their tests (planned).

## The user's own folders are hard-blocked

- **Rule:** the home folder and its standard folders — Desktop, Documents, Downloads, Movies, Music, Pictures, Public,
  Library — can never be removed as a whole, whatever a rule or a selection says. Items inside them can be, one by one.
- **Why:** a broken rule or a wrong selection must not be able to wipe a person's files; the guard sits below every
  rule.
- **Enforced by:** the cleaner's guard and its tests (planned).

## Items in use are skipped, never forced

- **Rule:** a file open by a running process, or a folder owned by a running app or service (a database, a container
  runtime), is skipped and reported with the reason; the cleaner never forces a removal and never quits an app on its
  own.
- **Why:** removing files under a running program corrupts its data or crashes it.
- **Enforced by:** the cleaner's in-use check and its tests (planned).

## Unknown is not junk

- **Rule:** a rule claims a path only when it can tell what it is. Unrecognized space is shown, never offered as junk.
- **Why:** rule 1 of [`../rules.md`](../rules.md): a false positive is worse than a missed gigabyte.
- **Enforced by:** every rule's negative tests.

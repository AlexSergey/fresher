# Threat model

Who could hurt the user through Fresher, what they control and what they want. Written for the audit of plan
[041](../plans/041-security-audit/readme.md) and completed by what it found. The rules that answer each threat are in
[`../restrictions/`](../restrictions/README.md).

## What is worth taking

- **The power to delete.** Fresher moves to the Trash, or deletes, what the user confirmed. Anyone who steers that
  removes the user's files with the user's own click.
- **Full Disk Access.** The user may grant it to Fresher alone. Any code that runs inside Fresher, or as a program it
  starts, inherits it.
- **What the scan sees.** Fresher reads the whole home folder; nothing of it may leave the Mac.

## Who

| Who                                                                   | Controls                                                                                         | Wants                                                                     |
| --------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| Another program of the same user                                      | files, links and folder names under the home, `/Applications` (an administrator's), timing      | the app deleting or moving something else, or reading for it              |
| The same program, aiming at Fresher's access                          | a look-alike app bundle, the `PATH` of the session, a library injected at launch                | its own code running under Fresher's Full Disk Access                     |
| The user's own configuration of a tool Fresher runs                   | the Docker CLI's contexts, credential helpers, hooks, plugins; the ssh config they reach         | the tool connecting elsewhere or starting another program                 |
| A hostile file tree (a cloned project, an archive, a mounted volume)   | names, links, depth, sizes, the content the modules parse (property lists, manifests, pins)      | a wrong claim, a crash, a stalled scan, a walk out of the root            |
| A tampered data folder                                                | the app's settings, scan cache and cleanup journal (plain files of the user)                     | a put back or a cleanup aimed at a path of its choice                     |
| Another local account                                                 | shared folders such as `/tmp`                                                                    | Fresher refusing to start                                                 |
| A page that is not Fresher's own                                      | what the webview loads (a development server, a navigation, a form)                              | the core's commands, a confirmation the user never gave                   |
| A dependency or the build                                             | code inside the app, the lock files, the CI's tools                                              | the network, the disk beyond what it is given, a tampered release         |
| A bug of our own                                                      | nothing: it is already inside                                                                    | —                                                                         |

## Out of scope

- The security of macOS itself, of the tools Fresher reads or runs (beyond how it runs them), and of the user's Mac.
- A program running as root or as an administrator acting through `sudo`: it can already do what it wants.
- A program that can change a genuine, signed app bundle: macOS's App Management guards that.

## Severity

| Level    | Meaning                                                                                                              |
| -------- | -------------------------------------------------------------------------------------------------------------------- |
| Critical | anyone can make Fresher delete or move the user's files, or send data off the Mac, with no click of the user         |
| High     | a same-user program gets code run under Fresher's access, or a file moved where the user did not choose, in one click |
| Medium   | a denial of service (memory, time, a refused start), or a protection crossed only in a narrow case                   |
| Low      | a hardening gap, a wrong claim in the docs, or an attack that needs an existing race or an unlikely setup            |
| Info     | behaviour worth writing down, not a defect                                                                          |

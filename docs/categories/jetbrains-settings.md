# Old JetBrains IDE settings

**Danger level:** Medium · **Rule:** none (the module's code, `modules/jetbrains`) · **Since:** 0.1.0

**Example:** `~/Library/Application Support/JetBrains/<IDE><older version>`

## What it is

The settings (`options/`, keymaps, code styles, live templates) and the `plugins/` an older version of a JetBrains
IDE (IntelliJ IDEA, WebStorm, PyCharm and the others) used. Each version keeps its own folder, and an update leaves
the older one behind.

## Where it lives

`~/Library/Application Support/JetBrains/<Product><Year>.<N>`, one folder per version: `WebStorm2024.3`, `IntelliJIdea2025.1`, `PyCharmCE2024.1`.

## What happens if you remove it

The folder goes to the Trash; your current version keeps its own settings. A newer version copies the settings
when it first starts, but if it did not (the copy was declined, or the newer version was never opened), custom
keymaps, code styles and templates are lost. It does not come back: if you install that older version again, it
starts with default settings and downloads its plugins again. Medium, not Low: the plugins alone would download again,
but one folder holds both, so the higher level wins.

## How the app decides

The module `jetbrains` (code, not rules: the answer depends on the installed and running apps across the three
places) claims a folder only when all of these hold:

- it is a folder (a file or a link is never claimed) directly inside `JetBrains/` of the place above;
- its name is **exactly** a known product's folder name, case included, then `<year>.<minor>` and nothing after: four
  digits starting `20`, a dot, one or two digits with no leading zero (`WebStorm2024.3`, `PyCharmCE2024.1`). The
  products and their bundle ids are the module's table, `modules/jetbrains/products.toml`.

Refused, so never claimed: `WebStorm2024`, `WebStorm2024.3.1`, `WebStorm2024.3.bak`, `WebStorm2024.3 copy`,
`WebStorm-2024.3`, `WebStorm.2024.3`, `WebStorm2024.03`, `WebStorm1999.1`, `webstorm2024.2`, an unknown product
(`FooStorm2024.1`), the vendor's other folders (`Toolbox`, `consentOptions`, `JetBrainsClient2025.1`), and the layout
before the 2020 versions (no `JetBrains/` folder). `PyCharmCE2024.1` belongs to PyCharm Community Edition, never to
PyCharm.

For each product, the first answer that applies, for every folder of the product:

| # | Situation                                                                                 | Result                                       |
| - | ----------------------------------------------------------------------------------------- | -------------------------------------------- |
| 1 | the installed or the running apps cannot be listed; or an app or apps folder could not be read | left out: the app could not list your apps; or it could not read every app |
| 2 | an app with one of the product's bundle ids is running                                    | left out: the IDE is open                    |
| 3 | an app of the vendor (`com.jetbrains.…`) that the products table does not list is running | left out: another JetBrains app is open, for every product |
| 4 | no installed app has one of the product's bundle ids                                      | the IDE is no longer installed: [`jetbrains.uninstalled`](./jetbrains-uninstalled.md) decides every folder of it |
| 5 | an installed app of the product has a version that cannot be read                         | left out: the version in use is unknown      |
| 6 | the folder's version is one an installed app of the product has                           | left out: the version you use                |
| 7 | the folder's version is newer than the newest installed one                               | left out: a newer version                    |
| 8 | the folder's version is older than the newest installed one                               | **offered**                                  |

A release and an early access build installed side by side are both in use. An open IDE keeps every version of its
product, the older ones too: a just-updated IDE reads the older folder to import its settings at its first start. An
app of the vendor the table does not list may be a new IDE, so it keeps every product's folders while it runs; the
vendor's apps that are not IDEs (the Toolbox app) are listed in the table and keep nothing.

# Caches of installed apps

**Since:** 2026-11-19 · **Proposed by:** the agent

Built by plan [030](../plans/030-installed-app-caches/readme.md) (2026-11-21) as the module `app-caches`: the folders
`~/Library/Caches/<bundle id>` that an installed app owns (its id, a helper's, or an id under them), one finding per
folder, Low, emptied into the Trash while the app and its helpers are closed. The web-engine caches in apps' support
folders, browsers, game launchers, the Metal shader cache, containers and Apple's apps are
[deferred](../deferred/installed-app-caches-more-places.md). The open questions are answered there: the generic finder
waits for a table that ties a folder to its app by bundle id (the layout alone cannot name the owner); a chat or music
app's offline data in its cache is a cache, at Low, and the texts say it downloads again.

## Problem

Apps the user still uses keep large caches that nobody empties: chat and music apps, editors built on a web engine,
browsers, game launchers, the GPU shader cache. Each one alone is hundreds of megabytes, together often tens of
gigabytes. The app-leftovers module covers only apps that are gone; the developer-caches module covers package
managers and build caches. The caches of installed, everyday apps are not covered.

## Direction

- **A generic finder of web-engine caches inside an app's own folder**: many apps (Discord, Slack, Spotify, Visual
  Studio Code, Microsoft Teams, Notion and others) embed the same web engine and keep its cache folders (`Cache`,
  `Code Cache`, `GPUCache`, `Service Worker/CacheStorage`) under `~/Library/Application Support/<app>/`. One rule that
  recognizes that folder layout, instead of one rule per app; the app is named from its folder and its bundle id.
- **Browsers**: Safari, Chrome, Firefox, Edge, Brave, Vivaldi, Opera, Arc — the cache only, never history, cookies,
  passwords or profiles.
- **Game launchers**: Steam, Epic Games, Battle.net — their download and web caches.
- **The Metal shader cache**: safe, but rebuilding it costs time on the next launch of each game or app — offered
  unticked ("safe, but unticked", [`best-practices.md`](./best-practices.md) #28).
- A cache is offered only while its app is not running (the app's own process may rewrite it), and the folder is
  emptied, not removed ([`best-practices.md`](./best-practices.md) #25).
- Danger level Safe: the app rebuilds the cache; the cost is a slower first start and, for some, a re-download.

Open questions: how sure the generic finder must be before it claims a folder (the layout alone, or the layout and a
known file inside it); whether offline data of a chat app (downloaded media) is a cache or the user's data.

**Drafted in plan [034](../plans/034-browser-caches/readme.md)** (2026-11-22): browsers and known web-engine apps, by a table, never a generic finder. Game launchers, the Metal shader cache, Safari and containers stay [deferred](../deferred/installed-app-caches-more-places.md).

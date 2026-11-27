# Testers' feedback on 0.1.0

**Agreed:** 2026-11-26 and 2026-11-27, by the user, who relayed it · **Due at:** 0.2.0 planning

## What is postponed

The first outside feedback on the 0.1.0 release, from developers who ran it on their own Macs. Each point is weighed
when 0.2.0 is planned; a point taken into a plan, or dropped by the user, leaves this file, and the file and its row
go with the last one.

## The first tester (2026-11-26)

### What worked

- The split into categories by how safe they are to remove reads well.
- It found long-unused `node_modules` folders.
- Its numbers matched what another cleaner showed for the same places (npm's cache).
- Xcode's old builds that are no longer used were found; another cleaner does not find them. "If it is worked on
  further, it will be great."

### What did not

1. **How to remove what was found was not obvious.** The selection bar at the bottom of the window did not catch the
   eye; the tester looked for the way to remove the items for a while. Needs a design: a more visible bar, a "Review"
   step in sight once something is ticked, or a hint on the first tick (mockup first).
2. **Where the rest of the disk went.** The tester could clean 5–6 GB at most while the disk was nearly full, and did
   not know where the rest went. The storage map (plan [012](../plans/012-storage-map/readme.md)) and the Largest
   screen answer part of it, but the tester did not reach them from the result. Largest's Folders (plan
   [044](../plans/044-largest-folders/readme.md)) answers part of it too: the big folders left alone. The line on the
   scan's result is still open. Leads:
   [Intelligent Storage Explainer](../ideas/storage-explainer.md), [System Data Analyzer](../ideas/system-data-analyzer.md),
   and a line on the scan's result that says how much of the used space it explains and where to look for the rest.
3. **Data of AI coding assistants and the projects they leave behind**: a new idea,
   [AI coding assistants' data](../ideas/ai-coding-assistants-data.md).
4. **Big projects not changed for a long time, whoever made them** (the user's addition): a new idea,
   [Big projects left alone](../ideas/big-stale-projects.md), taken by plan
   [044](../plans/044-largest-folders/readme.md) (Largest's Folders, Show in Finder only).
5. **Other accounts, more tools, many small places** (the tester's second message): new ideas
   [Other users on the Mac](../ideas/other-users-on-the-mac.md), [.NET and NuGet packages](../ideas/dotnet-nuget-packages.md)
   and [Many small places that add up](../ideas/many-small-places.md).

## The second tester (2026-11-27)

1. **Who it is for was not clear at the top of the site.** "Make it clear at the top that it is for people who develop
   on a Mac." Done on `main` the same day: a "For developers on a Mac" badge above the hero's title, and a lead that
   opens with what it finds on a developer's Mac.
2. **Answers before searching.** "A troubleshooting section right away, so people do not search on their own." Done
   on `main` the same day: a Troubleshooting section after Install (Help in the navigation) with six questions, from
   macOS not opening Fresher to reporting a problem, each answered with the app's own texts.
3. **Full Disk Access asked too late.** "I opened it, scanned, and then saw many folders were not scanned; better if
   it asked for the permission at once, with an option to skip, so I scan everything right away — a very minimal
   onboarding." A new idea, [A first-run onboarding](../ideas/first-run-onboarding.md), a candidate for 0.2.0.

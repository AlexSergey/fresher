# The first tester's feedback on 0.1.0

**Agreed:** 2026-11-26, by the user, who relayed it · **Due at:** 0.2.0 planning

## What is postponed

The first outside feedback on the 0.1.0 release, from a developer who ran it on their own Mac. Each point is weighed
when 0.2.0 is planned; a point taken into a plan, or dropped by the user, leaves this file, and the file and its row
go with the last one.

## What worked

- The split into categories by how safe they are to remove reads well.
- It found long-unused `node_modules` folders.
- Its numbers matched what another cleaner showed for the same places (npm's cache).
- Xcode's old builds that are no longer used were found; another cleaner does not find them. "If it is worked on
  further, it will be great."

## What did not

1. **How to remove what was found was not obvious.** The selection bar at the bottom of the window did not catch the
   eye; the tester looked for the way to remove the items for a while. Needs a design: a more visible bar, a "Review"
   step in sight once something is ticked, or a hint on the first tick (mockup first).
2. **Where the rest of the disk went.** The tester could clean 5–6 GB at most while the disk was nearly full, and did
   not know where the rest went. The storage map (plan [012](../plans/012-storage-map/readme.md)) and the Largest
   screen answer part of it, but the tester did not reach them from the result. Leads:
   [Intelligent Storage Explainer](../ideas/storage-explainer.md), [System Data Analyzer](../ideas/system-data-analyzer.md),
   and a line on the scan's result that says how much of the used space it explains and where to look for the rest.
3. **Data of AI coding assistants and the projects they leave behind**: a new idea,
   [AI coding assistants' data](../ideas/ai-coding-assistants-data.md).
4. **Big projects not changed for a long time, whoever made them** (the user's addition): a new idea,
   [Big projects left alone](../ideas/big-stale-projects.md), drafted as plan
   [044](../plans/044-largest-folders/readme.md) after [043](../plans/043-build-flags/readme.md) turns Largest and
   High back on.
5. **Other accounts, more tools, many small places** (the tester's second message): new ideas
   [Other users on the Mac](../ideas/other-users-on-the-mac.md), [.NET and NuGet packages](../ideas/dotnet-nuget-packages.md)
   and [Many small places that add up](../ideas/many-small-places.md).

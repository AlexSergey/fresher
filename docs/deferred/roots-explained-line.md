# A root's explained line in the by-place roots list

**Agreed:** 2026-11-17, by delegation (plan [012](../plans/012-storage-map/readme.md), 04/05) · **Due at:** when
several scan roots on one volume are common

## What is postponed

When a volume holds several scan roots, the by-place view lists them as rows, each with its size. A root's row does
not say how much of it the modules explain, unlike a folder's row.

## What doing it needs

A core change: the roots come as plain node facts today; they would carry each root's `explained` sizes (the engine's
`Shares` already hold them by root), and the roots list would show the line as a folder row does.

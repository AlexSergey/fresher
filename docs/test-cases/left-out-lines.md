# Left-out lines on a real Mac

What a category's details show when the modules leave items out: the lines below the offered items, folded, and no
"0 bytes / 0 items" when nothing is offered. The autotests hold the screens with a mocked core; these cases check them
with a real scan, where a long list of left-out items is common.

## Setup

A release or development build (not sandbox mode) on a Mac with:

- Xcode installed, at least one project built (its build data is in `~/Library/Developer/Xcode/DerivedData`), and
  Xcode open for the whole run;
- several projects with `node_modules` changed within the days set in Settings for old projects (a fresh
  `npm install` in each is enough), and at least one project older than those days.

## Cases

### LO-01 Many left-out items are folded at the bottom

Scan, then open the details of the old projects' packages. Below the offered items is one bar per reason, for example
"12 items left out (3.4 GB): It or its project changed within the days set in Settings, so it is kept. …", with a
chevron, and no item under it. Pressing the bar, or Tab to it then Enter, opens its items (name, path from `~`, size,
then "And N more items" past the largest ten) and turns the chevron; pressing again folds them. Nothing on the lines
can be selected. Leaving the screen and coming back shows them folded again.

### LO-02 A category whose items are all left out shows no zeros

With Xcode open, scan. Xcode's build data has no card; it is under "Found, but left out" with its line folded, and its
items open from it. Open its Details: the header shows the name and the group, with no size and no "0 items"; the note
says "Everything found here was left out. The lines below say why.", and the folded line is below it.

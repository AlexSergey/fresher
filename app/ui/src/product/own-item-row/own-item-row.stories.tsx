import type { JSX } from 'react';

import type { LargestRow, OwnRow } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { LARGEST_DICTIONARY, LARGEST_ROWS } from '../../test/largest-scan';
import { OwnItemRow, type OwnItemRowProps, ownItemRowsClass } from './own-item-row';
import styles from './own-item-row.stories.module.css';

const nothing = (): void => undefined;

const [IMAGE, PACKAGE, PART, , ARCHIVE] = LARGEST_ROWS as readonly [
  LargestRow,
  LargestRow,
  LargestRow,
  LargestRow,
  LargestRow,
];

// A folder of the folder view: it holds 3.1 GB that Scan explains, so it opens and is not offered to be picked whole.
const FOLDER: OwnRow = {
  explained: { allocated: 3_100_000_000, completeness: 'complete', logical: 3_100_000_000, reclaimable: 3_100_000_000 },
  kind: 'folder',
  node: { ...IMAGE.node, kind: 'folder', modified: 1_735_689_600, name: 'Developer' },
  offer: { kind: 'withheld', reason: 'holds-explained' },
  opens: true,
  picked: null,
};

// One row in a list's frame, as the Largest screen shows it.
const row = (
  item: LargestRow | OwnRow,
  props: Partial<OwnItemRowProps> = {},
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(LARGEST_DICTIONARY);

    return (
      <div className={`${styles.frame ?? ''} ${ownItemRowsClass}`} role="list">
        <OwnItemRow checked={false} onToggle={nothing} row={item} {...props} />
      </div>
    );
  },
  width: 640,
});

// A file offered, a package, an archive with its hint, a row withheld, a folder that opens, a row checked.
export const stories = storiesOf('OwnItemRow', {
  archive: row(ARCHIVE),
  checked: row(IMAGE, { checked: true }),
  file: row(IMAGE),
  folder: row(FOLDER, { onOpen: nothing }),
  package: row(PACKAGE),
  withheld: row(PART),
});

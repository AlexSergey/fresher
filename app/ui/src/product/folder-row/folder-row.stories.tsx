import type { JSX } from 'react';

import type { FolderRow as FolderRowData } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { FOLDERS, LARGEST_DICTIONARY } from '../../test/largest-scan';
import { FolderRow, folderRowsClass } from './folder-row';
import styles from './folder-row.stories.module.css';

const nothing = (): void => undefined;

const ROWS: readonly FolderRowData[] = FOLDERS.rows;
const [PLAIN, PROJECT, YEARS] = ROWS as readonly [FolderRowData, FolderRowData, FolderRowData];

const LONG: FolderRowData = {
  ...PLAIN,
  node: { ...PLAIN.node, name: 'Client footage, raw takes and exports of the winter campaign, all cameras' },
  place: '~/Movies/Projects/Clients/2022/Winter/Footage/Cameras/Raw/Takes',
};

// One row in a list's frame, as the Folders tab shows it.
const row = (item: FolderRowData): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(LARGEST_DICTIONARY);

    return (
      <div className={`${styles.frame ?? ''} ${folderRowsClass}`} role="list">
        <FolderRow onReveal={nothing} row={item} />
      </div>
    );
  },
  width: 640,
});

// A plain folder, a project, one left a single year, a long name over a long path.
export const stories = storiesOf('FolderRow', {
  folder: row(PLAIN),
  long: row(LONG),
  project: row(PROJECT),
  years: row({ ...YEARS, leftFor: { count: 1, unit: 'years' } }),
});

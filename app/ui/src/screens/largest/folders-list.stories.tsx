import type { JSX } from 'react';

import type { FoldersPage } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { CoreContext, createCoreStore } from '../../state/store';
import { foldersPageOf, LARGEST_DICTIONARY, manyFolders } from '../../test/largest-scan';
import { FoldersList } from './folders-list';

const nothing = (): void => undefined;

// The Folders tab over a page: the store is only there for a refused reveal, which a story never makes.
const tab = (page: FoldersPage): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(LARGEST_DICTIONARY);

    return (
      <CoreContext value={createCoreStore()}>
        <FoldersList folders={{ failed: false, more: nothing, page }} />
      </CoreContext>
    );
  },
  width: 760,
});

// Five folders shown of eight, the rest one segment on the bar; and a home with none left alone.
export const stories = storiesOf('FoldersList', {
  empty: tab(foldersPageOf({ limit: 100, offset: 0 }, [])),
  'left-alone': tab(foldersPageOf({ limit: 5, offset: 0 }, manyFolders(3))),
});

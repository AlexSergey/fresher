import type { JSX } from 'react';

import type { AppSummary } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  APPS_DICTIONARY,
  MANAGED,
  NEEDS_ADMIN,
  notOfferedApp,
  OLD_EDITOR,
  OPEN_NOW,
  UNDATED,
  VIDEO_STUDIO,
} from '../../test/apps-list';
import { AppRow } from './app-row';

const nothing = (): void => undefined;

// One row in a list's frame, as the Apps screen shows it.
const row = (
  summary: AppSummary,
  partsOpen = false,
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary(APPS_DICTIONARY);

    return (
      <div role="list">
        <AppRow onOpen={nothing} partsOpen={partsOpen} summary={summary} />
      </div>
    );
  },
  width: 640,
});

// An app offered, one not used for long and with no version, one the core cannot date, one for every reason it is not
// offered, and the breakdown open.
export const stories = storiesOf('AppRow', {
  installedBy: row(MANAGED),
  locked: row(notOfferedApp('locked', 6, 'Notes Plus')),
  needsAdministrator: row(NEEDS_ADMIN),
  noDate: row(UNDATED),
  offered: row(VIDEO_STUDIO),
  open: row(OPEN_NOW),
  partsOpen: row(VIDEO_STUDIO, true),
  protected: row(notOfferedApp('protected', 7, 'Disk Guard')),
  tooManyFiles: row(notOfferedApp('too-many-files', 8, 'Game Hub')),
  unmeasured: row(notOfferedApp('unmeasured', 9, 'Mystery App')),
  unusedNoVersion: row(OLD_EDITOR),
});

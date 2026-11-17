import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { withStorageTexts } from '../storage-map/story-data';
import { AppRow } from './app-row';
import { MEANING } from './story-data';

const APPS = MEANING.flatMap((group) => (group.kind === 'app' ? [group.value] : []));

// The apps of the meaning list's story, the first one opened or not.
const apps = (defaultExpanded: boolean): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: withStorageTexts(() => (
    <div aria-label="Apps" role="group">
      {APPS.map((app, index) => (
        <AppRow
          key={app.app.id}
          {...app}
          defaultExpanded={defaultExpanded && index === 0}
          onOpen={() => undefined}
          onOpenPlace={() => undefined}
        />
      ))}
    </div>
  )),
  width: 600,
});

export const stories = storiesOf('AppRow', {
  collapsed: apps(false),
  expanded: apps(true),
});

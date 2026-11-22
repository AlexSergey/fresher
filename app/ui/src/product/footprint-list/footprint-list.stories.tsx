import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { VIDEO_STUDIO_DETAIL } from '../../test/app-detail';
import { APPS_DICTIONARY } from '../../test/apps-list';
import { FINDING_DICTIONARY } from '../../test/finding-fixtures';
import { FootprintList } from './footprint-list';

const state = (
  items: typeof VIDEO_STUDIO_DETAIL.items,
): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: function Story(): JSX.Element {
    setCoreDictionary({ ...FINDING_DICTIONARY, ...APPS_DICTIONARY });

    return <FootprintList items={items} />;
  },
  width: 640,
});

// An app's whole footprint (every kind, one kept row), and a short list of what goes with both choices and with only one.
export const stories = storiesOf('FootprintList', {
  everyKind: state(VIDEO_STUDIO_DETAIL.items),
  oneOfEach: state(VIDEO_STUDIO_DETAIL.items.filter(({ kind }) => ['bundle', 'caches', 'settings'].includes(kind))),
});

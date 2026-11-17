import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { withStorageTexts } from '../storage-map/story-data';
import { PlaceList } from './place-list';
import { DESCENT, EMPTY, HOME, PARTIAL, type StoryPlace, UNREADABLE } from './story-data';

const shown = (place: StoryPlace): { readonly render: () => JSX.Element; readonly width: number } => ({
  render: withStorageTexts(() => (
    <PlaceList
      crumbs={place.path.map(({ id, name }) => ({ key: String(id.index), name }))}
      failure={null}
      folder={place.folder}
      onOpen={() => undefined}
      onRangeChange={() => undefined}
      onStep={() => undefined}
      rowAt={(index) => place.rows[index]}
      total={place.rows.length}
    />
  )),
  width: 720,
});

export const stories = storiesOf('PlaceList', {
  descent: shown(DESCENT),
  empty: shown(EMPTY),
  home: shown(HOME),
  partial: shown(PARTIAL),
  unreadable: shown(UNREADABLE),
});

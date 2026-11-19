import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { BUSY } from '../../test/demo-scan';
import { DEMO_TOTALS, DEMO_TOTALS_FOUND } from '../scan-hero/story-data';
import { withStorageTexts } from '../storage-map/story-data';
import { MeaningList, type MeaningListProps } from './meaning-list';
import { MEANING } from './story-data';

type Shown = Pick<MeaningListProps, 'access' | 'failure' | 'groups' | 'unavailable'>;

const list = ({ access, failure, groups, unavailable }: Shown): (() => JSX.Element) =>
  withStorageTexts(() => (
    <MeaningList
      access={access}
      failure={failure}
      found={DEMO_TOTALS_FOUND}
      groups={groups}
      onOpen={() => undefined}
      onOpenPlace={() => undefined}
      onToggle={() => undefined}
      scanning={false}
      selected={(category) => category === 'demo.caches'}
      totals={DEMO_TOTALS}
      {...(unavailable === undefined ? {} : { unavailable })}
    />
  ));

export const stories = storiesOf('MeaningList', {
  'cards-only': { render: list({ failure: null, groups: null }), width: 900 },
  complete: { render: list({ failure: null, groups: MEANING }), width: 900 },
  failed: { render: list({ failure: { kind: 'refused', refusal: BUSY }, groups: null }), width: 900 },
  'with-unavailable': {
    render: list({
      access: new Map([
        [
          'demo.volumes',
          { kind: 'unavailable', text: 'Docker is installed but not running. Start it, then scan again.' },
        ],
      ]),
      failure: null,
      groups: MEANING,
      unavailable: DEMO_TOTALS.filter(({ category }) => category === 'demo.volumes').map((total) => ({
        ...total,
        count: 0,
        sizes: { allocated: 0, completeness: 'complete', logical: 0, reclaimable: 0 },
      })),
    }),
    width: 900,
  },
});

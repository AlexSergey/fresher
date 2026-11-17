import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { BUSY } from '../../test/demo-scan';
import { DEMO_TOTALS, DEMO_TOTALS_FOUND } from '../scan-hero/story-data';
import { withStorageTexts } from '../storage-map/story-data';
import { MeaningList, type MeaningListProps } from './meaning-list';
import { MEANING } from './story-data';

type Shown = Pick<MeaningListProps, 'failure' | 'groups'>;

const list = ({ failure, groups }: Shown): (() => JSX.Element) =>
  withStorageTexts(() => (
    <MeaningList
      failure={failure}
      found={DEMO_TOTALS_FOUND}
      groups={groups}
      onOpen={() => undefined}
      onOpenPlace={() => undefined}
      onToggle={() => undefined}
      scanning={false}
      selected={(category) => category === 'demo.caches'}
      totals={DEMO_TOTALS}
    />
  ));

export const stories = storiesOf('MeaningList', {
  'cards-only': { render: list({ failure: null, groups: null }), width: 900 },
  complete: { render: list({ failure: null, groups: MEANING }), width: 900 },
  failed: { render: list({ failure: { kind: 'refused', refusal: BUSY }, groups: null }), width: 900 },
});

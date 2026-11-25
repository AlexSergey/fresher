import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { DEMO_TOTALS, withTexts } from '../../test/story-data';
import { CategoryGrid } from './category-grid';

const grid = withTexts((): JSX.Element => (
  <CategoryGrid
    icon={() => 'cache'}
    label="Categories"
    onDetails={() => undefined}
    onIgnore={() => undefined}
    onToggle={() => undefined}
    scanning={false}
    selected={(category) => category === 'demo.caches'}
    totals={DEMO_TOTALS}
  />
));

export const stories = storiesOf('CategoryGrid', {
  'one-column': { render: grid, width: 400 },
  'two-columns': { render: grid, width: 700 },
});

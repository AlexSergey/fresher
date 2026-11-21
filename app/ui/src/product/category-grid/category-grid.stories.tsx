import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { DEMO_TOTALS, DEMO_TOTALS_FOUND, withTexts } from '../scan-hero/story-data';
import { CategoryGrid } from './category-grid';

const grid = withTexts((): JSX.Element => (
  <CategoryGrid
    found={DEMO_TOTALS_FOUND}
    label="Categories"
    moduleOf={() => ({ id: 'demo', name: 'Demo module' })}
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
  'three-columns': { render: grid, width: 1000 },
  'two-columns': { render: grid, width: 700 },
});

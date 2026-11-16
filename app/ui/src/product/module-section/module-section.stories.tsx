import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { CategoryGrid } from '../category-grid/category-grid';
import { DEMO_TOTALS, DEMO_TOTALS_FOUND, withTexts } from '../scan-hero/story-data';
import { ModuleSection, type ModuleSectionProps } from './module-section';

const WIDTH = 760;

const section = (state: ModuleSectionProps['state']): (() => JSX.Element) =>
  withTexts(() => (
    <ModuleSection found={DEMO_TOTALS_FOUND} module="demo" state={state}>
      <CategoryGrid
        found={DEMO_TOTALS_FOUND}
        onOpen={() => undefined}
        onToggle={() => undefined}
        scanning={state.kind === 'scanning'}
        selected={() => false}
        totals={DEMO_TOTALS.slice(0, 2)}
      />
    </ModuleSection>
  ));

export const stories = storiesOf('ModuleSection', {
  failed: { render: section({ kind: 'failed', reason: { args: {}, key: 'demo.error.failed' } }), width: WIDTH },
  finished: { render: section({ kind: 'completed' }), width: WIDTH },
  scanning: { render: section({ kind: 'scanning' }), width: WIDTH },
});

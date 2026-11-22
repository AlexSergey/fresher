import type { JSX } from 'react';

import type { Insight } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  DEMO_INSIGHT,
  DEMO_INSIGHT_DICTIONARY,
  DEMO_INSIGHT_STOPPED,
  DEMO_INSIGHT_UNKNOWN,
} from '../../test/demo-insight';
import { InsightCard } from './insight-card';

const WIDTH = 360;

const card =
  (insight: Insight): (() => JSX.Element) =>
  () => {
    setCoreDictionary(DEMO_INSIGHT_DICTIONARY);

    return <InsightCard insight={insight} onDetails={() => undefined} />;
  };

export const stories = storiesOf('InsightCard', {
  answered: { render: card(DEMO_INSIGHT), width: WIDTH },
  'nothing-known': { render: card(DEMO_INSIGHT_UNKNOWN), width: WIDTH },
  stopped: { render: card(DEMO_INSIGHT_STOPPED), width: WIDTH },
});

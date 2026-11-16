import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { DEMO_DICTIONARY, DEMO_FINDINGS } from '../../test/demo-scan';
import { ResultSummary, type ResultSummaryProps } from './result-summary';
import styles from './result-summary.stories.module.css';

const ignore = (): void => undefined;

const DICTIONARY = {
  ...DEMO_DICTIONARY,
  'demo.skipped.gone': 'It was already gone when the cleanup reached it.',
  'demo.skipped.in-use': 'An app is using it. Quit the app and clean again.',
};

const FREED = DEMO_FINDINGS.reduce((total, finding) => total + finding.sizes.allocated, 0);

const summary = (props: Partial<ResultSummaryProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    // Each story reads the demo texts; set here, so a story shown alone has them whatever else the gallery loads.
    setCoreDictionary(DICTIONARY);

    return (
      <div className={styles.stage}>
        <ResultSummary
          freed={{ allocated: FREED, completeness: 'complete', logical: FREED, reclaimable: FREED }}
          method="trash"
          moved={DEMO_FINDINGS.length}
          onDone={ignore}
          onOpenTrash={ignore}
          skipped={[]}
          {...props}
        />
      </div>
    );
  };

export const stories = storiesOf('ResultSummary', {
  'all-moved': { render: summary({}), surface: 'raised', width: 560 },
  permanent: { render: summary({ method: 'permanent' }), surface: 'raised', width: 560 },
  'some-skipped': {
    render: summary({
      freed: { allocated: 1_500_000_000, completeness: 'complete', logical: 1_500_000_000, reclaimable: 1_500_000_000 },
      moved: 1,
      skipped: [
        {
          reason: { args: {}, key: 'demo.skipped.in-use' },
          title: DEMO_FINDINGS[1]?.title ?? { args: {}, key: 'demo.finding.title' },
        },
      ],
    }),
    surface: 'raised',
    width: 560,
  },
});

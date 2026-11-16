import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  CLEANUP_DICTIONARY,
  DELETED_REPORT,
  DEMO_PLAN,
  MOVED_REPORT,
  SKIPPED_OUTCOMES,
  SKIPPED_REPORT,
  STOPPED_REPORT,
  UNMEASURED_REPORT,
} from '../../test/cleanup-plan';
import { DEMO_DICTIONARY } from '../../test/demo-scan';
import { ResultSummary, type ResultSummaryProps } from './result-summary';
import styles from './result-summary.stories.module.css';

const ignore = (): void => undefined;

const TITLES = DEMO_PLAN.groups.flatMap((group) => group.items.map((item) => item.title));

// The skipped items of `SKIPPED_REPORT`, with the titles the confirmation showed.
const PROBLEMS: ResultSummaryProps['problems'] = SKIPPED_OUTCOMES.flatMap((outcome, index) =>
  outcome.kind === 'skipped' ? [{ outcome, title: TITLES[index] ?? null }] : [],
);

const summary = (props: Partial<ResultSummaryProps>): (() => JSX.Element) =>
  function Story(): JSX.Element {
    // Each story reads the demo texts; set here, so a story shown alone has them whatever else the gallery loads.
    setCoreDictionary({ ...DEMO_DICTIONARY, ...CLEANUP_DICTIONARY });

    return (
      <div className={styles.stage}>
        <ResultSummary
          onDone={ignore}
          onSeeCleanup={ignore}
          onShowInTrash={ignore}
          problems={[]}
          report={MOVED_REPORT}
          {...props}
        />
      </div>
    );
  };

export const stories = storiesOf('ResultSummary', {
  deleted: { render: summary({ report: DELETED_REPORT }), surface: 'raised', width: 560 },
  moved: { render: summary({}), surface: 'raised', width: 560 },
  'not-measured': { render: summary({ report: UNMEASURED_REPORT }), surface: 'raised', width: 560 },
  'some-skipped': { render: summary({ problems: PROBLEMS, report: SKIPPED_REPORT }), surface: 'raised', width: 560 },
  stopped: { render: summary({ report: STOPPED_REPORT }), surface: 'raised', width: 560 },
});

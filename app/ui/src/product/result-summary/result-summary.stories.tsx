import type { JSX } from 'react';

import type { Text } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary, tText } from '../../i18n';
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
import { BUILD_DICTIONARY, SANDBOX_TRASH } from '../../test/sandbox-build';
import { ResultSummary, type ResultSummaryProps } from './result-summary';
import styles from './result-summary.stories.module.css';

const ignore = (): void => undefined;

const TITLES = DEMO_PLAN.groups.flatMap((group) => group.items.map((item) => item.title));

// The skipped items of `SKIPPED_REPORT`, with the titles the confirmation showed.
const PROBLEMS: ResultSummaryProps['problems'] = SKIPPED_OUTCOMES.flatMap((outcome, index) =>
  outcome.kind === 'skipped' ? [{ outcome, title: TITLES[index] ?? null }] : [],
);

// A summary with `props`; with `trash`, the trash note a sandbox build gives, in the core's words.
const summary = (props: Partial<ResultSummaryProps>, trash?: Text): (() => JSX.Element) =>
  function Story(): JSX.Element {
    // Each story reads the demo texts; set here, so a story shown alone has them whatever else the gallery loads.
    setCoreDictionary({ ...DEMO_DICTIONARY, ...CLEANUP_DICTIONARY, ...BUILD_DICTIONARY });
    const note = trash === undefined ? {} : { trashNote: tText(trash) };

    return (
      <div className={styles.stage}>
        <ResultSummary
          onDone={ignore}
          onSeeCleanup={ignore}
          onShowInTrash={ignore}
          problems={[]}
          report={MOVED_REPORT}
          {...props}
          {...note}
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
  // A sandbox build's cleanup to the Trash says where its Trash is, in the core's words.
  'trash-in-sandbox': { render: summary({}, SANDBOX_TRASH), surface: 'raised', width: 560 },
});

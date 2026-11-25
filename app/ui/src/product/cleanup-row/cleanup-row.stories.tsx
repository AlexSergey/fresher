import type { JSX } from 'react';

import type { CleanupItemView, CleanupSummary } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import {
  CLEANUP_DICTIONARY,
  cleanupSummaryOf,
  DELETED_DETAIL,
  INTERRUPTED_DETAIL,
  MOVED_DETAIL,
  STOPPED_REPORT,
} from '../../test/cleanup-plan';
import { DEMO_DICTIONARY } from '../../test/demo-scan';
import { CleanupItemRow, CleanupRow } from './cleanup-row';
import styles from './cleanup-row.stories.module.css';

const ignore = (): void => undefined;

const DICTIONARY = { ...DEMO_DICTIONARY, ...CLEANUP_DICTIONARY };

const MOVED_ITEM = MOVED_DETAIL.items[0] as CleanupItemView;

const INSTALLER = MOVED_DETAIL.items[2] as CleanupItemView;

const cleanup = (summary: CleanupSummary): (() => JSX.Element) =>
  function Story(): JSX.Element {
    setCoreDictionary(DICTIONARY);

    return (
      <ul className={styles.list}>
        <CleanupRow onOpen={ignore} summary={summary} />
      </ul>
    );
  };

const item = (view: CleanupItemView, putBack: null | string = null): (() => JSX.Element) =>
  function Story(): JSX.Element {
    setCoreDictionary(DICTIONARY);

    return (
      <div role="list">
        <CleanupItemRow item={view} onPutBack={ignore} putBack={putBack} />
      </div>
    );
  };

export const stories = [
  ...storiesOf('CleanupRow', {
    deleted: { render: cleanup(DELETED_DETAIL.summary), surface: 'raised', width: 640 },
    'in-trash': { render: cleanup(MOVED_DETAIL.summary), surface: 'raised', width: 640 },
    interrupted: { render: cleanup(INTERRUPTED_DETAIL.summary), surface: 'raised', width: 640 },
    'not-in-trash': { render: cleanup({ ...MOVED_DETAIL.summary, inTrash: 0 }), surface: 'raised', width: 640 },
    'put-back': {
      render: cleanup({ ...MOVED_DETAIL.summary, inTrash: 1, putBack: 2 }),
      surface: 'raised',
      width: 640,
    },
    stopped: { render: cleanup(cleanupSummaryOf(4, STOPPED_REPORT, {})), surface: 'raised', width: 640 },
  }),
  ...storiesOf('CleanupItemRow', {
    deleted: { render: item(DELETED_DETAIL.items[0] ?? MOVED_ITEM), surface: 'raised', width: 640 },
    gone: { render: item({ ...INSTALLER, trash: 'gone' }), surface: 'raised', width: 640 },
    'in-trash': { render: item(MOVED_ITEM), surface: 'raised', width: 640 },
    occupied: {
      render: item(MOVED_ITEM, DICTIONARY['core.cleanup.put-back.occupied'] ?? null),
      surface: 'raised',
      width: 640,
    },
    'put-back': {
      render: item({ ...MOVED_ITEM, putBack: { kind: 'put-back' }, trash: null }),
      surface: 'raised',
      width: 640,
    },
    skipped: {
      render: item({ ...INSTALLER, outcome: { kind: 'skipped', reason: 'in-use' }, trash: null }),
      surface: 'raised',
      width: 640,
    },
    unknown: { render: item(INTERRUPTED_DETAIL.items[1] ?? MOVED_ITEM), surface: 'raised', width: 640 },
  }),
];

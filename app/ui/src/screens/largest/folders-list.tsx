import type { JSX } from 'react';

import type { TextArgs } from '../../i18n/types';
import type { FoldersPage } from '../../ipc/bindings.generated';

import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { failureOf, revealNode } from '../../ipc';
import { FolderRow, folderRowsClass } from '../../product/folder-row/folder-row';
import { useCoreStore } from '../../state/store';
import { FoldersSummary } from './folders-summary';
import styles from './largest-screen.module.css';
import { type LargestFolders } from './use-largest-folders';

export type FoldersListProps = { readonly folders: LargestFolders };

const keyOf = ({ node }: FoldersPage['rows'][number]): string => `${String(node.id.state)}:${String(node.id.index)}`;

// The rule's size and days, both the core's settings.
const ruleArgs = ({ days, minSize }: FoldersPage): TextArgs => ({
  days: { kind: 'count', value: days },
  size: { kind: 'size', value: minSize },
});

// The folders of the home left alone: what to do with them and the rule that lists them, the home's bar, the list's
// tiles, then the rows with Show in Finder, and Show more while rows remain. Fresher never removes them: no row ticks.
export const FoldersList = ({ folders: { failed, more, page } }: FoldersListProps): JSX.Element => {
  const store = useCoreStore();

  if (failed) {
    return <p className={styles.state}>{t('ui.screen.largest.unavailable')}</p>;
  }

  if (page === null) {
    return <p className={styles.state}>{t('ui.screen.largest.loading')}</p>;
  }

  if (page.total === 0) {
    return <p className={styles.state}>{t('ui.screen.largest.folders.empty', ruleArgs(page))}</p>;
  }

  return (
    <div className={styles.folders}>
      <div className={styles.lead}>
        <h2 className={styles.heading}>{t('ui.screen.largest.folders.heading')}</h2>
        <p className={styles.note}>{t('ui.screen.largest.folders.rule', ruleArgs(page))}</p>
      </div>
      <FoldersSummary page={page} />
      <Surface elevation={1} padding="none" radius="lg">
        <ul aria-label={t('ui.screen.largest.folders')} className={`${styles.list ?? ''} ${folderRowsClass}`}>
          {page.rows.map((row) => (
            <FolderRow
              key={keyOf(row)}
              onReveal={() => {
                revealNode(row.node.id).catch((error: unknown) => {
                  store.dispatch({ failure: failureOf(error), type: 'command-failed' });
                });
              }}
              row={row}
            />
          ))}
        </ul>
      </Surface>
      {page.rows.length < page.total ? (
        <div className={styles.more}>
          <Button onClick={more}>{t('ui.screen.largest.more')}</Button>
        </div>
      ) : null}
    </div>
  );
};

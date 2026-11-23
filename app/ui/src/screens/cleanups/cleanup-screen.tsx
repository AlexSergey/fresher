import type { JSX } from 'react';

import type { CleanupId, CleanupItemView } from '../../ipc/bindings.generated';

import { ListRow } from '../../design/data/list-row/list-row';
import { VirtualList } from '../../design/data/virtual-list/virtual-list';
import { Notice } from '../../design/feedback/notice/notice';
import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { CleanupItemRow } from '../../product/cleanup-row/cleanup-row';
import { RecoveryHelp } from '../../product/recovery-help/recovery-help';
import { useNavigation } from '../../state/navigation';
import styles from './cleanups-screen.module.css';
import { type OneCleanup, useCleanup } from './use-cleanup';

// A cleanup lists up to this many items as they are; more go into a virtual list.
const PLAIN_ITEMS = 50;

// Pixels: `--size-row`.
const ITEM_HEIGHT = 44;

type ItemsProps = { readonly cleanup: OneCleanup; readonly items: readonly CleanupItemView[] };

const Items = ({ cleanup, items }: ItemsProps): JSX.Element => {
  const label = t('ui.screen.cleanups.items');
  const row = (item: CleanupItemView): JSX.Element => (
    <CleanupItemRow
      item={item}
      key={item.index}
      onPutBack={() => {
        cleanup.putBack([item.index]);
      }}
      putBack={cleanup.answers.get(item.index) ?? null}
    />
  );

  return items.length > PLAIN_ITEMS ? (
    <div className={styles.long}>
      <VirtualList
        count={items.length}
        getItem={(index) => items[index]}
        getKey={(index) => String(items[index]?.index ?? index)}
        label={label}
        renderRow={(_, item, props) =>
          item === undefined ? (
            <ListRow {...props} title="" />
          ) : (
            <CleanupItemRow
              {...props}
              item={item}
              onPutBack={() => {
                cleanup.putBack([item.index]);
              }}
              putBack={cleanup.answers.get(item.index) ?? null}
            />
          )
        }
        rowHeight={ITEM_HEIGHT}
      />
    </div>
  ) : (
    <div aria-label={label} role="list">
      {items.map(row)}
    </div>
  );
};

// One cleanup: how it went, its items with what became of each, and putting back what is still in the Trash.
export const CleanupScreen = ({ cleanup: id }: { readonly cleanup: CleanupId }): JSX.Element => {
  const { navigate } = useNavigation();
  const cleanup = useCleanup(id);
  const { detail } = cleanup;
  const summary = detail?.summary;
  const inTrash = detail?.items.some((item) => item.trash === 'in-trash') ?? false;

  return (
    <>
      <Toolbar
        actions={
          <>
            <RecoveryHelp />
            <Button
              onClick={() => {
                navigate({ kind: 'cleanups' });
              }}
              variant="ghost"
            >
              {t('ui.screen.cleanups.all')}
            </Button>
          </>
        }
      />
      <PageHeader
        {...(summary === undefined
          ? {}
          : { subtitle: tText({ args: {}, key: `core.cleanup.ended.${summary.ended}` }) })}
        title={
          summary === undefined
            ? t('ui.screen.frame.nav.cleanups')
            : t('ui.cleanup-row.title', { date: { kind: 'date', value: summary.started } })
        }
      />
      <div className={styles.content}>
        {summary?.ended === 'interrupted' ? (
          <Notice title={t('ui.cleanup-row.state.interrupted')} tone="warning">
            {t('ui.screen.cleanups.interrupted')}
          </Notice>
        ) : null}
        {cleanup.refusal === null ? null : (
          <Notice live title={t('ui.screen.cleanups.failed-action')} tone="danger">
            {cleanup.refusal}
          </Notice>
        )}
        {detail === null ? null : (
          <>
            {inTrash ? (
              <div className={styles.actions}>
                <Button
                  onClick={() => {
                    cleanup.putBack(null);
                  }}
                  variant="primary"
                >
                  {t('ui.screen.cleanups.put-back-all')}
                </Button>
                <Button onClick={cleanup.showInTrash}>{t('ui.screen.cleanups.show-in-trash')}</Button>
              </div>
            ) : null}
            <Surface elevation={1} padding="small" radius="lg">
              <Items cleanup={cleanup} items={detail.items} />
            </Surface>
          </>
        )}
      </div>
    </>
  );
};

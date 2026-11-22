import { type JSX, useEffect, useState } from 'react';

import type { AppDetail, AppId, UninstallChoice } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { Notice } from '../../design/feedback/notice/notice';
import { ChevronLeft } from '../../design/icons/icons';
import { PageHeader } from '../../design/layout/page-header/page-header';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { ConfirmDialog } from '../../product/confirm-dialog/confirm-dialog';
import { FootprintList } from '../../product/footprint-list/footprint-list';
import { honestSize } from '../../product/sizes';
import { UninstallChoices } from '../../product/uninstall-choices/uninstall-choices';
import { confirm, discard, prepareUninstallPlan } from '../../state/cleanup-actions';
import { useNavigation } from '../../state/navigation';
import { useCore, useCoreStore } from '../../state/store';
import { AppManagementNotice } from './app-management-notice';
import styles from './app-screen.module.css';
import { NotOffered } from './not-offered';
import { useApp } from './use-app';

export type AppScreenProps = {
  readonly app: AppId;
};

type OfferedProps = {
  readonly detail: AppDetail;
  readonly onUninstall: (choice: UninstallChoice) => void;
  readonly preparing: boolean;
};

// The two choices with `keep-data` chosen when the page opens, and the button that asks the core for the plan.
const Offered = ({ detail, onUninstall, preparing }: OfferedProps): JSX.Element => {
  const [choice, setChoice] = useState<UninstallChoice>('keep-data');

  return (
    <UninstallChoices
      choices={detail.choices}
      onChoose={setChoice}
      onUninstall={() => {
        onUninstall(choice);
      }}
      preparing={preparing}
      value={choice}
    />
  );
};

// "What would this take with it?": one app's page. Its sentence and size, the notice about App Management, a link to each
// of its cleanable categories, every file it would lose, and when the core offers the uninstall, the two choices and the
// button that opens the confirmation. When it does not, the reason, and the one thing the user can do instead.
export const AppScreen = ({ app }: AppScreenProps): JSX.Element => {
  const store = useCoreStore();
  const { navigate } = useNavigation();
  const { index: appIndex, state: appState } = app;
  const { detail, refusal, reload, stale } = useApp(app);
  const cleanup = useCore((state) => state.cleanup);
  // What the cleanup state holds for this page: an uninstall of this very app, never another app's.
  const own =
    cleanup.kind === 'preparing' || cleanup.kind === 'confirming' || cleanup.kind === 'refused'
      ? cleanup.origin === 'uninstall' && cleanup.app?.state === appState && cleanup.app.index === appIndex
      : false;
  const refused = own && cleanup.kind === 'refused' ? cleanup.refusal : null;

  // A new scan made the id stale: the list has the app's new one.
  useEffect(() => {
    if (stale) {
      navigate({ kind: 'apps' });
    }
  }, [navigate, stale]);

  // Leaving the page forgets an uninstall still being asked for or confirmed: a late plan opens nowhere.
  useEffect(
    () => (): void => {
      const { cleanup: left } = store.getState();

      if (
        (left.kind === 'preparing' || left.kind === 'confirming') &&
        left.origin === 'uninstall' &&
        left.app?.state === appState &&
        left.app.index === appIndex
      ) {
        discard(store);
      }
    },
    [appIndex, appState, store],
  );

  // The core refused the uninstall: what the page offers may have changed, so the page is read again.
  useEffect(() => {
    if (refused !== null) {
      reload();
    }
  }, [refused, reload]);

  const back = (
    <Button
      icon={ChevronLeft}
      onClick={() => {
        navigate({ kind: 'apps' });
      }}
      variant="ghost"
    >
      {t('ui.screen.app.back')}
    </Button>
  );

  if (detail === null) {
    return (
      <>
        <Toolbar title={back} />
        <PageHeader title={t('ui.screen.frame.nav.apps')} />
        <div className={styles.content}>
          <p className={styles.state}>
            {refusal !== null && !stale ? tText(refusal.text) : t('ui.screen.app.loading')}
          </p>
        </div>
      </>
    );
  }

  const { summary } = detail;

  return (
    <>
      <Toolbar title={back} />
      <PageHeader subtitle={tText(summary.summary)} title={summary.name} />
      <div className={styles.content}>
        <div className={styles.size}>
          {summary.version === null ? null : (
            <span className={styles.version}>
              {t('ui.product.app-row.version', { version: { kind: 'text', value: summary.version } })}
            </span>
          )}
          <SizeText {...honestSize(summary.sizes)} size="title-3" />
        </div>
        {refused === null ? null : (
          <Notice
            action={{
              label: t('ui.screen.app.dismiss'),
              onClick: () => {
                store.dispatch({ type: 'cleanup-dismissed' });
              },
            }}
            live
            title={t('ui.screen.app.refused')}
            tone="warning"
          >
            {tText(refused.text)}
          </Notice>
        )}
        {detail.notice === null ? null : <AppManagementNotice notice={detail.notice} />}
        {detail.categories.length === 0 ? null : (
          <section aria-label={t('ui.screen.app.part-of-it')} className={styles.part}>
            <h2 className={styles.heading}>{t('ui.screen.app.part-of-it')}</h2>
            <div className={styles.links}>
              {detail.categories.map((category) => (
                <Button
                  key={category}
                  onClick={() => {
                    navigate({ category, kind: 'category' });
                  }}
                >
                  {tText({ args: {}, key: `${category}.name` })}
                </Button>
              ))}
            </div>
          </section>
        )}
        <section aria-label={t('ui.screen.app.files')} className={styles.part}>
          <h2 className={styles.heading}>{t('ui.screen.app.files')}</h2>
          <FootprintList items={detail.items} />
          <p className={styles.note}>{tText({ args: {}, key: 'core.apps.other-names' })}</p>
        </section>
        {summary.offer.kind === 'offered' ? (
          <Offered
            detail={detail}
            onUninstall={(choice) => {
              prepareUninstallPlan(store, app, choice);
            }}
            preparing={own && cleanup.kind === 'preparing'}
          />
        ) : (
          <NotOffered app={app} detail={detail} offer={summary.offer} />
        )}
      </div>
      {own && cleanup.kind === 'confirming' ? (
        <ConfirmDialog
          onCancel={() => {
            discard(store);
          }}
          onConfirm={(method) => {
            confirm(store, method);
            navigate({ kind: 'review' });
          }}
          open
          plan={cleanup.plan}
        />
      ) : null}
    </>
  );
};

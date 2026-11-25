import type { JSX } from 'react';

import type { CleanupState } from '../../state/cleanup';

import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { t, tText } from '../../i18n';
import { CleanupProgress } from '../../product/cleanup-progress/cleanup-progress';
import { ResultSummary } from '../../product/result-summary/result-summary';
import { showInTrash, stop } from '../../state/cleanup-actions';
import { useNavigation } from '../../state/navigation';
import { useCore, useCoreStore } from '../../state/store';
import { useAppInfo } from '../use-app-info';
import styles from './review-screen.module.css';

type CleanupPanelProps = {
  readonly cleanup: Extract<CleanupState, { readonly kind: 'finished' | 'running' }>;
};

// The review screen while a cleanup runs and once it is over: its progress, then its result.
export const CleanupPanel = ({ cleanup }: CleanupPanelProps): JSX.Element => {
  const store = useCoreStore();
  const { navigate } = useNavigation();
  const build = useAppInfo()?.build;
  const demo = useCore((state) => state.demo);
  // A sandbox's cleanups do not go to the Trash the user knows: the result says where they went, in the core's words.
  // A demo does not say it: the sentence names the sandbox's folder.
  const trashNote = build?.kind === 'sandbox' && !demo ? { trashNote: tText(build.trash) } : {};

  return (
    <>
      <Toolbar />
      <PageHeader title={t('ui.screen.frame.nav.review')} />
      <div className={styles.content}>
        <Surface elevation={1} padding="none" radius="lg">
          {cleanup.kind === 'running' ? (
            <CleanupProgress
              done={cleanup.done}
              method={cleanup.method}
              now={cleanup.current === null ? null : tText(cleanup.current)}
              onStop={() => {
                stop(store);
              }}
              stopping={cleanup.stopping}
              total={cleanup.total}
            />
          ) : (
            <ResultSummary
              onDone={() => {
                store.dispatch({ type: 'cleanup-dismissed' });
                navigate({ kind: 'scan' });
              }}
              onSeeCleanup={() => {
                store.dispatch({ type: 'cleanup-dismissed' });
                navigate({ cleanup: cleanup.cleanup, kind: 'cleanup' });
              }}
              onShowInTrash={() => {
                showInTrash(store);
              }}
              problems={cleanup.problems}
              report={cleanup.report}
              {...trashNote}
            />
          )}
        </Surface>
      </div>
    </>
  );
};

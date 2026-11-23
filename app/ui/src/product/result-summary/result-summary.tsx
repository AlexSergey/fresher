import { type JSX, useEffect, useId, useState } from 'react';

import type { CleanupReport, ItemOutcome, Text } from '../../ipc/bindings.generated';

import { AnimatedNumber } from '../../design/data/animated-number/animated-number';
import { AmbientLight } from '../../design/effects/ambient-light/ambient-light';
import { Icon } from '../../design/icons/icon';
import { Check } from '../../design/icons/icons';
import { useMotion } from '../../design/motion';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { formatSize } from '../../i18n/format-size';
import { skipReasonText } from '../cleanup-texts';
import { ProblemText } from '../problem-text/problem-text';
import { RecoveryHelp } from '../recovery-help/recovery-help';
import styles from './result-summary.module.css';

export type ResultSummaryProps = {
  readonly onDone: () => void;
  // Opens this cleanup in the history.
  readonly onSeeCleanup?: () => void;
  // Shows a moved item in the Trash; offered after a move to the Trash only.
  readonly onShowInTrash?: () => void;
  readonly problems: readonly ResultProblem[];
  // The core's report: every figure shown is its own.
  readonly report: CleanupReport;
  // Where the moved items are, when the build says it (a sandbox's own Trash); shown after a move to the Trash only.
  readonly trashNote?: string;
};

// An item that was not removed: the title the confirmation showed (`null` when it is not known) and why.
type ResultProblem = {
  readonly outcome: Extract<ItemOutcome, { readonly kind: 'failed' | 'skipped' }>;
  readonly title: null | Text;
};

const Reason = ({ problem: { outcome } }: { readonly problem: ResultProblem }): JSX.Element =>
  outcome.kind === 'failed' ? <ProblemText text={outcome.reason} /> : <>{skipReasonText(outcome.reason)}</>;

const countArg = (value: number): { readonly count: { readonly kind: 'count'; readonly value: number } } => ({
  count: { kind: 'count', value },
});

// What happened besides the removal: how many a tool removed, how a stopped cleanup ended.
const Notes = ({
  report,
  trashNote,
}: {
  readonly report: CleanupReport;
  readonly trashNote: string | undefined;
}): JSX.Element => {
  const { counts, ended, measured, method } = report;
  const size = { kind: 'size', value: measured ?? 0 } as const;

  return (
    <div className={styles.notes}>
      {counts.toolDone === 0 ? null : <p>{t('ui.result.tool-done', countArg(counts.toolDone))}</p>}
      {ended === 'cancelled' ? <p>{t('ui.result.stopped', countArg(counts.notReached))}</p> : null}
      {ended === 'stopped' || ended === 'interrupted' ? (
        <p>{tText({ args: {}, key: `core.cleanup.ended.${ended}` })}</p>
      ) : null}
      {method === 'trash' ? <p className={styles.later}>{t('ui.result.space-later')}</p> : null}
      <p>
        {measured === null
          ? t('ui.result.unmeasured')
          : t(method === 'trash' ? 'ui.result.measured-trash' : 'ui.result.measured', { size })}
      </p>
      {method === 'trash' && trashNote !== undefined ? <p className={styles.trashNote}>{trashNote}</p> : null}
    </div>
  );
};

// The cleaned moment, once the removal is over: a check mark that draws itself, the size counting up from 0 over one
// soft green light wave, then — with no motion — what was done, what was not and why, and the ways on. A move to the
// Trash says "Moved to the Trash", never "Freed": the space comes back when the Trash is emptied, and the space the
// disk measured is shown beside it. Serious, not festive: no bounce, no confetti. Under reduced motion the final
// values fade in once; with motion off they are simply there.
export const ResultSummary = ({
  onDone,
  onSeeCleanup,
  onShowInTrash,
  problems,
  report,
  trashNote,
}: ResultSummaryProps): JSX.Element => {
  const motion = useMotion();
  const titleId = useId();
  const problemsId = useId();
  const inviteId = useId();
  const trash = report.method === 'trash';
  const sizes = trash ? report.moved : report.deleted;
  // The number counts from 0: it mounts at 0 and is given the size on the next frame.
  const [counted, setCounted] = useState(0);
  // The wave and the green light belong to the first moment only; once the wave ends they never come back.
  const [waved, setWaved] = useState(false);
  const waving = motion === 'full' && !waved;
  const { counts } = report;
  const invited = trash && counts.moved > 0 && onShowInTrash !== undefined;
  const removed = trash ? counts.moved + counts.emptied : counts.deleted + counts.emptied;

  useEffect(() => {
    const frame = requestAnimationFrame(() => {
      setCounted(sizes.allocated);
    });

    return (): void => {
      cancelAnimationFrame(frame);
    };
  }, [sizes.allocated]);

  return (
    <section aria-labelledby={titleId} className={styles.summary} data-motion={motion}>
      <AmbientLight tone={waving ? 'safe' : 'iris'} />
      <div className={styles.moment}>
        <span className={styles.check}>
          <Icon icon={Check} size={24} />
        </span>
        <h2 className={styles.title} id={titleId}>
          {trash ? t('ui.result.moved-title') : t('ui.result.freed')}
        </h2>
        <span className={styles.freed}>
          {waving ? (
            <span
              aria-hidden="true"
              className={styles.wave}
              data-light-wave
              onAnimationEnd={() => {
                setWaved(true);
              }}
            />
          ) : null}
          <span className={styles.number}>
            <AnimatedNumber
              duration="deliberate"
              format={(value) => formatSize(value, sizes.completeness)}
              value={motion === 'full' ? counted : sizes.allocated}
            />
          </span>
        </span>
      </div>
      <p className={styles.method}>
        {trash ? t('ui.result.moved', countArg(removed)) : t('ui.result.deleted', countArg(removed))}
      </p>
      <Notes report={report} trashNote={trashNote} />
      {problems.length === 0 ? null : (
        <div className={styles.skipped}>
          <p className={styles.skippedTitle} id={problemsId}>
            {t('ui.result.skipped', countArg(problems.length))}
          </p>
          <ul aria-labelledby={problemsId} className={styles.list}>
            {problems.map((problem, index) => (
              // Two items may carry the same texts; their order is the order the core reported them in.
              // eslint-disable-next-line @eslint-react/no-array-index-key -- the list is fixed once shown
              <li className={styles.item} key={index}>
                <span className={styles.itemTitle}>
                  {problem.title === null ? t('ui.result.unnamed') : tText(problem.title)}
                </span>
                <span className={styles.reason}>
                  <Reason problem={problem} />
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}
      <div className={styles.actions}>
        {invited ? (
          <Button aria-describedby={inviteId} onClick={onShowInTrash}>
            {t('ui.result.show-in-trash')}
          </Button>
        ) : null}
        {onSeeCleanup === undefined ? null : <Button onClick={onSeeCleanup}>{t('ui.result.see-cleanup')}</Button>}
        <RecoveryHelp />
        <Button onClick={onDone} variant="primary">
          {t('ui.result.done')}
        </Button>
      </div>
      {invited ? (
        <p className={styles.invite} id={inviteId}>
          {t('ui.trash.invite')}
        </p>
      ) : null}
    </section>
  );
};

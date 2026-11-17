import type { JSX } from 'react';

import type { ModuleGroup, ModuleId, ModuleOutcome } from '../../ipc/bindings.generated';

import { AnimatedNumber } from '../../design/data/animated-number/animated-number';
import { Icon } from '../../design/icons/icon';
import { Check, CircleAlert, Minus, Square } from '../../design/icons/icons';
import { Button } from '../../design/primitives/button/button';
import { ProgressBar } from '../../design/primitives/progress-bar/progress-bar';
import { Tooltip } from '../../design/primitives/tooltip/tooltip';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { t, tText } from '../../i18n';
import { type InGroup, inGroups } from '../module-groups';
import { formatCount } from '../sizes';
import styles from './scan-progress.module.css';

// One module's part of a scan: not started, running, or how it ended.
export type ModuleProgress = {
  readonly group: ModuleGroup;
  readonly module: ModuleId;
  readonly state: ModuleOutcome | { readonly kind: 'running' } | { readonly kind: 'waiting' };
};

export type ScanProgressProps = {
  // How many files and folders the scan has looked at.
  readonly entries: number;
  readonly modules: readonly ModuleProgress[];
  readonly onStop: () => void;
};

type ChipState = ModuleProgress['state'];

// The word that tells a chip's state without its mark.
const STATE_WORDS = {
  cancelled: () => t('ui.scan-progress.module.cancelled'),
  completed: () => t('ui.scan-progress.module.done'),
  failed: () => t('ui.scan-progress.module.failed'),
  running: () => t('ui.scan-progress.module.running'),
  unavailable: () => t('ui.scan-progress.module.unavailable'),
  waiting: () => t('ui.scan-progress.module.waiting'),
} as const satisfies Record<ChipState['kind'], () => string>;

const Mark = ({ state }: { readonly state: ChipState }): JSX.Element => {
  switch (state.kind) {
    case 'cancelled':
    case 'unavailable':
      return <Icon icon={Minus} />;
    case 'completed':
      return <Icon icon={Check} />;
    case 'failed':
      return <Icon icon={CircleAlert} />;
    case 'running':
    case 'waiting':
      return <span aria-hidden="true" className={styles.dot} />;
  }
};

const Chip = ({ module, state }: Omit<ModuleProgress, 'group'>): JSX.Element => {
  const chip = (describedBy?: string): JSX.Element => (
    <span
      aria-describedby={describedBy}
      className={styles.chip}
      data-state={state.kind}
      // A failed chip takes the focus, so its reason can be read from the keyboard too.
      tabIndex={state.kind === 'failed' ? 0 : undefined}
    >
      <Mark state={state} />
      <span>{tText({ args: {}, key: `${module}.enabled.label` })}</span>
      <VisuallyHidden>{`, ${STATE_WORDS[state.kind]()}`}</VisuallyHidden>
    </span>
  );

  return (
    <li className={styles.item}>
      {state.kind === 'failed' ? <Tooltip content={tText(state.reason)}>{chip}</Tooltip> : chip()}
    </li>
  );
};

const GroupChips = ({ group, items }: InGroup<ModuleProgress>): JSX.Element => {
  const name = tText({ args: {}, key: `core.group.${group}.name` });

  return (
    <div className={styles.group}>
      <p aria-hidden="true" className={styles.groupName}>
        {name}
      </p>
      <ul aria-label={name} className={styles.chips}>
        {items.map(({ module, state }) => (
          <Chip key={module} module={module} state={state} />
        ))}
      </ul>
    </div>
  );
};

// A running scan: how many items it looked at, a bar that shows it is alive (the core reports no total), one chip
// per module of the scan under its group's name, and Stop.
export const ScanProgress = ({ entries, modules, onStop }: ScanProgressProps): JSX.Element => (
  <div className={styles.progress}>
    <div className={styles.head}>
      <p className={styles.looked}>
        {t('ui.scan-progress.looked-at')} <AnimatedNumber format={formatCount} value={entries} />{' '}
        {t('ui.scan-progress.entries', { count: { kind: 'count', value: entries } })}
      </p>
      <Button icon={Square} onClick={onStop} size="small">
        {t('ui.scan-progress.stop')}
      </Button>
    </div>
    <ProgressBar label={t('ui.scan-progress.label')} value={null} />
    <div aria-label={t('ui.scan-progress.modules')} className={styles.groups} role="group">
      {inGroups(modules, ({ group }) => group).map(({ group, items }) => (
        <GroupChips group={group} items={items} key={group} />
      ))}
    </div>
  </div>
);

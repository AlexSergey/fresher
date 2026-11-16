import { type JSX, type ReactNode, useId } from 'react';

import type { ModuleId, ModuleOutcome, Sizes } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { Notice } from '../../design/feedback/notice/notice';
import { Icon } from '../../design/icons/icon';
import { Check, CircleAlert, Minus } from '../../design/icons/icons';
import { Spinner } from '../../design/primitives/spinner/spinner';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './module-section.module.css';

export type ModuleSectionProps = {
  readonly children?: ReactNode;
  // What the module found in this scan.
  readonly found: Sizes;
  readonly module: ModuleId;
  // Still scanning, or how its part of the scan ended.
  readonly state: ModuleOutcome | { readonly kind: 'scanning' };
};

type SectionState = ModuleSectionProps['state'];

const Status = ({ state }: { readonly state: SectionState }): JSX.Element => {
  switch (state.kind) {
    case 'cancelled':
      return (
        <span className={styles.status} data-state={state.kind}>
          <Icon icon={Minus} />
          {t('ui.module-section.stopped')}
        </span>
      );
    case 'completed':
      return (
        <span className={styles.status} data-state={state.kind}>
          <Icon icon={Check} />
          {t('ui.module-section.done')}
        </span>
      );
    case 'failed':
      return (
        <span className={styles.status} data-state={state.kind}>
          <Icon icon={CircleAlert} />
          {t('ui.module-section.failed')}
        </span>
      );
    case 'scanning':
      return (
        <span className={styles.status} data-state={state.kind}>
          <Spinner size="small" />
          {t('ui.module-section.scanning')}
        </span>
      );
    case 'unavailable':
      return (
        <span className={styles.status} data-state={state.kind}>
          <Icon icon={Minus} />
          {t('ui.module-section.unavailable')}
        </span>
      );
  }
};

// A module's part of a page: its name, what it found and how its scan went above its content; a module that failed
// says why, in the core's words.
export const ModuleSection = ({ children, found, module, state }: ModuleSectionProps): JSX.Element => {
  const headingId = useId();

  return (
    <section aria-labelledby={headingId} className={styles.section}>
      <header className={styles.header}>
        <h2 className={styles.name} id={headingId}>
          {tText({ args: {}, key: `${module}.enabled.label` })}
        </h2>
        <SizeText {...honestSize(found)} size="callout" />
        <Status state={state} />
      </header>
      {state.kind === 'failed' ? (
        <Notice title={t('ui.module-section.failed-title')} tone="danger">
          {tText(state.reason)}
        </Notice>
      ) : null}
      {children}
    </section>
  );
};

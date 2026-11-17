import { type JSX, useId, useState } from 'react';

import type { Text } from '../../ipc/bindings.generated';

import { t, tText } from '../../i18n';
import styles from './problem-text.module.css';

export type ProblemTextProps = {
  readonly text: Text;
};

// A problem as the core worded it. The tool's own line, when there is one, is the `detail` arg that no sentence names:
// it opens only on request, below the sentence.
export const ProblemText = ({ text }: ProblemTextProps): JSX.Element => {
  const detailId = useId();
  const [open, setOpen] = useState(false);
  const { detail } = text.args;

  return (
    <span className={styles.problem}>
      <span>{tText(text)}</span>
      {detail?.kind === 'text' ? (
        <>
          <button
            aria-controls={detailId}
            aria-expanded={open}
            className={styles.toggle}
            onClick={() => {
              setOpen(!open);
            }}
            type="button"
          >
            {t('ui.problem.details')}
          </button>
          <code className={styles.detail} hidden={!open} id={detailId}>
            {detail.value}
          </code>
        </>
      ) : null}
    </span>
  );
};

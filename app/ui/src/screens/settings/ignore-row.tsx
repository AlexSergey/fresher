import type { JSX } from 'react';

import { Button } from '../../design/primitives/button/button';
import { Switch } from '../../design/primitives/switch/switch';
import { t } from '../../i18n';
import styles from './settings-screen.module.css';

type IgnoreRowProps = {
  readonly description?: string;
  readonly ignored: boolean;
  // Ignored by something else (its group): the switch stays on and cannot be turned off here, with this note.
  readonly lockedNote?: string;
  readonly name: string;
  readonly onIgnore: () => void;
  readonly onRestore: () => void;
  readonly readOnly: boolean;
};

const text = (value: string): { readonly kind: 'text'; readonly value: string } => ({ kind: 'text', value });

// One group or module with its Ignore switch; an ignored one also shows Restore, which does what turning the switch
// off does.
export const IgnoreRow = ({
  description,
  ignored,
  lockedNote,
  name,
  onIgnore,
  onRestore,
  readOnly,
}: IgnoreRowProps): JSX.Element => {
  const locked = lockedNote !== undefined;

  return (
    <div className={styles.line}>
      <span className={styles.lineText}>
        <span>{name}</span>
        {description === undefined ? null : <span className={styles.note}>{description}</span>}
        {locked ? <span className={styles.note}>{lockedNote}</span> : null}
      </span>
      <span className={styles.lineActions}>
        {ignored && !locked ? (
          <Button disabled={readOnly} onClick={onRestore} size="small" variant="secondary">
            {t('ui.screen.settings.ignored.restore', { name: text(name) })}
          </Button>
        ) : null}
        <Switch
          checked={ignored || locked}
          disabled={readOnly || locked}
          hideLabel
          label={t('ui.screen.settings.ignored.switch', { name: text(name) })}
          onChange={(checked) => {
            if (checked) {
              onIgnore();
            } else {
              onRestore();
            }
          }}
        />
      </span>
    </div>
  );
};

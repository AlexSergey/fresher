import { type JSX, type KeyboardEvent, useId, useRef } from 'react';

import type { ChoiceView, UninstallChoice } from '../../ipc/bindings.generated';

import { SizeText } from '../../design/data/size-text/size-text';
import { Trash2 } from '../../design/icons/icons';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { honestSize } from '../sizes';
import styles from './uninstall-choices.module.css';

export type UninstallChoicesProps = {
  // What `Uninstall…` is asked for; the core's two choices, with their texts and what each takes.
  readonly choices: readonly ChoiceView[];
  readonly onChoose: (choice: UninstallChoice) => void;
  readonly onUninstall: () => void;
  // A plan is being prepared: the button waits.
  readonly preparing: boolean;
  // The chosen one; the page starts with `keep-data`.
  readonly value: UninstallChoice;
};

// The two ways to uninstall an app as radio buttons, each with the core's label, its sentence and what it takes, and the
// button that asks the core for the plan. The button opens the confirmation; it removes nothing.
export const UninstallChoices = ({
  choices,
  onChoose,
  onUninstall,
  preparing,
  value,
}: UninstallChoicesProps): JSX.Element => {
  const buttonsRef = useRef<(HTMLButtonElement | null)[]>([]);

  const baseId = useId();

  // The arrows move the choice and wrap around, Home and End go to the ends, as native radios and a menu do.
  const onKeyDown = (event: KeyboardEvent<HTMLButtonElement>, index: number): void => {
    const last = choices.length - 1;
    const targets: Readonly<Record<string, number>> = {
      ArrowDown: index === last ? 0 : index + 1,
      ArrowLeft: index === 0 ? last : index - 1,
      ArrowRight: index === last ? 0 : index + 1,
      ArrowUp: index === 0 ? last : index - 1,
      End: last,
      Home: 0,
    };
    const at = targets[event.key];
    const target = at === undefined ? undefined : choices[at];

    if (at === undefined || target === undefined) {
      return;
    }
    event.preventDefault();
    if (at !== index) {
      onChoose(target.choice);
      buttonsRef.current[at]?.focus();
    }
  };

  return (
    <div className={styles.choices}>
      <div aria-label={t('ui.product.uninstall-choices.label')} className={styles.options} role="radiogroup">
        {choices.map(({ choice, description, label, sizes }, index) => (
          <button
            aria-checked={choice === value}
            aria-describedby={`${baseId}-${choice}-description ${baseId}-${choice}-size`}
            aria-labelledby={`${baseId}-${choice}-label`}
            className={styles.option}
            key={choice}
            onClick={() => {
              onChoose(choice);
            }}
            onKeyDown={(event) => {
              onKeyDown(event, index);
            }}
            ref={(element) => {
              buttonsRef.current[index] = element;
            }}
            role="radio"
            tabIndex={choice === value ? 0 : -1}
            type="button"
          >
            <span aria-hidden="true" className={styles.dot} />
            <span className={styles.text}>
              <span className={styles.label} id={`${baseId}-${choice}-label`}>
                {tText(label)}
              </span>
              <span className={styles.description} id={`${baseId}-${choice}-description`}>
                {tText(description)}
              </span>
            </span>
            <span className={styles.size} id={`${baseId}-${choice}-size`}>
              <SizeText {...honestSize(sizes)} />
            </span>
          </button>
        ))}
      </div>
      <Button busy={preparing} icon={Trash2} onClick={onUninstall} size="large" variant="primary">
        {t('ui.product.uninstall-choices.button')}
      </Button>
    </div>
  );
};

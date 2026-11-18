import type { JSX } from 'react';

import type { CategoryTotal, ModuleInfo } from '../../ipc/bindings.generated';

import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { useCoreStore } from '../../state/store';
import styles from './module-choices.module.css';

export type ModuleChoicesProps = {
  readonly module: ModuleInfo;
  readonly totals: readonly CategoryTotal[];
};

const textOf = (key: string): string => tText({ args: {}, key });

// The module's choices above its categories: one button each with what it does beside it. A choice selects those of
// its categories that hold a finding, so one with none is disabled. Nothing for a module without choices.
export const ModuleChoices = ({ module, totals }: ModuleChoicesProps): JSX.Element | null => {
  const store = useCoreStore();
  const within = module.categories.map(({ descriptor }) => descriptor.id);

  if (module.choices.length === 0) {
    return null;
  }

  return (
    <div aria-label={t('ui.screen.module.choices')} className={styles.choices} role="group">
      {module.choices.map((choice) => {
        const categories = choice.categories.filter((id) => totals.some((one) => one.category === id && one.count > 0));
        const prefix = `${module.id}.choice.${choice.name}`;

        return (
          <div className={styles.choice} key={choice.name}>
            <Button
              disabled={categories.length === 0}
              onClick={() => {
                store.dispatch({ categories, type: 'choose', within });
              }}
            >
              {textOf(`${prefix}.label`)}
            </Button>
            <p className={styles.description}>{textOf(`${prefix}.description`)}</p>
          </div>
        );
      })}
    </div>
  );
};

import type { JSX } from 'react';

import type { CategoryId, CategoryTotal, ModuleInfo } from '../../ipc/bindings.generated';

import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { useCoreStore } from '../../state/store';
import styles from './module-choices.module.css';

export type ModuleChoicesProps = {
  // The category whose details show the choices: only the choices that touch it are offered.
  readonly category: CategoryId;
  readonly module: ModuleInfo;
  readonly totals: readonly CategoryTotal[];
};

const textOf = (key: string): string => tText({ args: {}, key });

// The module's choices that touch a category: one button each with what it does beside it. A choice selects those of
// its categories that hold a finding, so one with none is disabled. Nothing when no choice touches the category.
export const ModuleChoices = ({ category, module, totals }: ModuleChoicesProps): JSX.Element | null => {
  const store = useCoreStore();
  const within = module.categories.map(({ descriptor }) => descriptor.id);
  const touching = module.choices.filter(({ categories }) => categories.includes(category));

  if (touching.length === 0) {
    return null;
  }

  return (
    <div aria-label={t('ui.screen.category.choices')} className={styles.choices} role="group">
      {touching.map((choice) => {
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

import type { JSX } from 'react';

import type { CategoryId, ModuleId } from '../../ipc/bindings.generated';

import { t } from '../../i18n';
import { FindingList } from '../../product/finding-list/finding-list';
import { useCore, useCoreStore } from '../../state/store';
import { useFindings } from '../../state/use-findings';
import { useRowMenu } from '../use-row-menu';

export type CategoryFindingsProps = {
  readonly category: CategoryId;
  // The home folder the core reports: the items' paths are shown from `~`.
  readonly home: null | string;
  readonly module: ModuleId;
  // The category's name, for the list's label.
  readonly name: string;
};

// The findings of a category, largest first, each with its box and its menu.
export const CategoryFindings = ({ category, home, module, name }: CategoryFindingsProps): JSX.Element => {
  const store = useCoreStore();
  const selection = useCore((state) => state.selection);
  const { menuOf, onReveal } = useRowMenu();
  const { failure, onRangeChange, rowAt, total } = useFindings({ category, module, order: 'largest' });

  return (
    <FindingList
      failure={failure}
      home={home}
      isSelected={(finding) => {
        const excluded = selection.categories.get(finding.category);

        return excluded === undefined ? selection.findings.has(finding.id) : !excluded.has(finding.id);
      }}
      label={t('ui.screen.category.items', { name: { kind: 'text', value: name } })}
      menuOf={menuOf}
      onRangeChange={onRangeChange}
      onReveal={onReveal}
      onToggle={(finding) => {
        store.dispatch({ finding, type: 'toggle-finding' });
      }}
      rowAt={rowAt}
      total={total}
      variant="details"
    />
  );
};

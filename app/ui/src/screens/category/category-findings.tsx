import type { JSX } from 'react';

import type { CategoryId, ModuleId } from '../../ipc/bindings.generated';

import { t } from '../../i18n';
import { failureOf, revealFinding } from '../../ipc';
import { FindingList } from '../../product/finding-list/finding-list';
import { useCore, useCoreStore } from '../../state/store';
import { useFindings } from '../../state/use-findings';

export type CategoryFindingsProps = {
  readonly category: CategoryId;
  readonly module: ModuleId;
  // The category's name, for the list's label.
  readonly name: string;
};

// The findings of a category, largest first, each with its box and Show in Finder.
export const CategoryFindings = ({ category, module, name }: CategoryFindingsProps): JSX.Element => {
  const store = useCoreStore();
  const selection = useCore((state) => state.selection);
  const { failure, onRangeChange, rowAt, total } = useFindings({ category, module, order: 'largest' });

  return (
    <FindingList
      failure={failure}
      isSelected={(finding) => {
        const excluded = selection.categories.get(finding.category);

        return excluded === undefined ? selection.findings.has(finding.id) : !excluded.has(finding.id);
      }}
      label={t('ui.screen.category.items', { name: { kind: 'text', value: name } })}
      onRangeChange={onRangeChange}
      onReveal={(finding) => {
        revealFinding(finding.id).catch((error: unknown) => {
          store.dispatch({ failure: failureOf(error), type: 'command-failed' });
        });
      }}
      onToggle={(finding) => {
        store.dispatch({ finding, type: 'toggle-finding' });
      }}
      rowAt={rowAt}
      total={total}
    />
  );
};

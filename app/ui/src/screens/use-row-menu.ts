import { useMemo } from 'react';

import type { Finding } from '../ipc/bindings.generated';

import { type MenuItem, SEPARATOR } from '../design/primitives/menu/menu';
import { t, tText } from '../i18n';
import { failureOf, revealFinding } from '../ipc';
import { useCoreStore } from '../state/store';
import { useIgnoreCommands } from './settings/use-ignore-commands';

type RowMenu = {
  readonly menuOf: (finding: Finding) => readonly MenuItem[];
  readonly onReveal: (finding: Finding) => void;
};

const text = (value: string): { readonly kind: 'text'; readonly value: string } => ({ kind: 'text', value });

// The one menu of a finding's row, on the details and on Review: Show in Finder, then Ignore this item and Ignore all
// of its category. A module is turned off only in Settings. A finding that is not a path (an object of an external tool) has
// no Finder place and no path to ignore: its menu holds only the category item.
export const useRowMenu = (): RowMenu => {
  const store = useCoreStore();
  const commands = useIgnoreCommands();

  return useMemo(() => {
    const onReveal = (finding: Finding): void => {
      revealFinding(finding.id).catch((error: unknown) => {
        store.dispatch({ failure: failureOf(error), type: 'command-failed' });
      });
    };
    const menuOf = (finding: Finding): readonly MenuItem[] => {
      const isPath = finding.subject.kind === 'path';
      const ignoreCategory: MenuItem = {
        key: 'ignore-category',
        label: t('ui.finding-list.ignore-category', {
          category: text(tText({ args: {}, key: `${finding.category}.name` })),
        }),
        onSelect: (): void => {
          commands.ignore({ category: finding.category, kind: 'category' });
        },
      };

      return isPath
        ? [
            {
              key: 'reveal',
              label: t('ui.finding-list.reveal'),
              onSelect: (): void => {
                onReveal(finding);
              },
            },
            SEPARATOR,
            {
              key: 'ignore',
              label: t('ui.finding-list.ignore'),
              onSelect: (): void => {
                commands.ignore({ finding: finding.id, kind: 'finding' });
              },
            },
            ignoreCategory,
          ]
        : [ignoreCategory];
    };

    return { menuOf, onReveal };
  }, [commands, store]);
};

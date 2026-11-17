import type { JSX } from 'react';

import type { IgnoredEntry } from '../../ipc/bindings.generated';

import { PathText } from '../../design/data/path-text/path-text';
import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import styles from './settings-screen.module.css';
import { useIgnoreCommands } from './use-ignore-commands';

type IgnoredItemsProps = {
  readonly categories: readonly string[];
  readonly paths: readonly string[];
  readonly readOnly: boolean;
};

// Plan 005's lists: kinds of items by their name and single items by their path, each with Restore.
export const IgnoredItems = ({ categories, paths, readOnly }: IgnoredItemsProps): JSX.Element => {
  const commands = useIgnoreCommands();
  const restoreButton = (entry: IgnoredEntry, name: string): JSX.Element => (
    <Button
      disabled={readOnly}
      onClick={() => {
        commands.restore(entry);
      }}
      size="small"
      variant="secondary"
    >
      {t('ui.screen.settings.ignored.restore', { name: { kind: 'text', value: name } })}
    </Button>
  );

  return (
    <Surface elevation={1} padding="none">
      <div className={styles.rows}>
        {categories.length === 0 && paths.length === 0 ? (
          <p className={styles.line}>{t('ui.screen.settings.ignored.empty')}</p>
        ) : null}
        {categories.map((category) => {
          const name = tText({ args: {}, key: `${category}.name` });

          return (
            <div className={styles.line} key={category}>
              <span>{name}</span>
              {restoreButton({ category, kind: 'category' }, name)}
            </div>
          );
        })}
        {paths.map((path) => (
          <div className={styles.line} key={path}>
            <PathText path={path} />
            {restoreButton({ kind: 'path', path }, path)}
          </div>
        ))}
      </div>
    </Surface>
  );
};

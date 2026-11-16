import type { JSX } from 'react';

import type { IgnoredEntry, SettingsView } from '../../ipc/bindings.generated';

import { PathText } from '../../design/data/path-text/path-text';
import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { failureOf, restoreIgnored } from '../../ipc';
import { useCoreStore } from '../../state/store';
import styles from './settings-screen.module.css';

type IgnoredGroupProps = {
  readonly headingId: string;
  readonly readOnly: boolean;
  readonly view: SettingsView;
};

// Everything the user ignores, kinds of items by their name and single items by their path, each with Restore.
export const IgnoredGroup = ({ headingId, readOnly, view }: IgnoredGroupProps): JSX.Element => {
  const store = useCoreStore();
  const categories = listOf(view, 'core.ignore.categories');
  const paths = listOf(view, 'core.ignore.paths');
  const restore = (entry: IgnoredEntry): void => {
    restoreIgnored(entry).catch((error: unknown) => {
      store.dispatch({ failure: failureOf(error), type: 'command-failed' });
    });
  };
  const restoreButton = (entry: IgnoredEntry, name: string): JSX.Element => (
    <Button
      disabled={readOnly}
      onClick={() => {
        restore(entry);
      }}
      size="small"
      variant="secondary"
    >
      {t('ui.screen.settings.ignored.restore', { name: { kind: 'text', value: name } })}
    </Button>
  );

  return (
    <section aria-labelledby={headingId} className={styles.group}>
      <h2 className={styles.heading} id={headingId}>
        {t('ui.screen.settings.group.ignored')}
      </h2>
      <p className={styles.groupText}>{t('ui.screen.settings.ignored.text')}</p>
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
    </section>
  );
};

// The entries of one ignore list, as the core keeps them.
const listOf = (view: SettingsView, key: string): readonly string[] => {
  const value = view.values[key];

  return value?.kind === 'ids' || value?.kind === 'paths' ? value.value : [];
};

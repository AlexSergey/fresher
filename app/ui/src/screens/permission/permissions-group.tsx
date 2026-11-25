import type { JSX } from 'react';

import type { SettingsView } from '../../ipc/bindings.generated';
import type { Settings } from '../settings/use-settings';

import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { failureOf, openAccessSettings, refreshAccess, relaunch } from '../../ipc';
import { SettingsRow } from '../../product/settings-row/settings-row';
import { useCoreStore } from '../../state/store';
import { isReadOnly } from '../settings/file-notice';
import screenStyles from '../settings/settings-screen.module.css';
import { useModules } from '../use-modules';
import { PermissionRows } from './permission-rows';
import styles from './permissions-group.module.css';
import { useAccessStatus } from './use-access';

// The one place that lists every permission: each capability's state, what it unlocks and the action that grants it,
// the four opt-in switches (the core's `core.access.*` settings) and a way to check again. The heading and the switches
// show at once; the states once the core has answered.
export const PermissionsGroup = ({
  headingId,
  settings,
  view,
}: {
  // The page's heading, which names the group.
  readonly headingId: string;
  readonly settings: Settings;
  readonly view: SettingsView;
}): JSX.Element => {
  const store = useCoreStore();
  const status = useAccessStatus();
  const modules = useModules();
  const readOnly = isReadOnly(view.file);
  const failed = (error: unknown): void => {
    store.dispatch({ failure: failureOf(error), type: 'command-failed' });
  };
  // The opt-in switch of the setting `key`: the typed setting row.
  const switchRow = (key: string): JSX.Element | null => {
    const descriptor = view.settings.find((setting) => setting.key === key);
    const refusal = settings.refusals[key];

    return descriptor === undefined ? null : (
      <SettingsRow
        descriptor={descriptor}
        onChange={(value) => {
          settings.change(key, value);
        }}
        problem={view.problems.find((problem) => problem.key === key) ?? null}
        readOnly={readOnly}
        {...(refusal === undefined ? {} : { refusal })}
        value={view.values[key] ?? descriptor.default}
      />
    );
  };

  return (
    <section aria-labelledby={headingId} className={screenStyles.group}>
      <div className={styles.title}>
        <Button
          onClick={() => {
            refreshAccess().catch(failed);
          }}
          size="small"
          variant="ghost"
        >
          {t('ui.permissions.check-again')}
        </Button>
      </div>
      <Surface elevation={1} padding="none">
        <div className={screenStyles.rows}>
          {status === null ? (
            // The switches (the core's settings of this group) before the core has told the states.
            view.settings
              .filter((setting) => setting.group === 'permissions')
              .map((setting) => <div key={setting.key}>{switchRow(setting.key)}</div>)
          ) : (
            <PermissionRows
              modules={modules ?? []}
              onOpenSettings={(capability) => {
                openAccessSettings(capability).catch(failed);
              }}
              onReopen={() => {
                // The promise never settles: the app restarts.
                relaunch().catch(failed);
              }}
              states={status.capabilities}
              switchOf={(id) => switchRow(`core.access.${id}`)}
            />
          )}
        </div>
      </Surface>
    </section>
  );
};

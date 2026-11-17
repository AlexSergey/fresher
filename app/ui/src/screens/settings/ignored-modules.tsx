import type { JSX } from 'react';

import type { ModuleInfo } from '../../ipc/bindings.generated';

import { Surface } from '../../design/layout/surface/surface';
import { t, tText } from '../../i18n';
import { modulesByGroup } from '../../product/module-groups';
import { IgnoreRow } from './ignore-row';
import styles from './settings-screen.module.css';
import { useIgnoreCommands } from './use-ignore-commands';

type IgnoredModulesProps = {
  readonly ignoredGroups: readonly string[];
  // The modules now ignored on their own, as the settings hold them.
  readonly ignoredModules: readonly string[];
  readonly modules: readonly ModuleInfo[];
  readonly readOnly: boolean;
};

// Every module under its group's name, each with its Ignore switch; a module of an ignored group is locked on. An
// ignored module nobody registers any more is listed last, so that it can still be restored.
export const IgnoredModules = ({
  ignoredGroups,
  ignoredModules,
  modules,
  readOnly,
}: IgnoredModulesProps): JSX.Element => {
  const commands = useIgnoreCommands();
  const unknown = ignoredModules.filter((id) => !modules.some((module) => module.id === id));
  const row = (id: string, name: string, groupName: null | string): JSX.Element => (
    <IgnoreRow
      ignored={ignoredModules.includes(id)}
      key={id}
      name={name}
      onIgnore={() => {
        commands.ignore({ kind: 'module', module: id });
      }}
      onRestore={() => {
        commands.restore({ kind: 'module', module: id });
      }}
      readOnly={readOnly}
      {...(groupName === null
        ? {}
        : { lockedNote: t('ui.screen.settings.ignored.with-group', { group: { kind: 'text', value: groupName } }) })}
    />
  );

  return (
    <>
      <p className={styles.groupText}>{tText({ args: {}, key: 'core.ignore.modules.description' })}</p>
      {modulesByGroup(modules).map(({ group, items }) => {
        const groupName = tText({ args: {}, key: `core.group.${group}.name` });

        return (
          <section aria-label={groupName} className={styles.group} key={group}>
            <h3 className={styles.moduleHeading}>{groupName}</h3>
            <Surface elevation={1} padding="none">
              <div className={styles.rows}>
                {items.map((module) =>
                  row(
                    module.id,
                    tText({ args: {}, key: `${module.id}.enabled.label` }),
                    ignoredGroups.includes(group) ? groupName : null,
                  ),
                )}
              </div>
            </Surface>
          </section>
        );
      })}
      {unknown.length === 0 ? null : (
        <Surface elevation={1} padding="none">
          <div className={styles.rows}>{unknown.map((id) => row(id, id, null))}</div>
        </Surface>
      )}
    </>
  );
};

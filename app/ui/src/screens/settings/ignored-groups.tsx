import type { JSX } from 'react';

import type { ModuleGroup, ModuleInfo } from '../../ipc/bindings.generated';

import { Surface } from '../../design/layout/surface/surface';
import { tText } from '../../i18n';
import { modulesByGroup } from '../../product/module-groups';
import { IgnoreRow } from './ignore-row';
import styles from './settings-screen.module.css';
import { useIgnoreCommands } from './use-ignore-commands';

type IgnoredGroupsProps = {
  // The groups now ignored, as the settings hold them.
  readonly ignored: readonly string[];
  readonly modules: readonly ModuleInfo[];
  readonly readOnly: boolean;
};

// Every group that has a module, and every ignored one, each with its Ignore switch.
export const IgnoredGroups = ({ ignored, modules, readOnly }: IgnoredGroupsProps): JSX.Element => {
  const commands = useIgnoreCommands();
  // The core stores group ids as text, and a group a newer core adds may have no texts here: the cast names what a
  // module's group is.
  const listed = modulesByGroup(modules).map(({ group }) => group as ModuleGroup);
  const unknown = ignored.filter((group) => !listed.some((known) => known === group));

  return (
    <>
      <p className={styles.groupText}>{tText({ args: {}, key: 'core.ignore.groups.description' })}</p>
      <Surface elevation={1} padding="none">
        <div className={styles.rows}>
          {listed.map((group) => (
            <GroupRow
              commands={commands}
              group={group}
              ignored={ignored.includes(group)}
              key={group}
              readOnly={readOnly}
            />
          ))}
          {unknown.map((group) => (
            <GroupRow commands={commands} group={group as ModuleGroup} ignored key={group} readOnly={readOnly} />
          ))}
        </div>
      </Surface>
    </>
  );
};

type GroupRowProps = {
  readonly commands: ReturnType<typeof useIgnoreCommands>;
  readonly group: ModuleGroup;
  readonly ignored: boolean;
  readonly readOnly: boolean;
};

const GroupRow = ({ commands, group, ignored, readOnly }: GroupRowProps): JSX.Element => (
  <IgnoreRow
    description={tText({ args: {}, key: `core.group.${group}.description` })}
    ignored={ignored}
    name={tText({ args: {}, key: `core.group.${group}.name` })}
    onIgnore={() => {
      commands.ignore({ group, kind: 'group' });
    }}
    onRestore={() => {
      commands.restore({ group, kind: 'group' });
    }}
    readOnly={readOnly}
  />
);

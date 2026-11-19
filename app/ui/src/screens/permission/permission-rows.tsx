import { type JSX, type ReactNode, useState } from 'react';

import type { Access, Capability, CapabilityState, ModuleInfo } from '../../ipc/bindings.generated';

import { Icon } from '../../design/icons/icon';
import { CircleAlert, CircleCheck, Info } from '../../design/icons/icons';
import { Button } from '../../design/primitives/button/button';
import { t, tText } from '../../i18n';
import { idOf, isOptIn } from './capability-id';
import styles from './permission-rows.module.css';

export type PermissionRowsProps = {
  readonly modules: readonly ModuleInfo[];
  readonly onOpenSettings: (capability: Capability) => void;
  readonly onReopen: () => void;
  // The capabilities in the core's order.
  readonly states: readonly CapabilityState[];
  // The opt-in switch of the capability with this id: the typed setting row.
  readonly switchOf: (id: string) => ReactNode;
};

const ICONS = { denied: CircleAlert, granted: CircleCheck, 'not-applicable': Info, 'not-asked': Info, unknown: Info };

const text = (id: string, part: string): string => tText({ args: {}, key: `core.access.${id}.${part}` });

const sameCapability = (a: Capability, b: Capability): boolean =>
  a.kind === b.kind && ('value' in a ? a.value : null) === ('value' in b ? b.value : null);

// The state as a word and an icon, never colour alone.
const State = ({ access }: { readonly access: Access }): JSX.Element => (
  <span className={styles.state} data-access={access}>
    <Icon icon={ICONS[access]} />
    {tText({ args: {}, key: `core.access.state.${access}` })}
  </span>
);

// The modules whose categories need the capability.
const modulesNeeding = (modules: readonly ModuleInfo[], capability: Capability): readonly ModuleInfo[] =>
  modules.filter((module) =>
    module.categories.some(({ descriptor }) =>
      descriptor.needs.some((need) => sameCapability(need.capability, capability)),
    ),
  );

type RowProps = {
  readonly access: Access;
  readonly children?: ReactNode;
  readonly description: ReactNode;
  readonly name: string;
};

const Row = ({ access, children, description, name }: RowProps): JSX.Element => (
  <li className={styles.row}>
    <div className={styles.head}>
      <div className={styles.text}>
        <span className={styles.name}>{name}</span>
        {description === null ? null : <p className={styles.description}>{description}</p>}
      </div>
      <State access={access} />
    </div>
    {children}
  </li>
);

const FullDiskAccess = ({
  access,
  onOpenSettings,
  onReopen,
}: Pick<PermissionRowsProps, 'onOpenSettings' | 'onReopen'> & { readonly access: Access }): JSX.Element => {
  const [opened, setOpened] = useState(false);

  return (
    <Row access={access} description={text('all-files', 'unlocks')} name={text('all-files', 'name')}>
      {access === 'granted' ? null : (
        <div className={styles.actions}>
          <Button
            onClick={() => {
              setOpened(true);
              onOpenSettings({ kind: 'all-files' });
            }}
            size="small"
            variant="secondary"
          >
            {t('ui.coverage.open-settings')}
          </Button>
          {opened ? (
            <>
              <Button onClick={onReopen} size="small" variant="secondary">
                {t('ui.coverage.reopen')}
              </Button>
              <span className={styles.note}>{t('ui.coverage.reopen.text')}</span>
            </>
          ) : null}
        </div>
      )}
    </Row>
  );
};

// Every capability of the core with its state, what it unlocks and the one action that grants it. A capability that
// is not needed on this Mac is left out, a granted one has no action.
export const PermissionRows = ({
  modules,
  onOpenSettings,
  onReopen,
  states,
  switchOf,
}: PermissionRowsProps): JSX.Element => (
  <ul className={styles.rows}>
    {states
      .filter(({ access }) => access !== 'not-applicable')
      .map(({ access, capability, note }) => {
        const id = idOf(capability);

        if (capability.kind === 'tool') {
          return modulesNeeding(modules, capability).map((module) => (
            <Row
              access={access}
              description={note === null ? null : tText(note)}
              key={`${capability.value}/${module.id}`}
              name={tText({ args: {}, key: `${module.id}.enabled.label` })}
            />
          ));
        }
        if (id === null) {
          return null;
        }
        if (capability.kind === 'all-files') {
          return <FullDiskAccess access={access} key={id} onOpenSettings={onOpenSettings} onReopen={onReopen} />;
        }

        return (
          <Row access={access} description={text(id, 'unlocks')} key={id} name={text(id, 'name')}>
            {isOptIn(capability) ? <div className={styles.switch}>{switchOf(id)}</div> : null}
            {access === 'denied' ? (
              <div className={styles.actions}>
                <Button
                  onClick={() => {
                    onOpenSettings(capability);
                  }}
                  size="small"
                  variant="secondary"
                >
                  {t('ui.coverage.open-settings')}
                </Button>
                <span className={styles.note}>{text(id, 'denied')}</span>
              </div>
            ) : isOptIn(capability) || access === 'granted' ? null : (
              <p className={styles.note}>{text(id, 'how')}</p>
            )}
          </Row>
        );
      })}
  </ul>
);

import { type JSX, useState } from 'react';

import { Notice } from '../../design/feedback/notice/notice';
import { t, tText } from '../../i18n';
import { failureOf, openAccessSettings, relaunch, updateSetting } from '../../ipc';
import { PermissionCard } from '../../product/permission-card/permission-card';
import { type NotSeen } from '../../state/access';
import { useCore, useCoreStore } from '../../state/store';
import { useScanActions } from '../use-scan-actions';
import { isOptIn } from './capability-id';

export type CoverageNoticeProps = {
  // The capability's state now: `granted` means the result on screen is out of date.
  readonly current: NotSeen['access'];
  readonly entry: NotSeen;
  // The core's id of the capability (`core.access.<id>`).
  readonly id: string;
};

const text = (id: string, part: string): string => tText({ args: {}, key: `core.access.${id}.${part}` });

// "1. Open it. 2. Turn it on." as the steps `['Open it.', 'Turn it on.']`.
const stepsOf = (how: string): readonly string[] => how.split(/\s*\d+\.\s+/u).filter((step) => step !== '');

// One capability that kept the scan from seeing something: what was not seen, what the capability unlocks and the one
// action that fixes it. Never names a path.
export const CoverageNotice = ({ current, entry, id }: CoverageNoticeProps): JSX.Element => {
  const store = useCoreStore();
  const { scan } = useScanActions();
  const [opened, setOpened] = useState(false);
  const key = `core.access.${id}`;
  const switchedOn = useCore((state) => {
    const value = state.settings?.values[key];

    return value?.kind === 'switch' && value.value;
  });
  const { capability, folders } = entry;
  const failed = (error: unknown): void => {
    store.dispatch({ failure: failureOf(error), type: 'command-failed' });
  };
  const lines = (
    <>
      {folders > 0 ? <p>{t('ui.coverage.unreadable', { count: { kind: 'count', value: folders } })}</p> : null}
      <p>{text(id, 'missing')}</p>
      <p>{text(id, 'unlocks')}</p>
    </>
  );
  const name = text(id, 'name');

  if (current === 'granted') {
    return (
      <Notice action={{ label: t('ui.screen.scan.scan-again'), onClick: scan }} title={name} tone="info">
        {lines}
      </Notice>
    );
  }

  if (capability.kind === 'all-files') {
    return (
      <PermissionCard
        access={current === 'unknown' ? 'unknown' : 'denied'}
        how={stepsOf(text(id, 'how'))}
        onOpenSettings={() => {
          setOpened(true);
          openAccessSettings(capability).catch(failed);
        }}
        {...(opened
          ? {
              onReopen: (): void => {
                // The promise never settles: the app restarts.
                relaunch().catch(failed);
              },
            }
          : {})}
      >
        {lines}
      </PermissionCard>
    );
  }

  // macOS refused a place the user opted into (a switch, or a volume as a scan root): only System Settings grants it.
  const refused = current === 'denied' && (switchedOn || !isOptIn(capability));

  if (refused) {
    return (
      <Notice
        action={{
          label: t('ui.coverage.open-settings'),
          onClick: (): void => {
            openAccessSettings(capability).catch(failed);
          },
        }}
        title={name}
        tone="info"
      >
        {lines}
        <p>{text(id, 'denied')}</p>
      </Notice>
    );
  }

  if (isOptIn(capability)) {
    return (
      <Notice
        {...(switchedOn
          ? {}
          : {
              action: {
                label: t('ui.coverage.opt-in'),
                onClick: (): void => {
                  updateSetting({ key, value: { kind: 'switch', value: true } }).catch(failed);
                },
              },
            })}
        title={name}
        tone="info"
      >
        {lines}
        <p>{switchedOn ? t('ui.coverage.scan-again-to-look') : text(id, 'description')}</p>
      </Notice>
    );
  }

  return (
    <Notice title={name} tone="info">
      {lines}
      <p>{text(id, 'how')}</p>
    </Notice>
  );
};

import type { JSX } from 'react';

import type { Text } from '../../ipc/bindings.generated';

import { Notice } from '../../design/feedback/notice/notice';
import { t, tText } from '../../i18n';
import { failureOf, openAccessSettings } from '../../ipc';
import { accessOf } from '../../state/access';
import { useCoreStore } from '../../state/store';
import { useAccessStatus } from '../permission/use-access';

const MANAGEMENT = { kind: 'app-management' } as const;

// The core's line about the permission an uninstall may need. Refused by macOS, it is a warning with the button that
// opens System Settings at the pane (the core picks it); before that it only informs.
export const AppManagementNotice = ({ notice }: { readonly notice: Text }): JSX.Element => {
  const store = useCoreStore();
  const denied = accessOf(useAccessStatus(), MANAGEMENT) === 'denied';

  return (
    <Notice
      {...(denied
        ? {
            action: {
              label: t('ui.coverage.open-settings'),
              onClick: (): void => {
                openAccessSettings(MANAGEMENT).catch((error: unknown) => {
                  store.dispatch({ failure: failureOf(error), type: 'command-failed' });
                });
              },
            },
          }
        : {})}
      title={tText({ args: {}, key: 'core.access.app-management.name' })}
      tone={denied ? 'warning' : 'info'}
    >
      <p>{tText(notice)}</p>
    </Notice>
  );
};

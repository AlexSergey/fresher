import { type JSX, type ReactNode, useEffect, useState } from 'react';

import type { ModuleInfo } from '../ipc/bindings.generated';

import { Announcer } from '../design/feedback/announcer/announcer';
import { Toaster } from '../design/feedback/toast/toaster';
import { t } from '../i18n';
import { failureOf, getFindings, getModules } from '../ipc';
import { systemClock } from '../state/clock';
import { connectCore } from '../state/connect';
import { createFindingsSource } from '../state/findings';
import { createNavigation, type Navigation, NavigationContext } from '../state/navigation';
import { CoreContext, type CoreStore } from '../state/store';
import { FindingsContext } from '../state/use-findings';
import { ModulesContext } from './use-modules';

export type AppProvidersProps = {
  readonly children: ReactNode;
  // The screen to show; a new one, at the overview, when not given (a test passes its own to read where it went).
  readonly navigation?: Navigation;
  readonly store: CoreStore;
};

// The finding lists keep at most 10 pages of 100 findings in all (plan 003/05).
const PAGE_SIZE = 100;
const MAX_PAGES = 10;

// Everything a screen reads, around the app: the core's store (connected to the core once), the finding pages, the
// navigation, the modules the core lists, the toasts and the one polite announcer.
export const AppProviders = ({ children, navigation, store }: AppProvidersProps): JSX.Element => {
  const [ownNavigation] = useState(createNavigation);
  const [findings] = useState(() =>
    createFindingsSource(getFindings, { clock: systemClock, maxPages: MAX_PAGES, pageSize: PAGE_SIZE }),
  );
  const [modules, setModules] = useState<null | readonly ModuleInfo[]>(null);

  useEffect(() => {
    connectCore(store);
  }, [store]);

  useEffect(() => findings.follow(store), [findings, store]);

  useEffect(() => {
    let current = true;

    getModules().then(
      (listed) => {
        if (current) {
          setModules(listed);
        }
      },
      (error: unknown) => {
        if (current) {
          store.dispatch({ failure: failureOf(error), type: 'command-failed' });
        }
      },
    );

    return (): void => {
      current = false;
    };
  }, [store]);

  return (
    <CoreContext value={store}>
      <NavigationContext value={navigation ?? ownNavigation}>
        <FindingsContext value={findings}>
          <ModulesContext value={modules}>
            <Toaster closeLabel={t('ui.screen.frame.close')} label={t('ui.screen.frame.toasts')}>
              <Announcer>{children}</Announcer>
            </Toaster>
          </ModulesContext>
        </FindingsContext>
      </NavigationContext>
    </CoreContext>
  );
};

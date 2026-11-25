import { type JSX, type ReactNode, useEffect, useState } from 'react';

import type { ModuleInfo } from '../ipc/bindings.generated';

import { Announcer } from '../design/feedback/announcer/announcer';
import { Toaster } from '../design/feedback/toast/toaster';
import { t } from '../i18n';
import { failureOf, getAppInfo, getFindings, getModules } from '../ipc';
import { systemClock } from '../state/clock';
import { connectCore } from '../state/connect';
import { createFindingsSource } from '../state/findings';
import { createNavigation, type Navigation, NavigationContext } from '../state/navigation';
import { CoreContext, type CoreStore } from '../state/store';
import { FindingsContext } from '../state/use-findings';
import { systemWallClock, type WallClock, WallClockContext, wallClockOf } from '../state/wall-clock';
import { DemoPathForm } from './demo-path-form';
import { ModulesContext } from './use-modules';

export type AppProvidersProps = {
  readonly children: ReactNode;
  // The screen to show; a new one, at the overview, when not given (a test passes its own to read where it went).
  readonly navigation?: Navigation;
  readonly store: CoreStore;
  // The time of day; the core's when not given (a test fixes it).
  readonly wallClock?: WallClock;
};

// The finding lists keep at most 10 pages of 100 findings in all (plan 003/05).
const PAGE_SIZE = 100;
const MAX_PAGES = 10;

// Everything a screen reads, around the app: the core's store (connected to the core once), the finding pages, the
// navigation, the modules the core lists, the time of day the core counts on, the toasts and the one polite announcer.
export const AppProviders = ({ children, navigation, store, wallClock }: AppProvidersProps): JSX.Element => {
  const [ownNavigation] = useState(createNavigation);
  const [findings] = useState(() =>
    createFindingsSource(getFindings, { clock: systemClock, maxPages: MAX_PAGES, pageSize: PAGE_SIZE }),
  );
  const [modules, setModules] = useState<null | readonly ModuleInfo[]>(null);
  const [coreClock, setCoreClock] = useState<WallClock>(systemWallClock);

  useEffect(() => {
    connectCore(store);
  }, [store]);

  useEffect(() => findings.follow(store), [findings, store]);

  // The core's clock, asked once: until it answers (or when it refuses), the page's own.
  useEffect(() => {
    if (wallClock !== undefined) {
      return;
    }
    let current = true;

    getAppInfo().then(
      (info) => {
        if (current) {
          setCoreClock(wallClockOf(info, Date.now()));
        }
      },
      () => undefined,
    );

    return (): void => {
      current = false;
    };
  }, [wallClock]);

  // The modules carry each category's availability, worked out by the core from the access it has at the moment of the
  // call: listed at start and again whenever the access changes. One request at a time; a change that comes while one
  // is out asks once more after it. A failed refresh keeps the last good list.
  useEffect(() => {
    let current = true;
    let loading = false;
    let again = false;
    let first = true;
    let access = store.getState().access;

    const load = (): void => {
      if (loading) {
        again = true;

        return;
      }
      loading = true;
      getModules()
        .then(
          (listed) => {
            if (current) {
              setModules(listed);
            }
          },
          (error: unknown) => {
            if (current && first) {
              store.dispatch({ failure: failureOf(error), type: 'command-failed' });
            }
          },
        )
        .finally(() => {
          loading = false;
          first = false;
          if (current && again) {
            again = false;
            load();
          }
        });
    };

    load();
    const unsubscribe = store.subscribe(() => {
      const next = store.getState().access;

      if (next !== access) {
        access = next;
        // The first answer of the status is no change of the access, but the modules were listed without it.
        load();
      }
    });

    return (): void => {
      current = false;
      unsubscribe();
    };
  }, [store]);

  return (
    <CoreContext value={store}>
      <NavigationContext value={navigation ?? ownNavigation}>
        <FindingsContext value={findings}>
          <ModulesContext value={modules}>
            <WallClockContext value={wallClock ?? coreClock}>
              <Toaster closeLabel={t('ui.screen.frame.close')} label={t('ui.screen.frame.toasts')}>
                <Announcer>
                  <DemoPathForm>{children}</DemoPathForm>
                </Announcer>
              </Toaster>
            </WallClockContext>
          </ModulesContext>
        </FindingsContext>
      </NavigationContext>
    </CoreContext>
  );
};

import { type JSX, type ReactNode, useEffect } from 'react';

import type { ModuleInfo } from '../ipc/bindings.generated';

import { ErrorState } from '../design/feedback/error-state/error-state';
import { useToasts } from '../design/feedback/toast/toaster';
import { AppShell } from '../design/layout/app-shell/app-shell';
import { ScrollArea } from '../design/layout/scroll-area/scroll-area';
import { t, tText } from '../i18n';
import { customViews } from '../modules/registry';
import { BuildBanner } from '../product/build-banner/build-banner';
import { type Screen, useNavigation, useScreen } from '../state/navigation';
import { useCore } from '../state/store';
import styles from './app-frame.module.css';
import { CleanupScreen } from './cleanups/cleanup-screen';
import { CleanupsScreen } from './cleanups/cleanups-screen';
import { FrameSidebar } from './frame-sidebar';
import { ModuleScreen } from './module/module-screen';
import { OverviewScreen } from './overview/overview-screen';
import { QuitQuestion } from './quit-question';
import { ReviewScreen } from './review/review-screen';
import { screenKey, ScreenStack } from './screen-stack';
import { SettingsScreen } from './settings/settings-screen';
import { useAppInfo } from './use-app-info';
import { useModules } from './use-modules';
import { useMotionSetting } from './use-motion-setting';

const moduleName = (module: ModuleInfo): string => tText({ args: {}, key: `${module.id}.enabled.label` });

// The screen's content and its accessible name; `null` for a module the core has not listed yet.
const contentOf = (
  screen: Screen,
  modules: null | readonly ModuleInfo[],
): null | { readonly content: ReactNode; readonly label: string } => {
  switch (screen.kind) {
    case 'cleanup':
      return { content: <CleanupScreen cleanup={screen.cleanup} />, label: t('ui.screen.frame.nav.cleanups') };
    case 'cleanups':
      return { content: <CleanupsScreen />, label: t('ui.screen.frame.nav.cleanups') };
    case 'module': {
      const module = modules?.find((one) => one.id === screen.module);

      if (module === undefined) {
        return null;
      }

      const View = customViews[module.id] ?? ModuleScreen;

      return { content: <View category={screen.category} module={module} />, label: moduleName(module) };
    }
    case 'overview':
      return { content: <OverviewScreen />, label: t('ui.screen.frame.nav.overview') };
    case 'review':
      return { content: <ReviewScreen />, label: t('ui.screen.frame.nav.review') };
    case 'settings':
      return { content: <SettingsScreen />, label: t('ui.screen.frame.nav.settings') };
  }
};

// A refused command anywhere becomes an error toast with the core's text, once per refusal.
const useRefusalToasts = (): void => {
  const failure = useCore((state) => state.failure);
  const { show } = useToasts();

  useEffect(() => {
    if (failure?.kind === 'refused') {
      show({ text: tText(failure.refusal.text), tone: 'error' });
    }
  }, [failure, show]);
};

// The app's frame: the sidebar beside the current screen. The screens render once the core's dictionary is read, so
// every text of theirs is the core's from the first frame; an unreachable core shows its error state instead.
export const AppFrame = (): JSX.Element => {
  useMotionSetting();
  const navigation = useNavigation();
  const modules = useModules();
  const info = useAppInfo();
  const screen = useScreen(modules?.map((module) => module.id) ?? null);
  const loaded = useCore((state) => state.dictionary !== null);
  const unreachable = useCore((state) => state.failure?.kind === 'unreachable');

  useRefusalToasts();

  const renderScreen = (shown: Screen): ReactNode => {
    const content = contentOf(shown, modules);

    return content === null ? null : (
      <ScrollArea label={content.label} screenKey={screenKey(shown)}>
        {content.content}
      </ScrollArea>
    );
  };

  return (
    <div className={styles.window}>
      {info === null ? null : <BuildBanner build={info.build} />}
      <div className={styles.body}>
        <AppShell sidebar={<FrameSidebar modules={modules} onNavigate={navigation.navigate} screen={screen} />}>
          {unreachable ? (
            <div className={styles.unreachable}>
              <ErrorState
                action={{
                  label: t('ui.screen.frame.unreachable.retry'),
                  onClick: () => {
                    window.location.reload();
                  },
                }}
                text={t('ui.core.unreachable')}
                title={t('ui.screen.frame.unreachable.title')}
              />
            </div>
          ) : loaded ? (
            <ScreenStack render={renderScreen} screen={screen} />
          ) : null}
          {loaded ? <QuitQuestion /> : null}
        </AppShell>
      </div>
    </div>
  );
};

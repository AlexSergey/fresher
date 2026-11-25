import { type JSX, type ReactNode, useEffect } from 'react';

import type { Insight, ModuleInfo } from '../ipc/bindings.generated';

import { ErrorState } from '../design/feedback/error-state/error-state';
import { useToasts } from '../design/feedback/toast/toaster';
import { AppShell } from '../design/layout/app-shell/app-shell';
import { ScrollArea } from '../design/layout/scroll-area/scroll-area';
import { t, tText } from '../i18n';
import { BuildBanner } from '../product/build-banner/build-banner';
import { type Screen, useNavigation, useScreen } from '../state/navigation';
import { useCore } from '../state/store';
import { AgreementGate } from './agreement-gate';
import styles from './app-frame.module.css';
import { AppScreen } from './apps/app-screen';
import { AppsScreen } from './apps/apps-screen';
import { CategoryScreen } from './category/category-screen';
import { CleanupScreen } from './cleanups/cleanup-screen';
import { CleanupsScreen } from './cleanups/cleanups-screen';
import { FrameSidebar } from './frame-sidebar';
import { InsightScreen } from './insight/insight-screen';
import { LargestScreen } from './largest/largest-screen';
import { QuitQuestion } from './quit-question';
import { ReviewScreen } from './review/review-screen';
import { ScanScreen } from './scan/scan-screen';
import { screenKey, ScreenStack } from './screen-stack';
import { SettingsScreen } from './settings/settings-screen';
import { useAppInfo } from './use-app-info';
import { moduleOfCategory, useModules } from './use-modules';
import { useMotionSetting } from './use-motion-setting';

// The screen's content and its accessible name; `null` for a category's details while the modules are not known.
const contentOf = (
  screen: Screen,
  modules: null | readonly ModuleInfo[],
  insightOf: (screen: Extract<Screen, { kind: 'insight' }>) => Insight | undefined,
): null | { readonly content: ReactNode; readonly label: string } => {
  switch (screen.kind) {
    case 'app':
      return { content: <AppScreen app={screen.app} />, label: t('ui.screen.frame.nav.apps') };
    case 'apps':
      return { content: <AppsScreen />, label: t('ui.screen.frame.nav.apps') };
    case 'category': {
      const module = moduleOfCategory(modules, screen.category);

      if (module === undefined) {
        return null;
      }

      return {
        content: <CategoryScreen category={screen.category} module={module} />,
        label: tText({ args: {}, key: `${screen.category}.name` }),
      };
    }
    case 'cleanup':
      return { content: <CleanupScreen cleanup={screen.cleanup} />, label: t('ui.screen.frame.nav.cleanups') };
    case 'cleanups':
      return { content: <CleanupsScreen />, label: t('ui.screen.frame.nav.cleanups') };
    case 'insight': {
      const insight = insightOf(screen);

      return insight === undefined
        ? null
        : { content: <InsightScreen insight={insight} module={screen.module} />, label: tText(insight.title) };
    }
    case 'largest':
      return { content: <LargestScreen />, label: t('ui.screen.frame.nav.largest') };
    case 'review':
      return { content: <ReviewScreen />, label: t('ui.screen.frame.nav.review') };
    case 'scan':
      return { content: <ScanScreen />, label: t('ui.screen.frame.nav.scan') };
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
  const described = useCore((state) => state.modules);
  const refs = described.flatMap(({ insights, module }) => insights.map(({ identity }) => ({ identity, module })));
  const insightOf = (shown: { readonly identity: string; readonly module: string }): Insight | undefined =>
    described
      .find(({ module }) => module === shown.module)
      ?.insights.find(({ identity }) => identity === shown.identity);
  const screen = useScreen(
    modules?.flatMap((module) => module.categories.map(({ descriptor }) => descriptor.id)) ?? null,
    refs,
    info?.high ?? null,
  );
  const loaded = useCore((state) => state.dictionary !== null);
  const unreachable = useCore((state) => state.failure?.kind === 'unreachable');
  const demo = useCore((state) => state.demo);

  useRefusalToasts();

  const renderScreen = (shown: Screen): ReactNode => {
    const content = contentOf(shown, modules, insightOf);

    return content === null ? null : (
      <ScrollArea label={content.label} screenKey={screenKey(shown)}>
        {content.content}
      </ScrollArea>
    );
  };

  return (
    <div className={styles.window}>
      {/* The banner waits for the core's dictionary too: a sandbox's line holds the core's sentence. A demo has none. */}
      {info === null || !loaded || demo ? null : <BuildBanner build={info.build} />}
      <div className={styles.body}>
        <AppShell sidebar={<FrameSidebar onNavigate={navigation.navigate} screen={screen} />}>
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
          {loaded ? <AgreementGate /> : null}
        </AppShell>
      </div>
    </div>
  );
};

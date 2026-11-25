import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { CoreAnswers } from '../ipc/mock';

import { mockCore, pushEvents, refuse, sampleAgreementAnswers, sampleStorageAnswers } from '../ipc/mock';
import { requestScan } from '../state/connect';
import { createNavigation, type Navigation } from '../state/navigation';
import { type CoreStore, createCoreStore } from '../state/store';
import { recordAnimations, setMotion } from '../test/animation';
import { BUSY, DEMO_DICTIONARY, DEMO_MODULES, DEMO_OVERVIEW, GRANTED, IDLE } from '../test/demo-scan';
import { BUILD_DICTIONARY, SANDBOX_HOME, sandboxBuild } from '../test/sandbox-build';
import { AppFrame } from './app-frame';
import { AppProviders } from './app-providers';

const core = (answers: Partial<CoreAnswers> = {}): void => {
  mockCore({
    ...sampleAgreementAnswers,
    appInfo: () => ({ build: { kind: 'release' }, high: 'offered', home: '/Users/someone', version: '0.1.0' }),
    getAccessStatus: () => GRANTED,
    getDictionary: () => DEMO_DICTIONARY,
    getModules: () => DEMO_MODULES,
    getOverview: () => IDLE,
    ...sampleStorageAnswers,
    ...answers,
  });
};

type Rendered = { readonly navigation: Navigation; readonly store: CoreStore };

// The frame, once the dictionary and the modules have arrived.
const renderFrame = async (navigation: Navigation = createNavigation()): Promise<Rendered> => {
  const store = createCoreStore();

  render(
    <AppProviders navigation={navigation} store={store}>
      <AppFrame />
    </AppProviders>,
  );
  await screen.findByRole('heading', { level: 1, name: 'Scan' });

  return { navigation, store };
};

const navItems = (): readonly string[] =>
  within(screen.getByRole('navigation', { name: 'Screens' }))
    .getAllByRole('button')
    .map((button) => button.textContent);

beforeEach(async () => {
  await setMotion('off');
});

afterEach(async () => {
  await setMotion(null);
});

// The demo totals and a "Look before removing" category of 4 findings.
const WITH_PHOTOS = {
  ...DEMO_OVERVIEW,
  totals: [
    ...DEMO_OVERVIEW.totals,
    { category: 'demo.photos', count: 4, highestDanger: 'medium' as const, sizes: DEMO_OVERVIEW.found },
  ],
};

const QUIT = 'Stop the cleanup and quit?';

const askToQuit = async (): Promise<HTMLElement> => {
  act(() => {
    pushEvents({ type: 'quit-requested' });
  });

  return screen.findByRole('dialog', { name: QUIT });
};

describe('AppFrame', () => {
  describe('negative cases', () => {
    it('keep cleaning has the first focus and changes nothing', async () => {
      const quitAfterCleanup = vi.fn(() => null);
      core({ quitAfterCleanup });
      const { store } = await renderFrame();
      await askToQuit();

      const keep = screen.getByRole('button', { name: 'Keep cleaning' });
      expect(keep).toHaveFocus();

      fireEvent.click(keep);

      expect(screen.queryByRole('dialog')).toBeNull();
      expect(store.getState().quitAsked).toBe(false);
      expect(quitAfterCleanup).not.toHaveBeenCalled();
    });

    it('a release build has no banner', async () => {
      core({
        appInfo: () => ({ build: { kind: 'release' }, high: 'offered', home: '/Users/someone', version: '0.1.0' }),
      });
      await renderFrame();

      expect(screen.queryByRole('note')).toBeNull();
    });

    it('a sandbox build puts no banner on the page while a demo plays', async () => {
      core({
        appInfo: () => ({
          build: sandboxBuild(['running']),
          high: 'offered',
          home: SANDBOX_HOME,
          version: '0.1.0',
        }),
        getDictionary: () => ({ ...DEMO_DICTIONARY, ...BUILD_DICTIONARY }),
      });
      const { store } = await renderFrame();
      await screen.findByRole('note');

      act(() => {
        store.dispatch({ type: 'demo-started' });
      });

      expect(screen.queryByRole('note')).toBeNull();
      expect(document.body.textContent).not.toContain('/Users/dev/sandboxes/trial');
    });

    it('the sidebar has no Largest while High is withheld', async () => {
      core({
        appInfo: () => ({ build: { kind: 'release' }, high: 'withheld', home: '/Users/someone', version: '0.1.0' }),
      });

      await renderFrame();
      await screen.findByRole('button', { name: 'Apps' });

      expect(navItems()).toStrictEqual(['Scan', 'Apps', 'Review', 'Cleanups', 'Settings']);
    });

    it('the sidebar has no Largest before the app info arrives', async () => {
      core({ appInfo: () => new Promise(() => {}) });

      await renderFrame();

      expect(navItems()).toStrictEqual(['Scan', 'Apps', 'Review', 'Cleanups', 'Settings']);
    });

    it('a category screen renders nothing while the modules are unknown', async () => {
      core({ getModules: () => new Promise(() => {}) });
      const navigation = createNavigation();
      navigation.navigate({ category: 'demo.caches', kind: 'category' });

      const store = createCoreStore();

      render(
        <AppProviders navigation={navigation} store={store}>
          <AppFrame />
        </AppProviders>,
      );
      await vi.waitFor(() => {
        expect(store.getState().dictionary).not.toBeNull();
      });

      expect(screen.getByRole('button', { current: 'page', name: 'Scan' })).toHaveAttribute('aria-current', 'page');
      expect(screen.queryByRole('heading', { level: 1 })).toBeNull();
    });

    it('the details of a category no listed module has fall back to Scan', async () => {
      core();
      const navigation = createNavigation();
      navigation.navigate({ category: 'gone.category', kind: 'category' });

      await renderFrame(navigation);

      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Scan');
    });

    it('the sidebar has no group heading', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderFrame();

      expect(within(screen.getByRole('navigation', { name: 'Screens' })).queryByRole('heading')).toBeNull();
    });

    it('the sidebar has no module item', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderFrame();
      await screen.findByRole('button', { name: 'Largest' });

      expect(navItems()).toStrictEqual(['Scan', 'Largest', 'Apps', 'Review', 'Cleanups', 'Settings']);
      expect(screen.queryByRole('button', { name: /^Demo module/u })).toBeNull();
    });

    it('the review badge is absent with nothing selected', async () => {
      core();

      await renderFrame();

      expect(screen.getByRole('button', { name: 'Review' })).toBeInTheDocument();
    });

    it('the review badge counts only what goes, not a category to check beside it', async () => {
      core();
      const { store } = await renderFrame();

      act(() => {
        store.dispatch({ overview: WITH_PHOTOS, type: 'overview-loaded' });
        store.dispatch({ category: 'demo.photos', type: 'toggle-category' });
        store.dispatch({ category: 'demo.caches', type: 'toggle-category' });
      });

      expect(screen.getByRole('button', { name: 'Review 2 items selected' })).toBeInTheDocument();
    });

    it('an unreachable core shows the error state, not a blank window', async () => {
      mockCore({ ...sampleAgreementAnswers });

      render(
        <AppProviders store={createCoreStore()}>
          <AppFrame />
        </AppProviders>,
      );

      expect(await screen.findByText('Fresher stopped answering. Quit it and open it again.')).toBeInTheDocument();
      expect(screen.getByText('Fresher is not answering')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
    });

    it('motion off changes screens without animation', async () => {
      core();
      await renderFrame();
      const animations = recordAnimations();

      fireEvent.click(screen.getByRole('button', { name: 'Settings' }));

      expect(screen.queryByRole('heading', { name: 'Scan' })).toBeNull();
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('General');
      expect(animations).toHaveLength(0);
    });

    it('coming back from a category whose card is gone puts the focus on the heading', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { navigation } = await renderFrame();

      act(() => {
        navigation.navigate({ category: 'demo.caches', kind: 'category' });
      });

      act(() => {
        navigation.navigate({ from: 'demo.gone', kind: 'scan' });
      });

      expect(screen.getByRole('heading', { level: 1, name: 'Scan' })).toHaveFocus();
    });
  });

  describe('positive cases', () => {
    it("coming back from a category's details puts the focus on that card's Details", async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { navigation } = await renderFrame();
      act(() => {
        navigation.navigate({ category: 'demo.caches', kind: 'category' });
      });

      act(() => {
        navigation.navigate({ from: 'demo.caches', kind: 'scan' });
      });

      const card = screen
        .getAllByRole('listitem')
        .find((item) => within(item).queryByRole('heading', { level: 3, name: 'Example caches' }) !== null);
      expect(within(card as HTMLElement).getByRole('button', { name: /Details/ })).toHaveFocus();
    });

    it('the sidebar lists Scan, Largest, Apps, Review, Cleanups and Settings, with Scan current on the scan screen', async () => {
      core({ getOverview: () => DEMO_OVERVIEW });

      await renderFrame();
      await screen.findByRole('button', { name: 'Largest' });

      expect(navItems()).toStrictEqual(['Scan', 'Largest', 'Apps', 'Review', 'Cleanups', 'Settings']);
      expect(screen.getByRole('button', { current: 'page', name: 'Scan' })).toBeInTheDocument();
    });

    it('the sidebar shows the logo beside the name', async () => {
      core();

      await renderFrame();

      const name = screen.getByText('Fresher');

      // eslint-disable-next-line testing-library/no-node-access -- the mark has no role
      expect(name.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
    });

    it('a sandbox build shows the sandbox banner above the window', async () => {
      core({
        appInfo: () => ({
          build: sandboxBuild(['running']),
          high: 'offered',
          home: SANDBOX_HOME,
          version: '0.1.0',
        }),
        getDictionary: () => ({ ...DEMO_DICTIONARY, ...BUILD_DICTIONARY }),
      });
      await renderFrame();

      expect(await screen.findByRole('note')).toHaveTextContent(
        "Sandbox · /Users/dev/sandboxes/trial · Cleanups move items to the sandbox's Trash: " +
          '/Users/dev/sandboxes/trial/home/.Trash · Simulated: open apps.',
      );
    });

    it('stop and quit asks the core', async () => {
      const quitAfterCleanup = vi.fn(() => null);
      core({ quitAfterCleanup });
      await renderFrame();
      await askToQuit();

      fireEvent.click(screen.getByRole('button', { name: 'Stop and quit' }));

      await vi.waitFor(() => {
        expect(quitAfterCleanup).toHaveBeenCalledOnce();
      });
      expect(screen.queryByRole('dialog')).toBeNull();
    });

    it('the review badge counts what is selected', async () => {
      core();
      const { store } = await renderFrame();

      act(() => {
        store.dispatch({ overview: DEMO_OVERVIEW, type: 'overview-loaded' });
        store.dispatch({ category: 'demo.caches', type: 'toggle-category' });
      });

      expect(screen.getByRole('button', { name: 'Review 2 items selected' })).toBeInTheDocument();
    });

    it('with nothing going, the review badge counts the categories to check', async () => {
      core();
      const { store } = await renderFrame();

      act(() => {
        store.dispatch({ overview: WITH_PHOTOS, type: 'overview-loaded' });
        store.dispatch({ category: 'demo.photos', type: 'toggle-category' });
      });

      expect(screen.getByRole('button', { name: 'Review 1 to check in Review' })).toBeInTheDocument();
    });

    it('choosing a nav item shows its screen and moves the focus to its heading', async () => {
      core();
      const { navigation } = await renderFrame();

      fireEvent.click(screen.getByRole('button', { name: 'Cleanups' }));

      const heading = screen.getByRole('heading', { level: 1 });
      expect(heading).toHaveTextContent('Cleanups');
      expect(heading).toHaveFocus();
      expect(screen.getByRole('button', { name: 'Cleanups' })).toHaveAttribute('aria-current', 'page');
      expect(navigation.getScreen()).toStrictEqual({ kind: 'cleanups' });
    });

    it('Largest opens its screen with its heading focused and the entry current', async () => {
      core();
      const { navigation } = await renderFrame();

      fireEvent.click(await screen.findByRole('button', { name: 'Largest' }));

      const heading = screen.getByRole('heading', { level: 1 });
      expect(heading).toHaveTextContent('Largest files and folders');
      expect(heading).toHaveFocus();
      expect(screen.getByRole('button', { name: 'Largest' })).toHaveAttribute('aria-current', 'page');
      expect(
        within(screen.getByRole('navigation', { name: 'Screens' })).getByRole('button', { name: 'Scan' }),
      ).not.toHaveAttribute('aria-current');
      expect(navigation.getScreen()).toStrictEqual({ kind: 'largest' });
    });

    it('marks Apps active on the apps and app screens', async () => {
      core();
      const { navigation } = await renderFrame();

      fireEvent.click(screen.getByRole('button', { name: 'Apps' }));

      expect(await screen.findByRole('heading', { level: 1, name: 'Apps' })).toHaveFocus();
      expect(screen.getByRole('button', { name: 'Apps' })).toHaveAttribute('aria-current', 'page');
      expect(navigation.getScreen()).toStrictEqual({ kind: 'apps' });

      act(() => {
        navigation.navigate({ app: { index: 0, state: 1 }, kind: 'app' });
      });

      expect(await screen.findByRole('heading', { level: 1, name: 'Apps' })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Apps' })).toHaveAttribute('aria-current', 'page');
      expect(screen.getByRole('button', { name: 'Scan' })).not.toHaveAttribute('aria-current');
    });

    it("a category's details render with its name, the focus on its heading and Scan current", async () => {
      core({ getOverview: () => DEMO_OVERVIEW });
      const { navigation } = await renderFrame();

      act(() => {
        navigation.navigate({ category: 'demo.caches', kind: 'category' });
      });

      const heading = screen.getByRole('heading', { level: 1 });
      expect(heading).toHaveTextContent('Example caches');
      expect(heading).toHaveFocus();
      expect(screen.getByRole('button', { current: 'page', name: 'Scan' })).toHaveAttribute('aria-current', 'page');
    });

    it('the first screen does not take the focus', async () => {
      core();

      await renderFrame();

      expect(screen.getByRole('heading', { level: 1 })).not.toHaveFocus();
    });

    it('with motion on, the outgoing screen fades out before it leaves', async () => {
      core();
      await renderFrame();
      await setMotion('full');
      const animations = recordAnimations();

      fireEvent.click(screen.getByRole('button', { name: 'Review' }));

      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Review');
      // Still there while it fades, but hidden from assistive technology and out of reach.
      expect(screen.getByText('Scan', { selector: 'h1' })).toBeInTheDocument();
      expect(screen.queryByRole('heading', { name: 'Scan' })).toBeNull();
      expect(animations.map(({ keyframes }) => keyframes)).toStrictEqual([[{ opacity: 1 }, { opacity: 0 }]]);

      await animations[0]?.finish();

      expect(screen.queryByText('Scan', { selector: 'h1' })).toBeNull();
    });

    it("a refused scan shows a toast with the core's text", async () => {
      core({ startScan: refuse(BUSY) });
      const { store } = await renderFrame();

      act(() => {
        requestScan(store);
      });

      expect(
        await within(screen.getByRole('region', { name: 'Notifications' })).findByText(
          'The app is already running a scan. Wait for it to finish, or cancel it.',
        ),
      ).toBeInTheDocument();
    });
  });
});

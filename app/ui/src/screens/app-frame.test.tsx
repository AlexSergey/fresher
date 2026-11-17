import { act, fireEvent, render, screen, within } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { ModuleInfo } from '../ipc/bindings.generated';
import type { CoreAnswers } from '../ipc/mock';

import { mockCore, pushEvents, refuse, sampleStorageAnswers } from '../ipc/mock';
import { requestScan } from '../state/connect';
import { createNavigation, type Navigation } from '../state/navigation';
import { type CoreStore, createCoreStore } from '../state/store';
import { recordAnimations, setMotion } from '../test/animation';
import { BUSY, DEMO_DICTIONARY, DEMO_MODULES, DEMO_OVERVIEW, GRANTED, IDLE } from '../test/demo-scan';
import { AppFrame } from './app-frame';
import { AppProviders } from './app-providers';

const core = (answers: Partial<CoreAnswers> = {}): void => {
  mockCore({
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
  await screen.findByRole('heading', { level: 1, name: 'Overview' });
  await screen.findByRole('button', { name: /^Demo module/ });

  return { navigation, store };
};

const navItems = (): readonly string[] =>
  within(screen.getByRole('navigation', { name: 'Screens' }))
    .getAllByRole('button')
    .map((button) => button.textContent);

const demoItem = (): HTMLElement => screen.getByRole('button', { name: /^Demo module/ });

beforeEach(async () => {
  await setMotion('off');
});

afterEach(async () => {
  await setMotion(null);
});

const QUIT = 'A cleanup is running. Stop after the current item and quit?';

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
      core({ appInfo: () => ({ build: { kind: 'release' }, version: '0.1.0' }) });
      await renderFrame();

      expect(screen.queryByRole('note')).toBeNull();
    });

    it('a module the core does not list has no nav item', async () => {
      core();
      const navigation = createNavigation();
      navigation.navigate({ kind: 'module', module: 'docker' });

      await renderFrame(navigation);

      expect(navItems()).toStrictEqual(['Overview', 'Demo module—', 'Review', 'Cleanups', 'Settings']);
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Overview');
    });

    it('a module shows a dash, not a zero, before a scan', async () => {
      core();

      await renderFrame();

      expect(demoItem()).toHaveTextContent('Demo module—');
    });

    it('the review badge is absent with nothing selected', async () => {
      core();

      await renderFrame();

      expect(screen.getByRole('button', { name: 'Review' })).toBeInTheDocument();
    });

    it('an unreachable core shows the error state, not a blank window', async () => {
      mockCore({});

      render(
        <AppProviders store={createCoreStore()}>
          <AppFrame />
        </AppProviders>,
      );

      expect(await screen.findByText('The app could not reach its core. Restart the app.')).toBeInTheDocument();
      expect(screen.getByText('The core is not answering')).toBeInTheDocument();
      expect(screen.getByRole('button', { name: 'Try again' })).toBeInTheDocument();
    });

    it('motion off changes screens without animation', async () => {
      core();
      await renderFrame();
      const animations = recordAnimations();

      fireEvent.click(screen.getByRole('button', { name: 'Settings' }));

      expect(screen.queryByRole('heading', { name: 'Overview' })).toBeNull();
      expect(screen.getByRole('heading', { level: 1 })).toHaveTextContent('Settings');
      expect(animations).toHaveLength(0);
    });
  });

  describe('positive cases', () => {
    it('a sandbox build shows the sandbox banner above the window', async () => {
      core({ appInfo: () => ({ build: { kind: 'sandbox', path: '/Users/dev/box' }, version: '0.1.0' }) });
      await renderFrame();

      expect(await screen.findByRole('note')).toHaveTextContent('Sandbox · /Users/dev/box');
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

    it('the nav items follow getModules in order', async () => {
      const second: ModuleInfo = { categories: [], id: 'logs', settings: ['logs.enabled'] };
      core({
        getDictionary: () => ({ ...DEMO_DICTIONARY, 'logs.enabled.label': 'Logs' }),
        getModules: () => [second, ...DEMO_MODULES],
      });
      await renderFrame();

      const modules = within(screen.getByRole('region', { name: 'Modules' })).getAllByRole('button');

      expect(modules.map((item) => item.textContent)).toStrictEqual(['Logs—', 'Demo module—']);
    });

    it("a module's size updates when the overview does", async () => {
      core();
      const { store } = await renderFrame();

      act(() => {
        store.dispatch({ overview: DEMO_OVERVIEW, type: 'overview-loaded' });
      });

      expect(demoItem()).toHaveTextContent('Demo module2.6 GB');
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

    it('choosing a nav item shows its screen and moves the focus to its heading', async () => {
      core();
      const { navigation } = await renderFrame();

      fireEvent.click(demoItem());

      const heading = screen.getByRole('heading', { level: 1 });
      expect(heading).toHaveTextContent('Demo module');
      expect(heading).toHaveFocus();
      expect(demoItem()).toHaveAttribute('aria-current', 'page');
      expect(navigation.getScreen()).toStrictEqual({ kind: 'module', module: 'demo' });
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
      expect(screen.getByText('Overview', { selector: 'h1' })).toBeInTheDocument();
      expect(screen.queryByRole('heading', { name: 'Overview' })).toBeNull();
      expect(animations.map(({ keyframes }) => keyframes)).toStrictEqual([[{ opacity: 1 }, { opacity: 0 }]]);

      await animations[0]?.finish();

      expect(screen.queryByText('Overview', { selector: 'h1' })).toBeNull();
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

import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

// The entry renders the app on import; the app and its store are replaced, only the window's mode is under test.
vi.mock('./app', () => ({ App: (): null => null }));
vi.mock('./state/store', () => ({ createCoreStore: (): object => ({}) }));

const start = async (): Promise<string | undefined> => {
  vi.resetModules();
  await import('./main');

  return document.documentElement.dataset.window;
};

describe('main', () => {
  beforeEach(() => {
    const root = document.createElement('div');
    root.id = 'root';
    document.body.append(root);
  });

  afterEach(() => {
    Reflect.deleteProperty(globalThis, 'isTauri');
    delete document.documentElement.dataset.window;
    document.body.replaceChildren();
  });

  describe('negative cases', () => {
    it('leaves the window solid outside the app', async () => {
      expect(await start()).toBeUndefined();
    });
  });

  describe('positive cases', () => {
    it('sets data-window only in the app', async () => {
      Reflect.set(globalThis, 'isTauri', true);

      expect(await start()).toBe('vibrant');
    });
  });
});

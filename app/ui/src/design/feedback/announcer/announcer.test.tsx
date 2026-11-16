import { act, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { type AnnounceOptions, Announcer, useAnnounce } from './announcer';

let announce: (text: string, options?: AnnounceOptions) => void = () => undefined;

const Speaker = (): null => {
  announce = useAnnounce();

  return null;
};

const say = (text: string, options?: AnnounceOptions): void => {
  act(() => {
    announce(text, options);
  });
};

const wait = (ms: number): void => {
  act(() => {
    vi.advanceTimersByTime(ms);
  });
};

// What the region says, without the no-break space that makes a repeated text change.
const spoken = (): string => screen.getByRole('status').textContent.replace(' ', '');

const renderAnnouncer = (): void => {
  render(
    <Announcer>
      <Speaker />
    </Announcer>,
  );
};

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('Announcer', () => {
  describe('negative cases', () => {
    it('the announcer does not speak the same key twice within its interval', () => {
      renderAnnouncer();

      say('Looked at 1,000 items', { key: 'scan' });
      wait(1000);
      say('Looked at 2,000 items', { key: 'scan' });

      expect(spoken()).toBe('Looked at 1,000 items');
    });

    it('a waiting text does not outlive the announcer', () => {
      const { unmount } = render(
        <Announcer>
          <Speaker />
        </Announcer>,
      );
      say('Looked at 1,000 items', { key: 'scan' });
      say('Looked at 2,000 items', { key: 'scan' });

      expect(vi.getTimerCount()).toBe(1);

      unmount();

      expect(vi.getTimerCount()).toBe(0);
    });

    it('useAnnounce outside an Announcer fails loudly', () => {
      vi.spyOn(console, 'error').mockImplementation(() => undefined);

      expect(() => render(<Speaker />)).toThrow('useAnnounce() is used outside an Announcer');
    });
  });

  describe('positive cases', () => {
    it('is one polite, visually hidden region', () => {
      renderAnnouncer();

      expect(screen.getByRole('status')).toHaveAttribute('aria-live', 'polite');
      expect(screen.getByRole('status')).toHaveClass('u-visually-hidden');
    });

    it('the announcer speaks the last text of a burst', () => {
      renderAnnouncer();

      say('Looked at 1,000 items', { key: 'scan' });
      say('Looked at 2,000 items', { key: 'scan' });
      wait(2000);
      say('Looked at 3,000 items', { key: 'scan' });
      wait(2999);

      expect(spoken()).toBe('Looked at 1,000 items');

      wait(1);

      expect(spoken()).toBe('Looked at 3,000 items');
    });

    it('a final announcement speaks at once', () => {
      renderAnnouncer();

      say('Looked at 1,000 items', { key: 'scan' });
      say('Looked at 2,000 items', { key: 'scan' });
      say('Scan finished: 42 GB found', { final: true, key: 'scan' });

      expect(spoken()).toBe('Scan finished: 42 GB found');

      wait(10_000);

      expect(spoken()).toBe('Scan finished: 42 GB found');
    });

    it('keys keep their own rate, a text without a key speaks at once', () => {
      renderAnnouncer();

      say('Looked at 1,000 items', { key: 'scan' });
      say('Removed 3 items', { key: 'clean' });

      expect(spoken()).toBe('Removed 3 items');

      say('Settings saved');

      expect(spoken()).toBe('Settings saved');
    });

    it('speaks the same text again after its interval, and honours its own interval', () => {
      renderAnnouncer();

      say('Still scanning', { key: 'scan', minInterval: 1000 });
      const first = screen.getByRole('status').textContent;
      wait(1000);
      say('Still scanning', { key: 'scan', minInterval: 1000 });

      expect(screen.getByRole('status').textContent).not.toBe(first);
      expect(spoken()).toBe('Still scanning');
    });
  });
});

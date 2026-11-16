import { act, fireEvent, render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { recordAnimations, setMotion } from '../../../test/animation';
import { type ToastContent } from './toast';
import { TOAST_TIMEOUT, Toaster, useToasts } from './toaster';

let show: (content: ToastContent) => void = () => undefined;

const Shower = (): null => {
  ({ show } = useToasts());

  return null;
};

const renderToaster = (): void => {
  render(
    <Toaster closeLabel="Close" label="Notifications">
      <Shower />
    </Toaster>,
  );
};

const toast = (content: ToastContent): void => {
  act(() => {
    show(content);
  });
};

const wait = (ms: number): void => {
  act(() => {
    vi.advanceTimersByTime(ms);
  });
};

const region = (): HTMLElement => screen.getByRole('region', { name: 'Notifications' });

beforeEach(async () => {
  vi.useFakeTimers();
  await setMotion('off');
});

afterEach(async () => {
  vi.useRealTimers();
  await setMotion(null);
});

describe('Toaster', () => {
  describe('negative cases', () => {
    it('an error toast does not hide by itself', () => {
      renderToaster();

      toast({ text: 'The Trash could not be reached.', tone: 'error' });
      wait(60_000);

      expect(region()).toHaveTextContent('The Trash could not be reached.');
    });

    it('a toast does not hide while hovered', () => {
      renderToaster();
      toast({ text: 'Settings saved.', tone: 'success' });
      const shown = screen.getByText('Settings saved.');

      fireEvent.pointerEnter(shown);
      wait(TOAST_TIMEOUT * 3);

      expect(region()).toHaveTextContent('Settings saved.');

      fireEvent.pointerLeave(shown);
      wait(TOAST_TIMEOUT);

      expect(region()).not.toHaveTextContent('Settings saved.');
    });

    it('a toast does not hide while its close button has the focus', () => {
      renderToaster();
      toast({ text: 'Settings saved.', tone: 'info' });

      act(() => {
        screen.getByRole('button', { name: 'Close' }).focus();
      });
      wait(TOAST_TIMEOUT * 2);

      expect(region()).toHaveTextContent('Settings saved.');

      act(() => {
        screen.getByRole('button', { name: 'Close' }).blur();
      });
      wait(TOAST_TIMEOUT);

      expect(region()).not.toHaveTextContent('Settings saved.');
    });

    it('a fourth toast pushes out the oldest', () => {
      renderToaster();

      for (const text of ['First.', 'Second.', 'Third.', 'Fourth.']) {
        toast({ text, tone: 'error' });
      }

      expect(region()).not.toHaveTextContent('First.');
      expect(region()).toHaveTextContent('Second.Third.Fourth.');
    });

    it('useToasts outside a Toaster fails loudly', () => {
      vi.spyOn(console, 'error').mockImplementation(() => undefined);

      expect(() => render(<Shower />)).toThrow('useToasts() is used outside a Toaster');
    });
  });

  describe('positive cases', () => {
    it('is one polite region', () => {
      renderToaster();

      expect(region()).toHaveAttribute('aria-live', 'polite');
      expect(region()).toBeEmptyDOMElement();
    });

    it('an info toast hides after 5 seconds', () => {
      renderToaster();

      toast({ text: 'Scan started.', tone: 'info' });
      wait(TOAST_TIMEOUT - 1);

      expect(region()).toHaveTextContent('Scan started.');

      wait(1);

      expect(region()).not.toHaveTextContent('Scan started.');
    });

    it('closes from its close button', () => {
      renderToaster();
      toast({ text: 'The Trash could not be reached.', tone: 'error' });

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));

      expect(region()).toBeEmptyDOMElement();
    });

    it('runs its action and closes', () => {
      const onClick = vi.fn();
      renderToaster();
      toast({ action: { label: 'Show in Finder', onClick }, text: 'Moved 3 items to the Trash.', tone: 'success' });

      fireEvent.click(screen.getByRole('button', { name: 'Show in Finder' }));

      expect(onClick).toHaveBeenCalledOnce();
      expect(region()).toBeEmptyDOMElement();
    });

    it('leaves by fading, then is removed', async () => {
      await setMotion('full');
      const animations = recordAnimations();
      renderToaster();
      toast({ text: 'The Trash could not be reached.', tone: 'error' });

      fireEvent.click(screen.getByRole('button', { name: 'Close' }));

      expect(region()).toHaveTextContent('The Trash could not be reached.');
      expect(animations.at(-1)?.keyframes).toStrictEqual([{ opacity: 1 }, { opacity: 0 }]);

      await animations.at(-1)?.finish();

      expect(region()).toBeEmptyDOMElement();
    });
  });
});

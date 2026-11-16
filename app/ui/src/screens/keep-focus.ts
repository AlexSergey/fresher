import { type RefObject, useEffect } from 'react';

// Moves the focus to the screen's heading (its first `h1`), so the keyboard and VoiceOver start on the new screen.
export const focusHeading = (element: HTMLElement | null): void => {
  const heading = element?.querySelector('h1') ?? null;

  if (heading === null) {
    return;
  }

  if (!heading.hasAttribute('tabindex')) {
    heading.tabIndex = -1;
  }
  heading.focus({ preventScroll: true });
};

const dropped = (): boolean => document.activeElement === null || document.activeElement === document.body;

// While `active`: when the element that had the focus leaves the screen with it (Scan turns into Stop, a scan ends,
// a bar closes), the focus goes to the screen's heading instead of being dropped on the page.
export const useKeepFocus = (ref: RefObject<HTMLElement | null>, active: boolean): void => {
  useEffect(() => {
    const screen = ref.current;

    if (screen === null || !active) {
      return;
    }

    let focused: Element | null = null;
    const onFocusIn = (event: FocusEvent): void => {
      focused = event.target instanceof Element ? event.target : null;
    };
    // The focus went elsewhere on purpose; an element removed with the focus has no next one.
    const onFocusOut = (event: FocusEvent): void => {
      if (event.relatedTarget instanceof Node && !screen.contains(event.relatedTarget)) {
        focused = null;
      }
    };
    const observer = new MutationObserver(() => {
      if (focused !== null && !focused.isConnected && dropped()) {
        focused = null;
        focusHeading(screen);
      }
    });

    screen.addEventListener('focusin', onFocusIn);
    screen.addEventListener('focusout', onFocusOut);
    observer.observe(screen, { childList: true, subtree: true });

    return (): void => {
      observer.disconnect();
      screen.removeEventListener('focusin', onFocusIn);
      screen.removeEventListener('focusout', onFocusOut);
    };
  }, [ref, active]);
};

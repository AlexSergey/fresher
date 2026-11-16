import { type RefObject, useCallback, useLayoutEffect, useState } from 'react';

const FOCUSABLE =
  'a[href], button:not(:disabled), input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex="-1"])';

// Whether a scrolling element must take the focus itself, so the keyboard can scroll it: only when it overflows and
// its content holds nothing focusable. Measured on every render and whenever the element or its content resizes.
export const useOverflowFocus = (
  areaRef: RefObject<HTMLElement | null>,
  contentRef: RefObject<HTMLElement | null>,
): boolean => {
  const [focusable, setFocusable] = useState(false);

  const measure = useCallback((): void => {
    const area = areaRef.current;
    const content = contentRef.current;

    if (area !== null && content !== null) {
      setFocusable(area.scrollHeight > area.clientHeight && content.querySelector(FOCUSABLE) === null);
    }
  }, [areaRef, contentRef]);

  // Its content may have changed with any render.
  useLayoutEffect(measure);

  useLayoutEffect(() => {
    const area = areaRef.current;
    const content = contentRef.current;

    if (area === null || content === null) {
      return;
    }

    const observer = new ResizeObserver(measure);

    observer.observe(area);
    observer.observe(content);

    return (): void => {
      observer.disconnect();
    };
  }, [areaRef, contentRef, measure]);

  return focusable;
};

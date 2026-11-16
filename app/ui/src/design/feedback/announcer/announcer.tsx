import { createContext, type JSX, type ReactNode, use, useCallback, useEffect, useRef, useState } from 'react';

export type AnnounceOptions = {
  // Speaks at once, whatever was spoken before (the end of a scan); a text waiting under the same key is dropped.
  readonly final?: boolean;
  // Announcements under one key share a rate; without a key a text is spoken at once.
  readonly key?: string;
  // Milliseconds between two texts of the same key; 5 seconds by default.
  readonly minInterval?: number;
};

type Announce = (text: string, options?: AnnounceOptions) => void;

type Rate = { last: number; pending: null | string; timer: ReturnType<typeof setTimeout> | undefined };

const DEFAULT_INTERVAL = 5000;

const AnnounceContext = createContext<Announce | null>(null);

// Gives its children `useAnnounce()` and renders the one visually hidden polite live region that speaks.
export const Announcer = ({ children }: { readonly children: ReactNode }): JSX.Element => {
  // A repeated text is spoken again: every other message ends with a no-break space, so the region always changes.
  const [message, setMessage] = useState({ text: '', toggle: false });
  const ratesRef = useRef(new Map<string, Rate>());

  const speak = useCallback((text: string): void => {
    setMessage((previous) => ({ text, toggle: !previous.toggle }));
  }, []);

  const announce = useCallback<Announce>(
    (text, { final = false, key, minInterval = DEFAULT_INTERVAL } = {}) => {
      if (key === undefined) {
        speak(text);

        return;
      }

      const now = Date.now();
      const rate = ratesRef.current.get(key) ?? { last: Number.NEGATIVE_INFINITY, pending: null, timer: undefined };

      ratesRef.current.set(key, rate);

      if (final || now - rate.last >= minInterval) {
        clearTimeout(rate.timer);
        rate.timer = undefined;
        rate.pending = null;
        rate.last = now;
        speak(text);

        return;
      }

      // Within the interval: the text waits, replacing any text waiting before it, and is spoken when it ends.
      rate.pending = text;
      rate.timer ??= setTimeout(
        () => {
          rate.timer = undefined;
          rate.last = Date.now();

          if (rate.pending !== null) {
            speak(rate.pending);
            rate.pending = null;
          }
        },
        rate.last + minInterval - now,
      );
    },
    [speak],
  );

  useEffect(() => {
    const rates = ratesRef.current;

    return (): void => {
      for (const rate of rates.values()) {
        clearTimeout(rate.timer);
      }
    };
  }, []);

  return (
    <AnnounceContext value={announce}>
      {children}
      <div aria-live="polite" className="u-visually-hidden" role="status">
        {message.toggle ? `${message.text} ` : message.text}
      </div>
    </AnnounceContext>
  );
};

// Speaks a text through the `Announcer` above: `announce(text, { key, minInterval })`, or `{ final: true }` at once.
export const useAnnounce = (): Announce => {
  const announce = use(AnnounceContext);

  if (announce === null) {
    throw new Error('useAnnounce() is used outside an Announcer');
  }

  return announce;
};

import {
  createContext,
  type JSX,
  type ReactNode,
  use,
  useCallback,
  useEffect,
  useEffectEvent,
  useMemo,
  useRef,
  useState,
} from 'react';

import { usePresence } from '../../motion';
import { DURATION } from '../../tokens/motion';
import { Toast, type ToastContent } from './toast';
import styles from './toaster.module.css';

export type ToasterProps = {
  readonly children: ReactNode;
  // Each toast's close button's accessible name ("Close").
  readonly closeLabel: string;
  // The region's accessible name ("Notifications").
  readonly label: string;
};

type Shown = { readonly content: ToastContent; readonly id: number; readonly open: boolean };

type Toasts = { readonly show: (content: ToastContent) => void };

// How long an info or a success toast stays, in milliseconds; an error stays until closed.
export const TOAST_TIMEOUT = 5000;

// The most toasts open at once; a new one pushes the oldest out.
const MOST = 3;

const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

const ToastsContext = createContext<null | Toasts>(null);

type ItemProps = {
  readonly closeLabel: string;
  readonly onClose: () => void;
  readonly onGone: () => void;
  readonly shown: Shown;
};

// One toast with its timer: paused while the pointer or the focus is on it, started again in full when they leave.
const Item = ({ closeLabel, onClose, onGone, shown }: ItemProps): JSX.Element | null => {
  const { mounted, ref } = usePresence<HTMLDivElement>(shown.open, { duration: DURATION.fast, exit: FADE_OUT });
  const [hovered, setHovered] = useState(false);
  const [focused, setFocused] = useState(false);
  const timed = shown.content.tone !== 'error';
  const close = useEffectEvent(onClose);
  const gone = useEffectEvent(onGone);

  useEffect(() => {
    if (!timed || !shown.open || hovered || focused) {
      return;
    }

    const timer = setTimeout(close, TOAST_TIMEOUT);

    return (): void => {
      clearTimeout(timer);
    };
  }, [timed, shown.open, hovered, focused]);

  useEffect(() => {
    if (!mounted) {
      gone();
    }
  }, [mounted]);

  if (!mounted) {
    return null;
  }

  return (
    <div
      onBlur={() => {
        setFocused(false);
      }}
      onFocus={() => {
        setFocused(true);
      }}
      onPointerEnter={() => {
        setHovered(true);
      }}
      onPointerLeave={() => {
        setHovered(false);
      }}
      ref={ref}
    >
      <Toast {...shown.content} closeLabel={closeLabel} onClose={onClose} />
    </div>
  );
};

// Gives its children `useToasts()` and renders the one toast region, a polite live region at the bottom centre.
export const Toaster = ({ children, closeLabel, label }: ToasterProps): JSX.Element => {
  const [shown, setShown] = useState<readonly Shown[]>([]);
  const nextIdRef = useRef(0);

  const close = useCallback((id: number): void => {
    setShown((all) => all.map((toast) => (toast.id === id ? { ...toast, open: false } : toast)));
  }, []);

  const remove = useCallback((id: number): void => {
    setShown((all) => all.filter((toast) => toast.id !== id));
  }, []);

  const show = useCallback((content: ToastContent): void => {
    const id = nextIdRef.current;

    nextIdRef.current += 1;
    setShown((all) => {
      const open = all.filter((toast) => toast.open);
      const leaving = new Set(open.slice(0, Math.max(0, open.length + 1 - MOST)).map((toast) => toast.id));

      return [
        ...all.map((toast) => (leaving.has(toast.id) ? { ...toast, open: false } : toast)),
        { content, id, open: true },
      ];
    });
  }, []);

  const toasts = useMemo(() => ({ show }), [show]);

  return (
    <ToastsContext value={toasts}>
      {children}
      <section aria-label={label} aria-live="polite" className={styles.region}>
        {shown.map((toast) => (
          <Item
            closeLabel={closeLabel}
            key={toast.id}
            onClose={() => {
              close(toast.id);
            }}
            onGone={() => {
              remove(toast.id);
            }}
            shown={toast}
          />
        ))}
      </section>
    </ToastsContext>
  );
};

// Shows a toast: `show({ tone, text, action })`. Only inside a `Toaster`.
export const useToasts = (): Toasts => {
  const toasts = use(ToastsContext);

  if (toasts === null) {
    throw new Error('useToasts() is used outside a Toaster');
  }

  return toasts;
};

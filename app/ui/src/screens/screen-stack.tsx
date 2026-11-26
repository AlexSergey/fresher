import { type JSX, type ReactNode, useEffect, useEffectEvent, useState } from 'react';

import type { Screen } from '../state/navigation';

import { usePresence } from '../design/motion';
import { DURATION } from '../design/tokens/motion';
import styles from './app-frame.module.css';
import { focusCard, focusHeading, focusInsightCard, useKeepFocus } from './keep-focus';

export type ScreenStackProps = {
  // The screen itself, inside its `ScrollArea`.
  readonly render: (screen: Screen) => ReactNode;
  readonly screen: Screen;
};

// One screen on the stack: open (the current one) or leaving. `arrived` when a screen change brought it: it fades in
// and its heading takes the focus (Scan after a details screen: the card it came from); the first screen of the app just appears.
// `mount` counts the openings of one key: a screen opened again while it still fades out is a new mount, so it reads
// its data anew as any screen does when it opens.
type Layer = {
  readonly arrived: boolean;
  readonly key: string;
  readonly mount: number;
  readonly open: boolean;
  readonly screen: Screen;
};

const FADE_OUT: readonly Keyframe[] = [{ opacity: 1 }, { opacity: 0 }];

// The screens of one kind share one scroll position; a category's details and a cleanup keep their own.
export const screenKey = (screen: Screen): string => {
  switch (screen.kind) {
    case 'app':
      return `app:${String(screen.app.state)}:${String(screen.app.index)}`;
    case 'apps':
    case 'cleanups':
    case 'largest':
    case 'review':
    case 'scan':
    case 'settings':
      return screen.kind;
    case 'category':
      return `category:${screen.category}`;
    case 'cleanup':
      return `cleanup:${String(screen.cleanup)}`;
    case 'insight':
      return `insight:${screen.module}/${screen.identity}`;
  }
};

type ScreenLayerProps = {
  readonly children: ReactNode;
  readonly layer: Layer;
  readonly onGone: () => void;
};

const ScreenLayer = ({ children, layer, onGone }: ScreenLayerProps): JSX.Element | null => {
  const { mounted, ref } = usePresence<HTMLDivElement>(layer.open, { duration: DURATION.fast, exit: FADE_OUT });
  const gone = useEffectEvent(onGone);

  useKeepFocus(ref, layer.open);

  useEffect(() => {
    if (!mounted) {
      gone();
    }
  }, [mounted]);

  useEffect(() => {
    if (!layer.open || !layer.arrived) {
      return;
    }

    const { from, fromInsight } =
      layer.screen.kind === 'scan' ? layer.screen : { from: undefined, fromInsight: undefined };
    const returned =
      (from !== undefined && focusCard(ref.current, from)) ||
      (fromInsight !== undefined && focusInsightCard(ref.current, fromInsight));

    if (!returned) {
      focusHeading(ref.current);
    }
  }, [layer.open, layer.arrived, layer.screen, ref]);

  if (!mounted) {
    return null;
  }

  return (
    <div
      aria-hidden={layer.open ? undefined : true}
      className={styles.screen}
      data-arrived={layer.arrived || undefined}
      data-screen={layer.key}
      inert={!layer.open}
      ref={ref}
    >
      {children}
    </div>
  );
};

// The current screen, and the one it replaced while that fades out (`--duration-fast`); the new one fades in and rises
// `--distance-1` over `--duration-base`. Both sit in one grid cell, the positioned parent of a screen's bottom bars.
export const ScreenStack = ({ render, screen }: ScreenStackProps): JSX.Element => {
  const [shown, setShown] = useState(screen);
  const [layers, setLayers] = useState<readonly Layer[]>(() => [
    { arrived: false, key: screenKey(screen), mount: 0, open: true, screen },
  ]);

  // A new screen replaces the open one in the same render, so no frame shows neither.
  if (shown !== screen) {
    const key = screenKey(screen);

    setShown(screen);
    setLayers((all) => {
      const current = all.find((layer) => layer.open);
      const leaving = all.find((layer) => layer.key === key && !layer.open);
      const others = all.filter((layer) => layer.key !== key).map((layer) => ({ ...layer, open: false }));
      const mount = current?.key === key ? current.mount : (leaving?.mount ?? -1) + 1;

      return [...others, { arrived: current?.key !== key || current.arrived, key, mount, open: true, screen }];
    });
  }

  return (
    <div className={styles.stack}>
      {layers.map((layer) => (
        <ScreenLayer
          key={`${layer.key}#${String(layer.mount)}`}
          layer={layer}
          onGone={() => {
            setLayers((all) => all.filter((one) => one.key !== layer.key || one.open));
          }}
        >
          {render(layer.screen)}
        </ScreenLayer>
      ))}
    </div>
  );
};

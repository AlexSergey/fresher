import { type JSX, useEffect, useRef, useState } from 'react';

import type { NodeId, OwnPlace } from '../../ipc/bindings.generated';

import { Surface } from '../../design/layout/surface/surface';
import { Button } from '../../design/primitives/button/button';
import { t } from '../../i18n';
import { failureOf, getLargestPlace } from '../../ipc';
import { OwnItemRow, ownItemRowsClass } from '../../product/own-item-row/own-item-row';
import { useCore, useCoreStore } from '../../state/store';
import { focusHeading } from '../keep-focus';
import styles from './largest-screen.module.css';
import { type OwnPick } from './use-own-pick';

export type FoldersViewProps = {
  // The folder shown: `null` is the home.
  readonly folder: NodeId | null;
  readonly onFolder: (folder: NodeId | null) => void;
  readonly pick: OwnPick;
};

// Rows asked of the core at a time (the engine's `MAX_PAGE`).
const PAGE_SIZE = 100;

// The nearest ancestor that holds the screen's heading.
const screenOf = (element: HTMLElement | null): HTMLElement | null => {
  let node = element;

  while (node !== null && node.querySelector('h1') === null) {
    node = node.parentElement;
  }

  return node;
};

// The folders of the home, largest first: the way from the home to the opened folder, then its rows. A folder that
// opens has an Open button; the folder is the screen's, so it stays while the user looks at Files.
export const FoldersView = ({ folder, onFolder, pick }: FoldersViewProps): JSX.Element => {
  const store = useCoreStore();
  const [place, setPlace] = useState<null | OwnPlace>(null);
  const rootRef = useRef<HTMLDivElement>(null);
  // Answers of an older folder are dropped; the next page is on its way.
  const generationRef = useRef(0);
  const loadingMoreRef = useRef(false);
  // Focus follows an opened folder, not the first showing of the view.
  const openedRef = useRef(false);
  const removed = useCore((state) => state.removed);
  const ignoreChanges = useCore((state) => state.ignoreChanges);
  const state = folder?.state ?? null;
  const index = folder?.index ?? null;

  useEffect(() => {
    generationRef.current += 1;
    loadingMoreRef.current = false;
    const asked = generationRef.current;
    const at = state === null || index === null ? null : { index, state };

    getLargestPlace(at, 0, PAGE_SIZE).then(
      (answer) => {
        if (asked === generationRef.current) {
          setPlace(answer);

          if (openedRef.current) {
            focusHeading(screenOf(rootRef.current));
          }
        }
      },
      (error: unknown) => {
        if (asked === generationRef.current) {
          store.dispatch({ failure: failureOf(error), type: 'command-failed' });

          // A folder that cannot be read (one of an older scan, say) is left for the home.
          if (at !== null) {
            onFolder(null);
          }
        }
      },
    );

    return (): void => {
      generationRef.current += 1;
    };
  }, [ignoreChanges, index, onFolder, removed, state, store]);

  const more = (): void => {
    if (place === null || loadingMoreRef.current || place.rows.length >= place.total) {
      return;
    }
    const asked = generationRef.current;
    const offset = place.rows.length;
    loadingMoreRef.current = true;
    getLargestPlace(folder, offset, PAGE_SIZE).then(
      (answer) => {
        if (asked === generationRef.current) {
          loadingMoreRef.current = false;
          setPlace((shown) =>
            shown === null ? answer : { ...answer, rows: [...shown.rows.slice(0, offset), ...answer.rows] },
          );
        }
      },
      (error: unknown) => {
        if (asked === generationRef.current) {
          loadingMoreRef.current = false;
          store.dispatch({ failure: failureOf(error), type: 'command-failed' });
        }
      },
    );
  };

  const open = (next: NodeId | null): void => {
    openedRef.current = true;
    onFolder(next);
  };

  if (place === null) {
    return (
      <div ref={rootRef}>
        <p className={styles.state}>{t('ui.screen.largest.loading')}</p>
      </div>
    );
  }

  return (
    <div className={styles.folders} ref={rootRef}>
      <nav aria-label={t('ui.screen.largest.way')}>
        <ol className={styles.way}>
          {place.path.map((step, at) => {
            const last = at === place.path.length - 1;
            const name = at === 0 ? t('ui.screen.largest.home') : step.name;

            return (
              <li key={`${String(step.id.state)}:${String(step.id.index)}`}>
                {last ? (
                  <span aria-current="page">{name}</span>
                ) : (
                  <Button
                    onClick={() => {
                      open(at === 0 ? null : step.id);
                    }}
                    size="small"
                    variant="ghost"
                  >
                    {name}
                  </Button>
                )}
              </li>
            );
          })}
        </ol>
      </nav>
      <Surface elevation={1} padding="none" radius="lg">
        <ul aria-label={t('ui.screen.largest.folders')} className={`${styles.list ?? ''} ${ownItemRowsClass}`}>
          {place.rows.map((row) => (
            <OwnItemRow
              checked={pick.ticked(row)}
              key={`${String(row.node.id.state)}:${String(row.node.id.index)}`}
              {...(row.opens
                ? {
                    onOpen: (): void => {
                      open(row.node.id);
                    },
                  }
                : {})}
              onToggle={() => {
                pick.toggle(row);
              }}
              row={row}
            />
          ))}
        </ul>
      </Surface>
      {place.rows.length < place.total ? (
        <div className={styles.more}>
          <Button onClick={more}>{t('ui.screen.largest.more')}</Button>
        </div>
      ) : null}
    </div>
  );
};

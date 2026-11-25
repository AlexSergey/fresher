import { type JSX, type KeyboardEvent } from 'react';

import { SearchField } from '../../design/primitives/search-field/search-field';
import { t } from '../../i18n';
import { type Range, type SearchHit } from './settings-search';
import styles from './settings-search-view.module.css';

type SettingsSearchProps = {
  // Enter in the field opens the first hit, when there is one; Enter on the clear button only clears.
  readonly onEnter: () => void;
  readonly onQuery: (query: string) => void;
  readonly query: string;
};

// The field at the top of the sidebar; its border is the accent's while it holds text.
export const SettingsSearch = ({ onEnter, onQuery, query }: SettingsSearchProps): JSX.Element => (
  <div
    className={styles.search}
    data-filled={query === '' ? undefined : ''}
    onKeyDown={(event: KeyboardEvent<HTMLDivElement>) => {
      if (event.key === 'Enter' && event.target instanceof HTMLInputElement) {
        event.preventDefault();
        onEnter();
      }
    }}
    role="presentation"
  >
    <SearchField
      clearLabel={t('ui.screen.settings.search.clear')}
      hideLabel
      label={t('ui.screen.settings.search')}
      onChange={onQuery}
      placeholder={t('ui.screen.settings.search')}
      value={query}
    />
  </div>
);

// `text` with each of `ranges` wrapped in a mark.
const Marked = ({ ranges, text }: { readonly ranges: readonly Range[]; readonly text: string }): JSX.Element => {
  const parts: JSX.Element[] = [];
  let from = 0;

  for (const [start, end] of ranges) {
    parts.push(<span key={`t${String(from)}`}>{text.slice(from, start)}</span>);
    parts.push(<mark key={`m${String(start)}`}>{text.slice(start, end)}</mark>);
    from = end;
  }
  parts.push(<span key={`t${String(from)}`}>{text.slice(from)}</span>);

  return <>{parts}</>;
};

type SearchResultsProps = {
  readonly hits: readonly SearchHit[];
  readonly onOpen: (hit: SearchHit) => void;
  readonly query: string;
};

// The results in the page's place: how many, a hint, and one surface of hits, each a button that opens its setting.
export const SearchResults = ({ hits, onOpen, query }: SearchResultsProps): JSX.Element => {
  const shown = { kind: 'text', value: query.trim() } as const;

  return (
    <div className={styles.results}>
      <header className={styles.header}>
        <h1 className={styles.title}>
          {hits.length === 0
            ? t('ui.screen.settings.no-match', { query: shown })
            : t('ui.screen.settings.matches', { count: { kind: 'count', value: hits.length }, query: shown })}
        </h1>
        {hits.length === 0 ? null : <p className={styles.hint}>{t('ui.screen.settings.matches-hint')}</p>}
      </header>
      {hits.length === 0 ? null : (
        <ul className={styles.hits}>
          {hits.map((hit) => (
            <li
              key={`${hit.entry.page.kind === 'module' ? hit.entry.page.module : hit.entry.page.kind}/${hit.entry.key ?? ''}`}
            >
              <button
                className={styles.hit}
                onClick={() => {
                  onOpen(hit);
                }}
                type="button"
              >
                {hit.entry.place === '' ? null : <span className={styles.place}>{hit.entry.place}</span>}
                <span className={styles.label}>
                  <Marked ranges={hit.labelRanges} text={hit.entry.label} />
                </span>
                {hit.entry.key === null ? null : (
                  <span className={styles.description}>
                    <Marked ranges={hit.descriptionRanges} text={hit.entry.description} />
                  </span>
                )}
              </button>
            </li>
          ))}
        </ul>
      )}
    </div>
  );
};

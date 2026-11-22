import { type JSX, useState } from 'react';

import type { ItemKind, LargestPage, NodeId, Unchanged } from '../../ipc/bindings.generated';

import { PageHeader } from '../../design/layout/page-header/page-header';
import { Surface } from '../../design/layout/surface/surface';
import { Tabs } from '../../design/layout/tabs/tabs';
import { Toolbar } from '../../design/layout/toolbar/toolbar';
import { Button } from '../../design/primitives/button/button';
import { Select } from '../../design/primitives/select/select';
import { t, tText } from '../../i18n';
import { formatBytes } from '../../i18n/format-size';
import { OwnItemRow, ownItemRowsClass } from '../../product/own-item-row/own-item-row';
import { SelectionBar } from '../../product/selection-bar/selection-bar';
import { useNavigation } from '../../state/navigation';
import { summarize } from '../../state/selection-summary';
import { useCore, useCoreStore } from '../../state/store';
import { CoverageNotices } from '../permission/coverage-notices';
import { useScanActions } from '../use-scan-actions';
import { FoldersView } from './folders-view';
import styles from './largest-screen.module.css';
import { type LargestFilter, NO_FILTER, useLargest } from './use-largest';
import { type OwnPick, useOwnPick } from './use-own-pick';

type FilesProps = {
  readonly filter: LargestFilter;
  readonly more: () => void;
  readonly onFilter: (filter: LargestFilter) => void;
  readonly outOfDate: boolean;
  readonly page: LargestPage;
  readonly pick: OwnPick;
};

type View = 'files' | 'folders';

const UNCHANGED: readonly Unchanged[] = ['any', 'month', 'half-year', 'year', 'two-years'];

const keyOf = ({ index, state }: NodeId): string => `${String(state)}:${String(index)}`;

const count = (value: number): { readonly kind: 'count'; readonly value: number } => ({ kind: 'count', value });

// The three filters of the Files list; their choices and counts are the core's.
const Filters = ({
  filter,
  onFilter,
  page,
}: Pick<FilesProps, 'filter' | 'onFilter'> & { readonly page: LargestPage | null }): JSX.Element => {
  const kinds: readonly { readonly label: string; readonly value: 'any' | ItemKind }[] = [
    { label: t('ui.screen.largest.filter.kind.any'), value: 'any' },
    ...(page?.kinds ?? []).map(({ count: total, kind }) => ({
      label: t('ui.screen.largest.filter.kind.option', {
        count: count(total),
        kind: { kind: 'text', value: tText({ args: {}, key: `core.largest.kind.${kind}` }) },
      }),
      value: kind,
    })),
  ];
  const places = [
    { label: t('ui.screen.largest.filter.place.any'), value: 'any' },
    ...(page?.places ?? []).map(({ count: total, folder }) => ({
      label: t('ui.screen.largest.filter.place.option', {
        count: count(total),
        name: { kind: 'text', value: folder.name },
      }),
      value: keyOf(folder.id),
    })),
  ];

  return (
    <div className={styles.filters}>
      <Select<'any' | ItemKind>
        label={t('ui.screen.largest.filter.kind')}
        onChange={(value) => {
          onFilter({ ...filter, kind: value === 'any' ? null : value });
        }}
        options={kinds}
        value={filter.kind ?? 'any'}
      />
      <Select<string>
        label={t('ui.screen.largest.filter.place')}
        onChange={(value) => {
          onFilter({
            ...filter,
            place: (page?.places ?? []).find(({ folder }) => keyOf(folder.id) === value)?.folder.id ?? null,
          });
        }}
        options={places}
        value={filter.place === null ? 'any' : keyOf(filter.place)}
      />
      <Select<Unchanged>
        label={t('ui.screen.largest.filter.unchanged')}
        onChange={(value) => {
          onFilter({ ...filter, unchanged: value });
        }}
        options={UNCHANGED.map((value) => ({
          label: tText({ args: {}, key: `core.largest.unchanged.${value}` }),
          value,
        }))}
        value={filter.unchanged}
      />
    </div>
  );
};

// The Files list: the filters, the rows as the core sent them, what is left out of it, and Show more while rows remain.
const Files = ({ filter, more, onFilter, outOfDate, page, pick }: FilesProps): JSX.Element => (
  <>
    <Filters filter={filter} onFilter={onFilter} page={page} />
    {outOfDate ? <p className={styles.note}>{t('ui.screen.largest.changed')}</p> : null}
    {page.total === 0 ? (
      <p className={styles.state}>
        {t('ui.screen.largest.empty', { size: { kind: 'text', value: formatBytes(page.minSize) } })}
      </p>
    ) : (
      <Surface elevation={1} padding="none" radius="lg">
        <ul aria-label={t('ui.screen.largest.list')} className={`${styles.list ?? ''} ${ownItemRowsClass}`}>
          {page.rows.map((row) => (
            <OwnItemRow
              checked={pick.ticked(row)}
              key={keyOf(row.node.id)}
              onToggle={() => {
                pick.toggle(row);
              }}
              row={row}
            />
          ))}
        </ul>
      </Surface>
    )}
    {page.truncated ? <p className={styles.note}>{t('ui.screen.largest.truncated')}</p> : null}
    {page.rows.length < page.total ? (
      <div className={styles.more}>
        <Button onClick={more}>{t('ui.screen.largest.more')}</Button>
      </div>
    ) : null}
  </>
);

// "Where did my space go, file by file?": the largest files and folders of the last scan, largest first, with what
// the core says of each. Ticking an offered row asks the core to pick it and puts the finding in the selection;
// nothing is ticked when the screen opens and there is no Select all.
export const LargestScreen = (): JSX.Element => {
  const store = useCoreStore();
  const navigation = useNavigation();
  const { scan } = useScanActions();
  const [view, setView] = useState<View>('files');
  const [filter, setFilter] = useState<LargestFilter>(NO_FILTER);
  const [folder, setFolder] = useState<NodeId | null>(null);
  const { failed, more, page } = useLargest(filter);
  const scanId = useCore((state) => state.overview?.lastScan?.scan ?? null);
  const [seenScan, setSeenScan] = useState(scanId);

  // The folders of an older scan are not the new scan's: both views start again from the home.
  if (seenScan !== scanId) {
    setSeenScan(scanId);
    setFilter((now) => (now.place === null ? now : { ...now, place: null }));
    setFolder(null);
  }
  const pick = useOwnPick();
  const selection = useCore((state) => state.selection);
  const totals = useCore((state) => state.overview?.totals);
  const known = useCore((state) => state.overview !== null);
  const scanned = useCore((state) => state.overview?.lastScan != null);
  const scanning = useCore((state) => state.status.state === 'scanning');
  const outOfDate = useCore((state) => state.overview?.lastScan?.changedSince !== undefined);

  const summary = page === null || page.total === 0 ? {} : { subtitle: summaryOf(page) };

  return (
    <>
      <Toolbar />
      <PageHeader title={t('ui.screen.largest.title')} {...summary} />
      <div className={styles.content}>
        <CoverageNotices />
        {scanning ? (
          <p className={styles.state}>{t('ui.screen.largest.scanning')}</p>
        ) : known && !scanned ? (
          <div className={styles.state}>
            <p>{t('ui.screen.largest.no-scan')}</p>
            <Button onClick={scan} variant="primary">
              {t('ui.scan-hero.scan')}
            </Button>
          </div>
        ) : (
          <Tabs<View>
            label={t('ui.screen.largest.views')}
            onChange={setView}
            tabs={[
              { label: t('ui.screen.largest.view.files'), value: 'files' },
              { label: t('ui.screen.largest.view.folders'), value: 'folders' },
            ]}
            value={view}
          >
            {view === 'folders' ? (
              <FoldersView folder={folder} onFolder={setFolder} pick={pick} />
            ) : failed ? (
              <>
                <Filters filter={filter} onFilter={setFilter} page={null} />
                <p className={styles.state}>{t('ui.screen.largest.unavailable')}</p>
              </>
            ) : page === null ? (
              <>
                <Filters filter={filter} onFilter={setFilter} page={null} />
                <p className={styles.state}>{t('ui.screen.largest.loading')}</p>
              </>
            ) : (
              <Files filter={filter} more={more} onFilter={setFilter} outOfDate={outOfDate} page={page} pick={pick} />
            )}
          </Tabs>
        )}
      </div>
      <SelectionBar
        onClear={() => {
          store.dispatch({ type: 'clear' });
        }}
        onReview={() => {
          navigation.navigate({ kind: 'review' });
        }}
        summary={summarize(selection, totals ?? [])}
      />
    </>
  );
};

const summaryOf = (page: LargestPage): string =>
  t('ui.screen.largest.summary', {
    count: { kind: 'count', value: page.total },
    size: { kind: 'size', value: page.sizes.allocated },
  });

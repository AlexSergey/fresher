import { type JSX, useEffect, useMemo, useRef } from 'react';

import type { Finding } from '../../ipc/bindings.generated';

import { stories as storiesOf } from '../../gallery/story';
import { setCoreDictionary } from '../../i18n';
import { BUSY } from '../../test/demo-scan';
import {
  EXTERNAL_FINDING,
  FINDING_DICTIONARY,
  HIGH_FINDING,
  LONG_PATH_FINDING,
  LOW_FINDING,
  MEDIUM_FINDING,
  PARTIAL_FINDING,
  SAFE_FINDING,
} from '../../test/finding-fixtures';
import { largeScan } from '../../test/large-scan';
import { FindingList, type FindingListProps } from './finding-list';
import styles from './finding-list.stories.module.css';

const ROW = 52;

const ignore = (): void => undefined;

// A page of findings, largest first, as the core orders them.
const PAGE: readonly Finding[] = [
  HIGH_FINDING,
  EXTERNAL_FINDING,
  SAFE_FINDING,
  LOW_FINDING,
  PARTIAL_FINDING,
  MEDIUM_FINDING,
  LONG_PATH_FINDING,
].toSorted((a, b) => b.sizes.allocated - a.sizes.allocated);

const SELECTED = new Set([HIGH_FINDING.id, SAFE_FINDING.id]);

type Shown = Pick<FindingListProps, 'failure' | 'rowAt' | 'total'> & { readonly scrollTo?: number };

const List = ({ failure, rowAt, scrollTo = 0, total }: Shown): JSX.Element => {
  const frameRef = useRef<HTMLDivElement>(null);

  setCoreDictionary(FINDING_DICTIONARY);

  // The list scrolls as a person would; it hears of it through its scroll event.
  useEffect(() => {
    const viewport = frameRef.current?.querySelector('[role="list"]');

    if (viewport !== null && viewport !== undefined) {
      viewport.scrollTop = scrollTo * ROW;
    }
  }, [scrollTo]);

  return (
    <div className={styles.frame} ref={frameRef}>
      <FindingList
        failure={failure}
        isSelected={(finding) => SELECTED.has(finding.id)}
        label="Example caches"
        onRangeChange={ignore}
        onToggle={ignore}
        rowAt={rowAt}
        total={total}
      />
    </div>
  );
};

// A scan of 100,000 findings, made when the story is shown, scrolled to its middle.
const Large = (): JSX.Element => {
  const findings = useMemo(() => largeScan({ categories: 1, count: 100_000 }).findings, []);

  return <List failure={null} rowAt={(index) => findings[index]} scrollTo={50_000} total={findings.length} />;
};

const shown = (render: () => JSX.Element): { readonly render: () => JSX.Element; readonly width: number } => ({
  render,
  width: 720,
});

export const stories = storiesOf('FindingList', {
  empty: shown(() => <List failure={null} rowAt={() => undefined} total={0} />),
  failed: shown(() => <List failure={{ kind: 'refused', refusal: BUSY }} rowAt={() => undefined} total={undefined} />),
  'hundred-thousand-middle': shown(() => <Large />),
  loading: shown(() => <List failure={null} rowAt={() => undefined} total={undefined} />),
  page: shown(() => <List failure={null} rowAt={(index) => PAGE[index]} total={PAGE.length} />),
});

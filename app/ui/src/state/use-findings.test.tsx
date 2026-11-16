import { act, render, screen } from '@testing-library/react';
import { type JSX, useEffect } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { CommandFailed } from '../ipc';
import { BUSY, DEMO_FINDINGS } from '../test/demo-scan';
import { createFakeClock } from '../test/fake-clock';
import { pageOf } from '../test/page-of';
import { createFindingsSource, type FindingsSource, type FindingsView } from './findings';
import { FindingsContext, useFindings } from './use-findings';

const VIEW: FindingsView = { category: 'demo.caches', module: 'demo', order: 'largest' };

// A list as `VirtualList` would drive it: it asks for its first rows, then shows them.
const List = (): JSX.Element => {
  const { failure, onRangeChange, rowAt, total } = useFindings(VIEW);

  useEffect(() => {
    onRangeChange(0, 9);
  }, [onRangeChange]);

  return (
    <p>
      {total ?? 'unknown'} · {rowAt(0)?.id ?? 'loading'} · {rowAt(1)?.id ?? 'loading'} · {failure?.kind ?? 'fine'}
    </p>
  );
};

const renderList = (source: FindingsSource): void => {
  render(
    <FindingsContext value={source}>
      <List />
    </FindingsContext>,
  );
};

describe('useFindings', () => {
  describe('negative cases', () => {
    it('refuses to be used outside its context', () => {
      vi.spyOn(console, 'error').mockImplementation(() => {});

      expect(() => render(<List />)).toThrow('useFindings needs a FindingsContext');
    });

    it('shows nothing but the failure when the page is refused', async () => {
      renderList(
        createFindingsSource(() => Promise.reject(new CommandFailed(BUSY)), {
          clock: createFakeClock(),
          maxPages: 10,
          pageSize: 100,
        }),
      );

      expect(await screen.findByText('unknown · loading · loading · refused')).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows the rows once their page arrives', async () => {
      let answer: () => void = () => undefined;
      const source = createFindingsSource(
        (query) =>
          new Promise((resolve) => {
            answer = (): void => {
              resolve(pageOf(DEMO_FINDINGS, query));
            };
          }),
        { clock: createFakeClock(), maxPages: 10, pageSize: 100 },
      );
      renderList(source);
      expect(screen.getByText('unknown · loading · loading · fine')).toBeInTheDocument();

      await act(async () => {
        answer();
        await Promise.resolve();
      });

      expect(await screen.findByText('2 · 1 · 3 · fine')).toBeInTheDocument();
    });
  });
});

import { render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import type { Sizes } from '../../ipc/bindings.generated';

import { setMotion } from '../../test/animation';
import { ScanningCard, type ScanningCardProps } from './scanning-card';

const found = (allocated: number, completeness: Sizes['completeness'] = 'complete'): Sizes => ({
  allocated,
  completeness,
  logical: allocated,
  reclaimable: allocated,
});

const mount = (overrides: Partial<ScanningCardProps> = {}): void => {
  render(
    <ScanningCard
      found={found(1_900_000_000)}
      module="Developer caches"
      percent={62}
      volumeName="Macintosh HD"
      {...overrides}
    />,
  );
};

describe('ScanningCard', () => {
  afterEach(async () => {
    await setMotion(null);
  });

  describe('negative cases', () => {
    it.each(['reduced', 'off'])('with %s motion a new module shows without a fade', async (mode) => {
      await setMotion(mode);
      mount();

      expect(screen.getByText('Looking at Developer caches')).not.toHaveAttribute('data-fade');
    });

    it.each(['reduced', 'off'])('with %s motion the Looking at line does not shimmer', async (mode) => {
      await setMotion(mode);
      mount();

      expect(screen.getByText('Looking at Developer caches')).not.toHaveAttribute('data-shimmer');
    });

    it('no module known shows no Looking at line', () => {
      mount({ module: null });

      expect(screen.queryByText(/Looking at/u)).toBeNull();
    });

    it('a found size that is not known shows a dash and never 0', () => {
      mount({ found: found(0, 'unknown') });

      expect(screen.getByText('—')).toBeInTheDocument();
      expect(screen.queryByText('0 B')).toBeNull();
    });
  });

  describe('positive cases', () => {
    it('shows the name, percent, module and found size', () => {
      mount();

      expect(screen.getByRole('region', { name: 'Scanning' })).toBeInTheDocument();
      expect(screen.getByText('Scanning Macintosh HD')).toBeInTheDocument();
      expect(screen.getByText('62%')).toBeInTheDocument();
      expect(screen.getByText('Looking at Developer caches')).toBeInTheDocument();
      expect(screen.getAllByText('1.9 GB')).toHaveLength(2);
      expect(screen.getByText('found so far')).toBeInTheDocument();
    });

    it('with full motion a new module fades in', async () => {
      await setMotion('full');
      mount();

      expect(screen.getByText('Looking at Developer caches')).toHaveAttribute('data-fade');
    });

    it('with full motion the Looking at line shimmers', async () => {
      await setMotion('full');
      mount();

      expect(screen.getByText('Looking at Developer caches')).toHaveAttribute('data-shimmer');
    });

    it('a partial found size is a lower bound', () => {
      mount({ found: found(1_900_000_000, 'partial') });

      expect(screen.getByText('at least 1.9 GB')).toBeInTheDocument();
      expect(screen.getByText('1.9 GB+')).toBeInTheDocument();
    });

    it('its progress bar reads the percent', () => {
      mount();

      expect(screen.getByRole('progressbar', { name: 'Scanning, 62 percent' })).toHaveAttribute('aria-valuenow', '62');
    });
  });
});

import { render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { DEMO_TOTALS_FOUND, STORY_DICTIONARY } from '../scan-hero/story-data';
import { ModuleSection, type ModuleSectionProps } from './module-section';

const section = (overrides: Partial<ModuleSectionProps> = {}): ModuleSectionProps => ({
  found: DEMO_TOTALS_FOUND,
  module: 'demo',
  state: { kind: 'completed' },
  ...overrides,
});

beforeEach(() => {
  setCoreDictionary(STORY_DICTIONARY);
});

describe('ModuleSection', () => {
  describe('negative cases', () => {
    it('a finished module shows no failure notice', () => {
      render(<ModuleSection {...section()} />);

      expect(screen.queryByText('This part of the scan could not finish')).not.toBeInTheDocument();
      expect(screen.getByText('Done')).toBeInTheDocument();
    });

    it('an unknown size shows — and never 0', () => {
      render(
        <ModuleSection
          {...section({ found: { allocated: 0, completeness: 'unknown', logical: 0, reclaimable: 0 } })}
        />,
      );

      expect(screen.getByText('Size not known')).toBeInTheDocument();
      expect(screen.queryByText(/bytes/u)).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('the module section shows a failed module reason', () => {
      render(
        <ModuleSection {...section({ state: { kind: 'failed', reason: { args: {}, key: 'demo.error.failed' } } })} />,
      );

      expect(screen.getByText('This part of the scan could not finish')).toBeInTheDocument();
      expect(screen.getByText('The demo module could not build its example findings.')).toBeInTheDocument();
      expect(screen.getByText('Could not finish')).toBeInTheDocument();
    });

    it('is a region named by the module', () => {
      render(
        <ModuleSection {...section()}>
          <p>Content</p>
        </ModuleSection>,
      );

      expect(screen.getByRole('region', { name: 'Demo module' })).toHaveTextContent('Content');
      expect(screen.getByRole('heading', { name: 'Demo module' })).toBeInTheDocument();
      expect(screen.getByText('3.1 GB', { selector: '.u-visually-hidden' })).toBeInTheDocument();
    });

    it('shows a scanning module with its spinner and a stopped one', () => {
      const { rerender } = render(<ModuleSection {...section({ state: { kind: 'scanning' } })} />);

      expect(screen.getByText('Scanning…')).toBeInTheDocument();

      rerender(<ModuleSection {...section({ state: { kind: 'cancelled' } })} />);

      expect(screen.getByText('Stopped')).toBeInTheDocument();
    });

    it('shows a module not run for what it lacks as not available', () => {
      render(<ModuleSection {...section({ state: { kind: 'unavailable', missing: [{ kind: 'all-files' }] } })} />);

      expect(screen.getByText('Not available')).toBeInTheDocument();
      expect(screen.queryByText('This part of the scan could not finish')).not.toBeInTheDocument();
    });
  });
});

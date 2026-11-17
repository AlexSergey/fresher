import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import type { Text } from '../../ipc/bindings.generated';

import { setCoreDictionary } from '../../i18n';
import { ProblemText } from './problem-text';

const SENTENCE = 'The tool could not finish.';
const DETAIL = 'Error response from daemon: no such volume';

const plain: Text = { args: {}, key: 'demo.error.failed' };
const detailed: Text = { args: { detail: { kind: 'text', value: DETAIL } }, key: 'demo.error.failed' };

beforeEach(() => {
  setCoreDictionary({ 'demo.error.failed': SENTENCE });
});

describe('ProblemText', () => {
  describe('negative cases', () => {
    it('shows no disclosure without a detail', () => {
      render(<ProblemText text={plain} />);

      expect(screen.queryByRole('button')).not.toBeInTheDocument();
    });

    it('keeps the detail closed until asked', () => {
      render(<ProblemText text={detailed} />);

      const toggle = screen.getByRole('button', { name: 'Details' });

      expect(toggle).toHaveAttribute('aria-expanded', 'false');
      expect(screen.getByText(DETAIL)).not.toBeVisible();
      expect(toggle).toHaveAttribute('aria-controls', screen.getByText(DETAIL).id);
    });

    it('never shows the detail inside the sentence', () => {
      render(<ProblemText text={detailed} />);
      fireEvent.click(screen.getByRole('button', { name: 'Details' }));

      expect(screen.getByText(SENTENCE)).not.toHaveTextContent(DETAIL);
    });
  });

  describe('positive cases', () => {
    it('opens the detail on demand', () => {
      render(<ProblemText text={detailed} />);
      fireEvent.click(screen.getByRole('button', { name: 'Details' }));

      expect(screen.getByText(DETAIL)).toBeVisible();
      expect(screen.getByRole('button', { name: 'Details' })).toHaveAttribute('aria-expanded', 'true');
    });

    it("shows the core's sentence as it is", () => {
      render(<ProblemText text={plain} />);

      expect(screen.getByText(SENTENCE)).toBeInTheDocument();
    });
  });
});

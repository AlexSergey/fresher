import { render, screen, within } from '@testing-library/react';
import { beforeEach, describe, expect, it } from 'vitest';

import { setCoreDictionary } from '../../i18n';
import { DEMO_DICTIONARY, DEMO_LEFT_OUT } from '../../test/demo-scan';
import { LeftOutLines } from './left-out-lines';

beforeEach(() => {
  setCoreDictionary(DEMO_DICTIONARY);
});

describe('LeftOutLines', () => {
  describe('negative cases', () => {
    it('shows no line without a left-out', () => {
      render(<LeftOutLines counts={[]} />);

      expect(screen.queryByRole('list')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('shows one line per reason with its count and size', () => {
      const other = { ...DEMO_LEFT_OUT, items: 3, reason: { args: {}, key: 'demo.error.failed' } };
      setCoreDictionary({ ...DEMO_DICTIONARY, 'demo.error.failed': 'It could not be read.' });

      render(<LeftOutLines counts={[DEMO_LEFT_OUT, other]} />);

      const lines = within(screen.getByRole('list', { name: 'Left out of this list' })).getAllByRole('listitem');

      expect(lines).toHaveLength(2);
      expect(lines[0]).toHaveTextContent(
        '1 item left out (157.3 MB): The app that made it is open. Quit the app, then scan again.',
      );
      expect(lines[1]).toHaveTextContent('3 items left out (157.3 MB): It could not be read.');
    });
  });
});

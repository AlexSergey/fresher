import { fireEvent, render, screen } from '@testing-library/react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import { Notice } from './notice';

// A page that shows its notice only after it was there.
const Later = ({ tone }: { readonly tone: 'info' | 'warning' }): React.JSX.Element => {
  const [shown, setShown] = useState(false);

  return (
    <>
      <button
        onClick={() => {
          setShown(true);
        }}
        type="button"
      >
        Finish
      </button>
      {shown ? (
        <Notice live title="The scan is partial" tone={tone}>
          Some folders could not be read.
        </Notice>
      ) : null}
    </>
  );
};

describe('Notice', () => {
  describe('negative cases', () => {
    it('a notice present from the first render is not announced', () => {
      render(
        <Notice title="The scan is partial" tone="warning">
          Some folders could not be read.
        </Notice>,
      );

      expect(screen.queryByRole('alert')).not.toBeInTheDocument();
      expect(screen.queryByRole('status')).not.toBeInTheDocument();
      expect(screen.getByText('The scan is partial')).toBeInTheDocument();
    });

    it('has no action unless given', () => {
      render(
        <Notice title="Read-only" tone="info">
          The settings file is newer than this app.
        </Notice>,
      );

      expect(screen.queryByRole('button')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('a warning notice that appears is an alert', () => {
      render(<Later tone="warning" />);

      fireEvent.click(screen.getByRole('button', { name: 'Finish' }));

      expect(screen.getByRole('alert')).toHaveTextContent('The scan is partialSome folders could not be read.');
    });

    it('an info notice that appears is a status', () => {
      render(<Later tone="info" />);

      fireEvent.click(screen.getByRole('button', { name: 'Finish' }));

      expect(screen.getByRole('status')).toHaveTextContent('The scan is partial');
    });

    it('a danger notice that appears is an alert', () => {
      render(
        <Notice live title="Nothing was removed" tone="danger">
          The Trash could not be reached.
        </Notice>,
      );

      expect(screen.getByRole('alert')).toHaveAttribute('data-tone', 'danger');
    });

    it('runs its action', () => {
      const onClick = vi.fn();
      render(
        <Notice action={{ label: 'Show details', onClick }} title="The scan is partial" tone="warning">
          Some folders could not be read.
        </Notice>,
      );

      fireEvent.click(screen.getByRole('button', { name: 'Show details' }));

      expect(onClick).toHaveBeenCalledOnce();
    });
  });
});

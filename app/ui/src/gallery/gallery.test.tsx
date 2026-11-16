import { act, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it } from 'vitest';

import { Gallery } from './gallery';
import { REGISTRY } from './registry';
import { stories } from './story';

const STORIES = stories('Button', {
  primary: () => <button type="button">Primary</button>,
  secondary: { render: () => <button type="button">Secondary</button>, surface: 'overlay', width: 240 },
});

const go = (hash: string): void => {
  act(() => {
    window.location.hash = hash;
    window.dispatchEvent(new HashChangeEvent('hashchange'));
  });
};

afterEach(() => {
  go('');
});

describe('Gallery', () => {
  describe('negative cases', () => {
    it('an unknown hash shows the list, not an error', () => {
      go('#/Nothing/here');
      render(<Gallery stories={STORIES} />);

      expect(screen.getByRole('link', { name: 'Button' })).toBeInTheDocument();
      expect(screen.getByText('Pick a component to see its states.')).toBeInTheDocument();
    });

    it('a malformed hash shows the list', () => {
      go('#/%E0%A4%A');
      render(<Gallery stories={STORIES} />);

      expect(screen.getByText('Pick a component to see its states.')).toBeInTheDocument();
    });

    it('an unknown state of a known component shows the list', () => {
      go('#/Button/ghost');
      render(<Gallery stories={STORIES} />);

      expect(screen.queryByRole('button', { name: 'Primary' })).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('renders one story alone by its hash', () => {
      go('#/Button/secondary');
      render(<Gallery stories={STORIES} />);

      expect(screen.getByRole('figure', { name: 'Button/secondary' })).toHaveAttribute(
        'data-story',
        'Button/secondary',
      );
      expect(screen.getByRole('button', { name: 'Secondary' })).toBeInTheDocument();
      expect(screen.queryByRole('button', { name: 'Primary' })).not.toBeInTheDocument();
      expect(screen.queryByRole('navigation')).not.toBeInTheDocument();
    });

    it('shows every state of a component by its hash, and follows the hash', () => {
      render(<Gallery stories={STORIES} />);

      go('#/Button');

      expect(screen.getByRole('region', { name: 'primary' })).toBeInTheDocument();
      expect(screen.getByRole('region', { name: 'secondary' })).toBeInTheDocument();
      expect(screen.getByRole('link', { name: 'Button' })).toHaveAttribute('aria-current', 'page');
      expect(screen.getByRole('link', { name: 'secondary' })).toHaveAttribute('href', '#/Button/secondary');
    });

    it('renders every story of the registry', () => {
      for (const story of REGISTRY) {
        go(`#/${story.component}/${story.state}`);
        const { unmount } = render(<Gallery stories={REGISTRY} />);

        // The gallery's own main comes first; a story of the app shell holds another.
        expect(screen.getAllByRole('main')[0]).not.toBeEmptyDOMElement();

        unmount();
      }
    });
  });
});

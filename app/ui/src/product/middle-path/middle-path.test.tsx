import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { MiddlePath } from './middle-path';

// A piece that is drawn: the full path is read once, from the visually hidden copy.
const piece = (text: string): HTMLElement => screen.getByText(text, { selector: '[aria-hidden="true"]' });

describe('MiddlePath', () => {
  describe('negative cases', () => {
    it('a path of two segments or fewer is one piece', () => {
      render(<MiddlePath path="~/Downloads" />);

      expect(piece('~/Downloads')).toBeInTheDocument();
    });

    it('the home alone is one piece', () => {
      render(<MiddlePath path="~" />);

      expect(piece('~')).toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('a short path is whole', () => {
      render(<MiddlePath path="~/Code/app" />);

      expect(piece('~')).toBeInTheDocument();
      expect(piece('/Code/app')).toBeInTheDocument();
    });

    it('a long path keeps its first segment and its last two', () => {
      render(<MiddlePath path="~/Movies/Clients/2022/Raw/Takes" />);

      expect(piece('~/Movies/Clients/2022')).toBeInTheDocument();
      expect(piece('/Raw/Takes')).toBeInTheDocument();
    });

    it('the title and the accessible text hold the full path once', () => {
      render(<MiddlePath path="~/Movies/Clients/2022/Raw/Takes" />);

      expect(screen.getByTitle('~/Movies/Clients/2022/Raw/Takes')).toBeInTheDocument();
      expect(screen.getAllByText('~/Movies/Clients/2022/Raw/Takes')).toHaveLength(1);
    });
  });
});

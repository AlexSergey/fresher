import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Icon } from './icon';
import {
  Check,
  ChevronDown,
  ChevronRight,
  CircleAlert,
  type IconShape,
  Info,
  LayoutGrid,
  ListChecks,
  Minus,
  OctagonAlert,
  RotateCw,
  Search,
  Settings,
  Shield,
  ShieldCheck,
  Square,
  Trash2,
  TriangleAlert,
  X,
} from './icons';

// Every icon of the allowlist, by name: a name the pinned lucide-react no longer has fails here.
const ALLOWLIST: Readonly<Record<string, IconShape>> = {
  Check,
  ChevronDown,
  ChevronRight,
  CircleAlert,
  Info,
  LayoutGrid,
  ListChecks,
  Minus,
  OctagonAlert,
  RotateCw,
  Search,
  Settings,
  Shield,
  ShieldCheck,
  Square,
  Trash2,
  TriangleAlert,
  X,
};

describe('Icon', () => {
  describe('negative cases', () => {
    it('is hidden from assistive technology without a label', () => {
      const { container } = render(<Icon icon={ShieldCheck} />);

      expect(screen.queryByRole('img')).not.toBeInTheDocument();
      // eslint-disable-next-line testing-library/no-container, testing-library/no-node-access -- a hidden icon has no role to query by
      expect(container.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
    });

    it('does not accept a size outside the scale', () => {
      // @ts-expect-error -- 18 is not a size of the scale.
      render(<Icon icon={ShieldCheck} size={18} />);

      expect(screen.queryByRole('img')).not.toBeInTheDocument();
    });
  });

  describe('positive cases', () => {
    it('has the role img and its label when labelled', () => {
      render(<Icon icon={ShieldCheck} label="Safe" />);

      const icon = screen.getByRole('img', { name: 'Safe' });

      expect(icon).not.toHaveAttribute('aria-hidden');
    });

    it('draws with the current colour at the design stroke and size', () => {
      render(<Icon icon={ShieldCheck} label="Safe" size={24} />);

      const icon = screen.getByRole('img', { name: 'Safe' });

      expect(icon).toHaveAttribute('stroke', 'currentColor');
      expect(icon).toHaveAttribute('stroke-width', '1.75');
      expect(icon).toHaveAttribute('width', '24');
    });

    it.each(Object.entries(ALLOWLIST))('draws %s', (name, shape) => {
      render(<Icon icon={shape} label={name} />);

      expect(screen.getByRole('img', { name })).toBeInTheDocument();
    });

    it('is 16 pixels by default and passes its class on', () => {
      render(<Icon className="badge-icon" icon={ShieldCheck} label="Safe" />);

      const icon = screen.getByRole('img', { name: 'Safe' });

      expect(icon).toHaveAttribute('width', '16');
      expect(icon).toHaveClass('badge-icon');
    });
  });
});

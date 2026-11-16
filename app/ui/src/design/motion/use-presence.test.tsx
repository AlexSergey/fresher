import type { JSX } from 'react';

import { render, screen } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { usePresence } from '.';
import { fakeReducedMotion, recordAnimations, setMotion } from '../../test/animation';

const EXIT: readonly Keyframe[] = [
  { opacity: 1, transform: 'none' },
  { opacity: 0, transform: 'translateY(4px)' },
];

const Panel = ({ open }: { readonly open: boolean }): JSX.Element | null => {
  const { mounted, ref } = usePresence<HTMLDivElement>(open, { duration: 120, exit: EXIT });

  return mounted ? <div ref={ref}>Panel</div> : null;
};

beforeEach(() => {
  fakeReducedMotion(false);
});

afterEach(async () => {
  await setMotion(null);
});

describe('usePresence', () => {
  describe('negative cases', () => {
    it('a closed element that was never open is not mounted', () => {
      const animations = recordAnimations();

      render(<Panel open={false} />);

      expect(screen.queryByText('Panel')).not.toBeInTheDocument();
      expect(animations).toHaveLength(0);
    });

    it('reopening during the exit keeps the element', async () => {
      const animations = recordAnimations();
      const { rerender } = render(<Panel open />);

      rerender(<Panel open={false} />);
      rerender(<Panel open />);
      await animations[0]?.finish();

      expect(animations[0]?.cancelled()).toBe(true);
      expect(screen.getByText('Panel')).toBeInTheDocument();
    });

    it('motion off unmounts at once, without an exit', async () => {
      await setMotion('off');
      const animations = recordAnimations();
      const { rerender } = render(<Panel open />);

      rerender(<Panel open={false} />);

      expect(screen.queryByText('Panel')).not.toBeInTheDocument();
      expect(animations).toHaveLength(0);
    });
  });

  describe('positive cases', () => {
    it('unmounts after the exit finishes', async () => {
      const animations = recordAnimations();
      const { rerender } = render(<Panel open />);

      rerender(<Panel open={false} />);

      expect(screen.getByText('Panel')).toBeInTheDocument();
      expect(animations[0]?.keyframes).toStrictEqual(EXIT);
      expect(animations[0]?.options).toMatchObject({ duration: 120, fill: 'forwards' });

      await animations[0]?.finish();

      expect(screen.queryByText('Panel')).not.toBeInTheDocument();
    });

    it('reduced motion fades out, at most for --duration-fast', async () => {
      await setMotion('reduced');
      const animations = recordAnimations();
      const Slow = ({ open }: { readonly open: boolean }): JSX.Element | null => {
        const { mounted, ref } = usePresence<HTMLDivElement>(open, { duration: 400, exit: EXIT });

        return mounted ? <div ref={ref}>Slow</div> : null;
      };
      const { rerender } = render(<Slow open />);

      rerender(<Slow open={false} />);

      expect(animations[0]?.keyframes).toStrictEqual([{ opacity: 1 }, { opacity: 0 }]);
      expect(animations[0]?.options).toMatchObject({ duration: 120 });
    });

    it('can open again after it has left', async () => {
      const animations = recordAnimations();
      const { rerender } = render(<Panel open />);

      rerender(<Panel open={false} />);
      await animations[0]?.finish();
      rerender(<Panel open />);

      expect(screen.getByText('Panel')).toBeInTheDocument();
    });
  });
});

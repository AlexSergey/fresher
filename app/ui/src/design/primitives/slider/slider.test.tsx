import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { Slider } from './slider';

const slider = (): HTMLInputElement => screen.getByRole('slider', { name: 'Old after, days' });

describe('Slider', () => {
  describe('negative cases', () => {
    it('a disabled slider sends nothing', () => {
      const onChange = vi.fn();
      const onCommit = vi.fn();
      render(
        <Slider
          disabled
          label="Old after, days"
          max={365}
          min={1}
          onChange={onChange}
          onCommit={onCommit}
          value={90}
        />,
      );

      fireEvent.change(slider(), { target: { value: '120' } });
      fireEvent.pointerUp(slider());
      fireEvent.keyUp(slider());

      expect(onChange).not.toHaveBeenCalled();
      expect(onCommit).not.toHaveBeenCalled();
    });

    it('a value outside its bounds is shown at the nearest bound', () => {
      const { rerender } = render(<Slider label="Old after, days" max={365} min={1} onChange={vi.fn()} value={900} />);

      expect(slider()).toHaveValue('365');

      rerender(<Slider label="Old after, days" max={365} min={1} onChange={vi.fn()} value={-4} />);

      expect(slider()).toHaveValue('1');
    });

    it('lets go without moving commits nothing', () => {
      const onCommit = vi.fn();
      render(<Slider label="Old after, days" max={365} min={1} onChange={vi.fn()} onCommit={onCommit} value={90} />);

      fireEvent.pointerUp(slider());
      fireEvent.keyUp(slider());

      expect(onCommit).not.toHaveBeenCalled();
    });
  });

  describe('positive cases', () => {
    it('shows its minimum and maximum', () => {
      render(<Slider label="Old after, days" max={365} min={1} onChange={vi.fn()} value={90} />);

      expect(screen.getByText('1')).toBeInTheDocument();
      expect(screen.getByText('365')).toBeInTheDocument();
      expect(slider()).toHaveAttribute('step', '1');
    });

    it('moving it sends the whole number', () => {
      const onChange = vi.fn();
      render(<Slider label="Old after, days" max={365} min={1} onChange={onChange} value={90} />);

      fireEvent.change(slider(), { target: { value: '120' } });

      expect(onChange).toHaveBeenLastCalledWith(120);
    });

    it('names the text that describes it', () => {
      render(<Slider describedBy="hint" label="Old after, days" max={365} min={1} onChange={vi.fn()} value={90} />);

      expect(slider()).toHaveAttribute('aria-describedby', 'hint');
    });

    it('commits once when the pointer lets go', () => {
      const onCommit = vi.fn();
      render(<Slider label="Old after, days" max={365} min={1} onChange={vi.fn()} onCommit={onCommit} value={90} />);

      fireEvent.input(slider(), { target: { value: '120' } });
      fireEvent.input(slider(), { target: { value: '130' } });

      expect(onCommit).not.toHaveBeenCalled();

      fireEvent.pointerUp(slider());
      fireEvent.change(slider());

      expect(onCommit).toHaveBeenCalledTimes(1);
      expect(onCommit).toHaveBeenCalledWith(130);
    });

    it('commits once when the key is released', () => {
      const onCommit = vi.fn();
      render(<Slider label="Old after, days" max={365} min={1} onChange={vi.fn()} onCommit={onCommit} value={90} />);

      fireEvent.input(slider(), { target: { value: '91' } });
      fireEvent.keyUp(slider());
      fireEvent.keyUp(slider());

      expect(onCommit).toHaveBeenCalledTimes(1);
      expect(onCommit).toHaveBeenCalledWith(91);
    });
  });
});

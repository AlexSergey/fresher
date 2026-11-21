import type { JSX } from 'react';

import type { Sizes } from '../../ipc/bindings.generated';

import { AnimatedNumber } from '../../design/data/animated-number/animated-number';
import { SizeText } from '../../design/data/size-text/size-text';
import { VisuallyHidden } from '../../design/primitives/visually-hidden/visually-hidden';
import { formatSize } from '../../i18n/format-size';
import { honestSize } from '../sizes';

// A part's honest size; during a scan it counts toward its value for the eye, while assistive technology reads the
// honest size ("at least 42 GB").
export const PartSize = ({ scanning, sizes }: { readonly scanning: boolean; readonly sizes: Sizes }): JSX.Element =>
  scanning ? (
    <>
      <span aria-hidden="true">
        <AnimatedNumber format={(value) => formatSize(value, sizes.completeness)} value={sizes.allocated} />
      </span>
      <VisuallyHidden>{honestSize(sizes).accessibleText}</VisuallyHidden>
    </>
  ) : (
    <SizeText {...honestSize(sizes)} />
  );

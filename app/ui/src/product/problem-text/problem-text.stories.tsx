import type { JSX } from 'react';

import { stories as storiesOf } from '../../gallery/story';
import { withTexts } from '../scan-hero/story-data';
import { ProblemText } from './problem-text';

const shown = (args: Record<string, { readonly kind: 'text'; readonly value: string }>): (() => JSX.Element) =>
  withTexts(() => <ProblemText text={{ args, key: 'demo.error.failed' }} />);

export const stories = storiesOf('ProblemText', {
  'with-detail': {
    render: shown({ detail: { kind: 'text', value: 'Error response from daemon: no such volume' } }),
    width: 480,
  },
  'without-detail': { render: shown({}), width: 480 },
});

import { describe, expect, it } from 'vitest';

import { collectStories, REGISTRY } from './registry';
import { stories, storyId } from './story';

const Sample = (): React.JSX.Element => <p>Sample</p>;

describe('the gallery registry', () => {
  describe('negative cases', () => {
    it('refuses two stories with the same component and state', () => {
      expect(() =>
        collectStories({
          'a.stories.tsx': { stories: stories('Button', { primary: Sample }) },
          'b.stories.tsx': { stories: stories('Button', { primary: Sample }) },
        }),
      ).toThrow('Two stories are named Button/primary');
    });

    it('refuses a stories file without its stories export', () => {
      expect(() => collectStories({ 'c.stories.tsx': {} })).toThrow('c.stories.tsx has no `export const stories`');
    });
  });

  describe('positive cases', () => {
    it('finds a stories file by its glob', () => {
      expect(REGISTRY.map((story) => storyId(story))).toContain('Tokens/palette');
    });

    it('sorts the stories by component and state', () => {
      const sorted = collectStories({
        'a.stories.tsx': { stories: stories('Switch', { on: Sample }) },
        'b.stories.tsx': { stories: stories('Button', { primary: Sample, secondary: Sample }) },
      });

      expect(sorted.map((story) => storyId(story))).toStrictEqual(['Button/primary', 'Button/secondary', 'Switch/on']);
    });

    it('keeps the surface and the width a story asks for', () => {
      const [story] = stories('Card', { hover: { render: Sample, surface: 'raised', width: 320 } });

      expect(story).toMatchObject({ component: 'Card', state: 'hover', surface: 'raised', width: 320 });
    });
  });
});

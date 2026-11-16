import type { Segment } from './segment';

// Segments for the charts' stories: what a scan of a developer's Mac might find, largest first.
export const STORY_SEGMENTS: readonly Segment[] = [
  { key: 'xcode', label: 'Xcode', level: 'safe', size: '38.2 GB', value: 382 },
  { key: 'docker', label: 'Docker', level: 'medium', size: '21.6 GB', value: 216 },
  { key: 'simulators', label: 'Simulators', level: 'low', size: '14.9 GB', value: 149 },
  { key: 'node', label: 'Node modules', level: 'safe', size: '9.4 GB', value: 94 },
  { key: 'downloads', label: 'Old downloads', level: 'high', size: '7.1 GB', value: 71 },
  { key: 'caches', label: 'App caches', level: 'safe', size: '5.3 GB', value: 53 },
  { key: 'logs', label: 'Logs', level: 'safe', size: '2.2 GB', value: 22 },
  { key: 'gradle', label: 'Gradle', level: 'low', size: '1.8 GB', value: 18 },
  { key: 'mail', label: 'Mail attachments', level: 'medium', size: '1.1 GB', value: 11 },
  { key: 'brew', label: 'Homebrew', level: 'safe', size: '0.6 GB', value: 6 },
  { key: 'trash', label: 'Trash', level: 'high', size: '0.3 GB', value: 3 },
  { key: 'crash', label: 'Crash reports', level: 'safe', size: '12 MB', value: 0.12 },
];

// The sum of the story segments.
export const STORY_TOTAL = STORY_SEGMENTS.reduce((sum, { value }) => sum + value, 0);

import type { Segment } from './segment';

// Segments for the charts' stories: what a scan of a developer's Mac might find, largest first.
export const STORY_SEGMENTS: readonly Segment[] = [
  { key: 'xcode', label: 'Xcode', size: '38.2 GB', tone: 'safe', value: 382 },
  { key: 'docker', label: 'Docker', size: '21.6 GB', tone: 'medium', value: 216 },
  { key: 'simulators', label: 'Simulators', size: '14.9 GB', tone: 'low', value: 149 },
  { key: 'node', label: 'Node modules', size: '9.4 GB', tone: 'safe', value: 94 },
  { key: 'downloads', label: 'Old downloads', size: '7.1 GB', tone: 'high', value: 71 },
  { key: 'caches', label: 'App caches', size: '5.3 GB', tone: 'safe', value: 53 },
  { key: 'logs', label: 'Logs', size: '2.2 GB', tone: 'safe', value: 22 },
  { key: 'gradle', label: 'Gradle', size: '1.8 GB', tone: 'low', value: 18 },
  { key: 'mail', label: 'Mail attachments', size: '1.1 GB', tone: 'medium', value: 11 },
  { key: 'brew', label: 'Homebrew', size: '0.6 GB', tone: 'safe', value: 6 },
  { key: 'trash', label: 'Trash', size: '0.3 GB', tone: 'high', value: 3 },
  { key: 'crash', label: 'Crash reports', size: '12 MB', tone: 'safe', value: 0.12 },
];

// The sum of the story segments.
export const STORY_TOTAL = STORY_SEGMENTS.reduce((sum, { value }) => sum + value, 0);

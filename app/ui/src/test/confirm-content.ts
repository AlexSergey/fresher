import type { DangerLevel, Finding, Sizes } from '../ipc/bindings.generated';
import type { ConfirmContent } from '../product/confirm-dialog/confirm-dialog';

import { DEMO_FINDINGS } from './demo-scan';
import { findingOf, HIGH_FINDING, LONG_PATH_FINDING, MEDIUM_FINDING } from './finding-fixtures';

// What the confirmation shows, built from findings as plan 008's cleanup plan will be: shared by the unit tests and
// the stories.

const LEVELS: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high'];

const highestOf = (findings: readonly Finding[]): DangerLevel =>
  findings.reduce<DangerLevel>(
    (highest, finding) => (LEVELS.indexOf(finding.danger) > LEVELS.indexOf(highest) ? finding.danger : highest),
    'safe',
  );

const sumOf = (findings: readonly Finding[]): Sizes => {
  const bytes = findings.reduce((sum, finding) => sum + finding.sizes.allocated, 0);

  return { allocated: bytes, completeness: 'complete', logical: bytes };
};

const subjectOf = (finding: Finding): string =>
  finding.subject.kind === 'path'
    ? finding.subject.value
    : `${finding.subject.value.tool}: ${finding.subject.value.id}`;

// The findings grouped by category, in the order the categories first appear.
const contentOf = (findings: readonly Finding[]): ConfirmContent => {
  const categories = [...new Set(findings.map((finding) => finding.category))];

  return {
    count: findings.length,
    groups: categories.map((category) => {
      const own = findings.filter((finding) => finding.category === category);

      return {
        category,
        count: own.length,
        danger: highestOf(own),
        items: own.map((finding) => ({
          danger: finding.danger,
          id: finding.id,
          sizes: finding.sizes,
          subject: subjectOf(finding),
          title: finding.title,
        })),
        sizes: sumOf(own),
      };
    }),
    highest: highestOf(findings),
    sizes: sumOf(findings),
  };
};

// The demo scan's three findings: two caches (Safe) and an installer (Low), 2.57 GB.
export const SHORT_CONTENT: ConfirmContent = contentOf(DEMO_FINDINGS);

// One finding at `level`, or the short content with a High project folder added.
export const confirmContentOf = (level: 'high' | 'safe'): ConfirmContent =>
  level === 'safe'
    ? contentOf(DEMO_FINDINGS.slice(0, 1))
    : contentOf([...DEMO_FINDINGS, MEDIUM_FINDING, LONG_PATH_FINDING, HIGH_FINDING]);

// 240 caches: more than the 50 a group lists without a virtual list.
export const LONG_CONTENT: ConfirmContent = contentOf(
  Array.from({ length: 240 }, (_, at) =>
    findingOf({
      bytes: 4_000_000 + ((at * 7_919) % 90) * 1_000_000,
      id: 100 + at,
      level: 'safe',
      name: `cache-${String(at + 1)}`,
    }),
  ),
);

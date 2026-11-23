import type { DangerLevel } from '../../ipc/bindings.generated';

import { type IconShape, OctagonAlert, Shield, ShieldCheck, TriangleAlert } from '../../design/icons/icons';

// The zones in the order they are shown: the safest first, the riskiest last.
export const ZONE_ORDER: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high'];

// Review's two parts: a grouping of the levels for display only; each part's name and meaning are the core's
// (`core.danger.part.<part>.*`).
export type DangerPart = 'danger' | 'safe';

export const PART_OF: Readonly<Record<DangerLevel, DangerPart>> = {
  high: 'danger',
  low: 'safe',
  medium: 'danger',
  safe: 'safe',
};

// The parts in the order they are shown: the safe part first.
export const PART_ORDER: readonly DangerPart[] = ['safe', 'danger'];

// The icons escalate in shape with the level, as the danger badge's do.
export const LEVEL_ICONS: Readonly<Record<DangerLevel, IconShape>> = {
  high: OctagonAlert,
  low: Shield,
  medium: TriangleAlert,
  safe: ShieldCheck,
};

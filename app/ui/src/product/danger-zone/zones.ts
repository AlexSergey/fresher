import type { DangerLevel } from '../../ipc/bindings.generated';

import { type IconShape, OctagonAlert, Shield, ShieldCheck, TriangleAlert } from '../../design/icons/icons';

// The zones in the order they are shown: the safest first, the riskiest last.
export const ZONE_ORDER: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high'];

// The icons escalate in shape with the level, as the danger badge's do.
export const LEVEL_ICONS: Readonly<Record<DangerLevel, IconShape>> = {
  high: OctagonAlert,
  low: Shield,
  medium: TriangleAlert,
  safe: ShieldCheck,
};

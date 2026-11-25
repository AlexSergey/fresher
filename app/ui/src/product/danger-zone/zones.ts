import type { DangerLevel } from '../../ipc/bindings.generated';

import { Eye, type IconShape, Shield, ShieldAlert, ShieldCheck } from '../../design/icons/icons';

// The zones in the order they are shown: the safest first, the riskiest last.
export const ZONE_ORDER: readonly DangerLevel[] = ['safe', 'low', 'medium', 'high'];

// The levels whose items the user ticks one by one, "Look before removing": a group or a category of them is never
// selected whole for the user. Their names and lines are the core's.
export const BY_HAND: ReadonlySet<DangerLevel> = new Set(['high', 'medium']);

// The icons of the group headings: safe, low, medium and high each have their own shield or eye.
export const GROUP_ICONS: Readonly<Record<DangerLevel, IconShape>> = {
  high: ShieldAlert,
  low: Shield,
  medium: Eye,
  safe: ShieldCheck,
};

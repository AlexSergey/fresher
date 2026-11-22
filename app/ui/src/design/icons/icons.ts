// The allowlist of the icons the app uses: the only file that imports lucide-react, so the bundle holds only these and
// an unused one fails knip. A later component adds the names it needs, one line each.
export type { LucideIcon as IconShape } from 'lucide-react';
export {
  AppWindow,
  ArrowDownWideNarrow,
  Check,
  ChevronDown,
  ChevronLeft,
  ChevronRight,
  CircleAlert,
  CircleCheck,
  FolderSearch,
  History,
  Info,
  KeyRound,
  LayoutGrid,
  ListChecks,
  Lock,
  LockKeyhole,
  Minus,
  // The danger levels escalate in shape: Safe, Low, Medium, High.
  OctagonAlert,
  Plus,
  RotateCw,
  Search,
  Settings,
  Shield,
  ShieldCheck,
  Square,
  Trash2,
  TriangleAlert,
  X,
} from 'lucide-react';

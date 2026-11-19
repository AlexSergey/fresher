import type { Capability } from '../../ipc/bindings.generated';

// The id the core's texts and switches of a capability carry (`core.access.<id>.…`); `null` for a capability that has
// no texts of its own (a tool).
export const idOf = (capability: Capability): null | string => {
  switch (capability.kind) {
    case 'all-files':
    case 'other-apps-data':
      return capability.kind;
    case 'folder':
      return capability.value;
    case 'tool':
      return null;
    case 'volume':
      return capability.value === 'removable' ? 'removable-volume' : 'network-volume';
  }
};

// Whether the user turns the capability on with a switch in the settings (the four opt-ins).
export const isOptIn = (capability: Capability): boolean =>
  capability.kind === 'folder' || capability.kind === 'other-apps-data';

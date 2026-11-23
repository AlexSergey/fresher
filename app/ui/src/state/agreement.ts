import type { Acceptance, AgreementView, AppId, UninstallChoice } from '../ipc/bindings.generated';

export type AgreementAction =
  // The sheet was closed: Not now, Escape, or the acceptance went through.
  | { readonly next: AgreementNext; readonly type: 'agreement-asked' }
  | { readonly type: 'agreement-dismissed' }
  | { readonly type: 'agreement-loaded'; readonly view: AgreementView };

// What to do once the agreement is accepted: the cleanup step that asked for it.
export type AgreementNext =
  { readonly app: AppId; readonly choice: UninstallChoice; readonly kind: 'uninstall' } | { readonly kind: 'prepare' };

// The agreement as `get_agreement` answered, and the step that waits for the user to accept it (`null` when the sheet
// is closed).
export type AgreementState = {
  readonly asking: AgreementNext | null;
  readonly view: AgreementView | null;
};

export const NO_AGREEMENT: AgreementState = { asking: null, view: null };

export const reduceAgreement = (state: AgreementState, action: AgreementAction): AgreementState => {
  switch (action.type) {
    case 'agreement-asked':
      return { ...state, asking: action.next };
    case 'agreement-dismissed':
      return state.asking === null ? state : { ...state, asking: null };
    case 'agreement-loaded':
      return { ...state, view: action.view };
  }
};

// Whether the user accepted the current version of the agreement.
export const isAccepted = (view: AgreementView): view is AgreementView & { readonly accepted: Acceptance } =>
  view.accepted?.version === view.version;

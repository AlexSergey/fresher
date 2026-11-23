import { type JSX, useState } from 'react';

import { AgreementSheet } from '../product/agreement-sheet/agreement-sheet';
import { LicenceDialog } from '../product/licence-dialog/licence-dialog';
import { acceptAgreementAndGoOn, declineAgreement } from '../state/cleanup-actions';
import { useCore, useCoreStore } from '../state/store';

// The agreement the first cleanup asks for, shown over any screen; the full licence opens from it.
export const AgreementGate = (): JSX.Element | null => {
  const store = useCoreStore();
  const view = useCore((state) => state.agreement.view);
  const asking = useCore((state) => state.agreement.asking !== null);
  const [licence, setLicence] = useState(false);

  // The licence opened from the sheet closes with it, so the next opening shows the sheet alone.
  if (!asking && licence) {
    setLicence(false);
  }

  if (view === null) {
    return null;
  }

  return (
    <>
      <AgreementSheet
        onAccept={() => {
          acceptAgreementAndGoOn(store);
        }}
        onDecline={() => {
          declineAgreement(store);
        }}
        onLicence={() => {
          setLicence(true);
        }}
        open={asking}
        view={view}
      />
      <LicenceDialog
        onClose={() => {
          setLicence(false);
        }}
        open={asking && licence}
      />
    </>
  );
};

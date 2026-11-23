//! A confirmation cannot be written in code: it exists only as the JSON the UI sends.

use fresher_core::{ByteSize, Confirmation, DangerLevel, Method, PlanId};

fn main() {
    let _ = Confirmation {
        plan: PlanId::new(1),
        count: 3,
        allocated: ByteSize::new(4096),
        highest: DangerLevel::Safe,
        method: Method::Permanent,
        high_risk_understood: false,
    };
}

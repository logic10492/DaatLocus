use serde::{Deserialize, Serialize};

use crate::activity_event::{
    ActivatePrimitiveActivityDescriptor, CreatePrimitiveSpecActivityDescriptor,
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivatePrimitiveActivityData {
    pub primitive_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatePrimitiveSpecActivityData {
    pub primitive_id: String,
}

impl From<ActivatePrimitiveActivityDescriptor> for ActivatePrimitiveActivityData {
    fn from(data: ActivatePrimitiveActivityDescriptor) -> Self {
        ActivatePrimitiveActivityData {
            primitive_id: data.primitive_id,
        }
    }
}

impl From<CreatePrimitiveSpecActivityDescriptor> for CreatePrimitiveSpecActivityData {
    fn from(data: CreatePrimitiveSpecActivityDescriptor) -> Self {
        CreatePrimitiveSpecActivityData {
            primitive_id: data.primitive_id,
        }
    }
}

use serde::{Deserialize, Serialize};

use crate::tool_ui::{ActivatePrimitiveUiData, CreatePrimitiveSpecUiData};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivatePrimitiveActivityCell {
    pub workflow_id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CreatePrimitiveSpecActivityCell {
    pub workflow_id: String,
}

impl From<ActivatePrimitiveUiData> for ActivatePrimitiveActivityCell {
    fn from(data: ActivatePrimitiveUiData) -> Self {
        ActivatePrimitiveActivityCell {
            workflow_id: data.workflow_id,
        }
    }
}

impl From<CreatePrimitiveSpecUiData> for CreatePrimitiveSpecActivityCell {
    fn from(data: CreatePrimitiveSpecUiData) -> Self {
        CreatePrimitiveSpecActivityCell {
            workflow_id: data.workflow_id,
        }
    }
}

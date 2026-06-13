use serde::{Deserialize, Serialize};

use crate::tool_ui::{PlanStepUiStatus, PlanUiData, PlanUiKind};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanActivityCell {
    #[serde(default)]
    pub kind: PlanUiKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub explanation: Option<String>,
    pub steps: Vec<PlanStepActivityCell>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlanStepActivityCell {
    pub status: PlanStepDisplayStatus,
    pub text: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlanStepDisplayStatus {
    Pending,
    InProgress,
    Completed,
}

impl From<PlanUiData> for PlanActivityCell {
    fn from(data: PlanUiData) -> Self {
        PlanActivityCell {
            kind: data.kind,
            explanation: data.explanation,
            steps: data
                .steps
                .into_iter()
                .map(|step| PlanStepActivityCell {
                    status: match step.status {
                        PlanStepUiStatus::Pending => PlanStepDisplayStatus::Pending,
                        PlanStepUiStatus::InProgress => PlanStepDisplayStatus::InProgress,
                        PlanStepUiStatus::Completed => PlanStepDisplayStatus::Completed,
                    },
                    text: step.text,
                })
                .collect(),
        }
    }
}

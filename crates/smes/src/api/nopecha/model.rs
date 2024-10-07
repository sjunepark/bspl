use crate::error::NopechaErrorBody;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum SubmitCaptchaResponse {
    Answer(Key),
    Error(NopechaErrorBody),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Key {
    pub(crate) data: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum GetAnswerResponse {
    Answer(Answer),
    Error(NopechaErrorBody),
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Answer {
    pub(crate) data: [String; 1],
}

impl Answer {
    pub(crate) fn data(&self) -> &str {
        &self.data[0]
    }
}

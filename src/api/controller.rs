#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SwitchoverRequestInput {
    pub(crate) requested_by: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct AcceptedResponse {
    pub(crate) accepted: bool,
}

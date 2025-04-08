use crate::backend::app::{FedResultStream, FederatedRequestType};

pub(crate) fn process_static(hash: u64, request: &FederatedRequestType) -> Option<FedResultStream> {
    None
}

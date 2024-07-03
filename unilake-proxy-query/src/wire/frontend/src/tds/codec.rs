mod batch_request;
mod column_data;
mod decode;
mod encode;
mod guid;
mod header;
mod login;
mod packet;
mod pre_login;
mod row;
mod rpc_request;
mod token;
mod tokenfactory;
mod type_info;

pub use column_data::*;
pub use header::*;
pub use login::*;
pub use packet::*;
pub use pre_login::*;
pub use row::*;
pub use token::*;
pub use type_info::*;

const HEADER_BYTES: usize = 8;
const ALL_HEADERS_LEN_TX: usize = 22;

#[derive(Debug)]
#[repr(u16)]
#[allow(dead_code)]
enum AllHeaderTy {
    QueryDescriptor = 1,
    TransactionDescriptor = 2,
    TraceActivity = 3,
}

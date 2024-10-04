mod batch_request;
mod column_data;
mod decode;
mod encode;
mod guid;
mod header;
mod login;
mod message;
mod pre_login;
mod response;
mod row;
mod rpc_request;
mod token;
pub mod tokenfactory;
mod type_info;

pub use batch_request::*;
pub use column_data::*;
pub use header::*;
pub use login::*;
pub use message::*;
pub use pre_login::*;
pub use response::*;
pub use row::*;
pub use rpc_request::*;
pub use token::*;
pub use tokenfactory::*;
pub use type_info::*;

pub const ALL_HEADERS_LEN_TX: usize = 8;
pub const MAX_PACKET_SIZE: usize = 32767;

#[derive(Debug)]
#[repr(u16)]
#[allow(dead_code)]
enum AllHeaderTy {
    QueryDescriptor = 1,
    TransactionDescriptor = 2,
    TraceActivity = 3,
}

mod token_col_metadata;
mod token_done;
mod token_env_change;
mod token_error;
mod token_feature_ext_ack;
mod token_fed_auth;
mod token_info;
mod token_login_ack;
mod token_order;
mod token_return_status;
mod token_return_value;
mod token_row;
mod token_sspi;
mod token_type;

pub use token_col_metadata::*;
pub use token_done::*;
pub use token_env_change::*;
pub use token_error::*;
pub use token_feature_ext_ack::*;
pub use token_fed_auth::*;
pub use token_info::*;
pub use token_login_ack::*;
pub use token_order::*;
pub use token_return_status::*;
pub use token_return_value::*;
pub use token_row::*;
pub use token_sspi::*;
pub use token_type::*;

use crate::frontend::Result;
use tokio_util::bytes::BytesMut;

#[derive(Debug)]
pub enum TdsToken {
    Done(TokenDone),
    EnvChange(TokenEnvChange),
    Error(TokenError),
    Info(TokenInfo),
    Order(TokenOrder),
    FeatureExtAck(TokenFeatureExtAck),
    ColMetaData(TokenColMetaData),
    FedAuth(TokenFedAuth),
    LoginAck(TokenLoginAck),
    ReturnValue(TokenReturnValue),
    Row(TokenRow),
    Sspi(TokenSspi),
}

pub trait TdsTokenCodec {
    fn encode(&self, dst: &mut BytesMut) -> Result<()>;
    fn decode(src: &mut BytesMut) -> Result<TdsToken>;
}

macro_rules! encode_match {
    ($self:ident, $dst:ident, $( $variant:ident ),* ) => {
        match $self {
            $(
                TdsToken::$variant(token) => token.encode($dst),
            )*
        }
    };
}

impl TdsToken {
    pub fn encode(&self, dst: &mut BytesMut) -> Result<()> {
        encode_match!(
            self,
            dst,
            Done,
            EnvChange,
            Error,
            Info,
            Order,
            FeatureExtAck,
            ColMetaData,
            FedAuth,
            LoginAck,
            ReturnValue,
            Row,
            Sspi
        )
    }
}

macro_rules! impl_into_tdstoken {
    ($t:ty, $tt:expr) => {
        impl Into<TdsToken> for $t {
            fn into(self) -> TdsToken {
                $tt(self)
            }
        }
    };
}

impl_into_tdstoken!(TokenInfo, TdsToken::Info);
impl_into_tdstoken!(TokenDone, TdsToken::Done);
impl_into_tdstoken!(TokenEnvChange, TdsToken::EnvChange);
impl_into_tdstoken!(TokenError, TdsToken::Error);
impl_into_tdstoken!(TokenOrder, TdsToken::Order);
impl_into_tdstoken!(TokenFeatureExtAck, TdsToken::FeatureExtAck);
impl_into_tdstoken!(TokenColMetaData, TdsToken::ColMetaData);
impl_into_tdstoken!(TokenFedAuth, TdsToken::FedAuth);
impl_into_tdstoken!(TokenLoginAck, TdsToken::LoginAck);
impl_into_tdstoken!(TokenReturnValue, TdsToken::ReturnValue);
impl_into_tdstoken!(TokenSspi, TdsToken::Sspi);
impl_into_tdstoken!(TokenRow, TdsToken::Row);

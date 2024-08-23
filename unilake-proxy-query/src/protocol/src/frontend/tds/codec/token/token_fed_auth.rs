use crate::frontend::{
    utils::ReadAndAdvance, Error, Result, TdsToken, TdsTokenCodec, TdsTokenType,
};
use tokio_util::bytes::{Buf, BufMut, BytesMut};

#[derive(PartialEq, Debug)]
pub enum TokenFedAuthOption {
    Spn(String),
    StsUrl(String),
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum TokenPreLoginFedAuthRequiredOption {
    FedAuthNotRequired = 0x00,
    FedAuthRequired = 0x01,
    Illegal = 0x02,
}

/// FedAuthInfo Token [2.2.7.12]
#[derive(Debug)]
pub struct TokenFedAuth {
    pub options: Vec<TokenFedAuthOption>,
}

impl TdsTokenCodec for TokenFedAuth {
    fn decode(src: &mut BytesMut) -> Result<TdsToken> {
        let mut options = Vec::new();
        let _token_length = src.get_u32_le();
        let count_of_ids = src.get_u32_le();
        let mut items: Vec<(u8, u32, u32)> = Vec::with_capacity(count_of_ids as usize);
        let mut current_count = 0;
        while current_count < count_of_ids {
            let ty = src.get_u8();
            let info_data_length = src.get_u32_le();
            let info_data_offset = src.get_u32_le();
            items.push((ty, info_data_length, info_data_offset));
            current_count += 1;
        }

        for (ty, info_data_length, _) in items {
            let (_, buff) = src.read_and_advance(info_data_length as usize);

            let content = String::from_utf8(buff.to_vec())
                .map_err(|_| Error::Protocol("Failed to convert UTF-8 to String".to_string()))
                .unwrap();

            match ty {
                // STS URL as Token Endpoint
                0x01 => {
                    options.push(TokenFedAuthOption::StsUrl(content));
                }
                // Service Principal Name
                0x02 => {
                    options.push(TokenFedAuthOption::Spn(content));
                }
                // Invalid InfoId
                0xEE | _ => {
                    break;
                }
            }
        }

        Ok(TdsToken::FedAuth(TokenFedAuth { options }))
    }

    fn encode(&self, dest: &mut BytesMut) -> Result<()> {
        dest.put_u8(TdsTokenType::FedAuthInfo as u8);
        let options_length = self.options.len() * 9;
        let mut token_length = 4 + options_length;
        let mut buff = Vec::with_capacity(token_length as usize);
        for t in &self.options {
            match t {
                TokenFedAuthOption::Spn(s) | TokenFedAuthOption::StsUrl(s) => {
                    let data = s.as_bytes();
                    buff.extend_from_slice(data);
                    token_length += data.len();
                }
            }
        }

        dest.put_u32_le(token_length as u32);
        dest.put_u32_le(self.options.len() as u32);
        let mut curr_offset = (4 + options_length) as u32;
        for t in &self.options {
            let length;
            match t {
                TokenFedAuthOption::Spn(d) => {
                    dest.put_u8(0x01);
                    length = d.as_bytes().len() as u32;
                }
                TokenFedAuthOption::StsUrl(d) => {
                    dest.put_u8(0x02);
                    length = d.as_bytes().len() as u32;
                }
            }
            dest.put_u32_le(length);
            dest.put_u32_le(curr_offset);
            curr_offset += length;
        }

        dest.extend_from_slice(&buff);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use tokio_util::bytes::{Buf, BytesMut};

    use crate::frontend::{
        Result, TdsToken, TdsTokenCodec, TdsTokenType, TokenFedAuth, TokenFedAuthOption,
    };

    #[test]
    fn encode_decode_token_fed_auth() -> Result<()> {
        let input = TokenFedAuth {
            options: vec![
                TokenFedAuthOption::StsUrl(String::from("https://example.com")),
                TokenFedAuthOption::Spn(String::from("b59a9020-5cd6-4867-a1ed-f3ecb2e3f49f")),
            ],
        };

        // arrange
        let mut buff = BytesMut::new();

        // encode
        input.encode(&mut buff).expect("should be ok");

        // decode
        let tokentype = buff.get_u8();
        let result = TokenFedAuth::decode(&mut buff).unwrap();

        // assert
        assert_eq!(tokentype, TdsTokenType::FedAuthInfo as u8);
        if let TdsToken::FedAuth(result) = result {
            assert_eq!(result.options[0], input.options[0]);
            assert_eq!(result.options[1], input.options[1]);
        }

        Ok(())
    }
}

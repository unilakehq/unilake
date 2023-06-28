use crate::{Error, Result, TokenType};
use std::borrow::Cow;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[derive(PartialEq, Debug)]
pub enum TokenFedAuthOption {
    Spn(String),
    StsUrl(String),
}

/// FedAuthInfo Token [2.2.7.12]
pub struct TokenFedAuth {
    pub options: Vec<TokenFedAuthOption>,
}

impl TokenFedAuth {
    pub async fn decode<R>(src: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let mut options = Vec::new();
        let _token_length = src.read_u32_le().await?;
        let count_of_ids = src.read_u32_le().await?;
        let mut items: Vec<(u8, u32, u32)> = Vec::with_capacity(count_of_ids as usize);
        let mut current_count = 0;
        while current_count < count_of_ids {
            let ty = src.read_u8().await?;
            let info_data_length = src.read_u32_le().await?;
            let info_data_offset = src.read_u32_le().await?;
            items.push((ty, info_data_length, info_data_offset));
            current_count += 1;
        }

        for (ty, info_data_length, _) in items {
            let mut buff = Vec::with_capacity(info_data_length as usize);
            src.take(info_data_length as u64)
                .read_to_end(&mut buff)
                .await?;
            let content = String::from_utf8(buff)
                .map_err(|_| Error::Protocol(Cow::from("Failed to convert UTF-8 to String")))
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

        Ok(TokenFedAuth { options })
    }

    pub async fn encode<W>(&self, dest: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        dest.write_u8(TokenType::FedAuthInfo as u8).await?;
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

        dest.write_u32_le(token_length as u32).await?;
        dest.write_u32_le(self.options.len() as u32).await?;
        let mut curr_offset = (4 + options_length) as u32;
        for t in &self.options {
            let mut length = 0;
            match t {
                TokenFedAuthOption::Spn(d) => {
                    dest.write_u8(0x01).await?;
                    length = d.as_bytes().len() as u32;
                }
                TokenFedAuthOption::StsUrl(d) => {
                    dest.write_u8(0x02).await?;
                    length = d.as_bytes().len() as u32;
                }
            }
            dest.write_u32_le(length).await?;
            dest.write_u32_le(curr_offset).await?;
            curr_offset += length;
        }

        dest.write_all(&buff).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{Result, TokenFedAuth, TokenFedAuthOption, TokenType};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};

    #[tokio::test]
    async fn encode_decode_token_fed_auth() -> Result<()> {
        let mut input = TokenFedAuth {
            options: vec![
                TokenFedAuthOption::StsUrl(String::from("https://example.com")),
                TokenFedAuthOption::Spn(String::from("b59a9020-5cd6-4867-a1ed-f3ecb2e3f49f")),
            ],
        };

        // arrange
        let (inner, outer) = tokio::io::duplex(256);
        let mut writer = BufWriter::new(inner);
        let mut reader = BufReader::new(outer);

        // encode
        input.encode(&mut writer).await?;
        writer.flush().await?;

        // decode
        let token_type = reader.read_u8().await?;
        let result = TokenFedAuth::decode(&mut reader).await?;

        // assert
        assert_eq!(token_type, TokenType::FedAuthInfo as u8);
        assert_eq!(result.options[0], input.options[0]);
        assert_eq!(result.options[1], input.options[1]);

        Ok(())
    }
}

use serde::{ Deserialize, Serialize };

#[derive(Debug, Serialize, Deserialize)]
pub struct JwtPayload {
    pub subject: String,
    pub issued_at: usize,
    pub expiry: usize,
}

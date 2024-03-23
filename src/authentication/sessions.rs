use std::{ str::FromStr, sync::{ Arc, Mutex } };

use rand_chacha::ChaCha8Rng;
use rand_core::RngCore;
use sqlx::{ Pool, Postgres };
use uuid::Uuid;

type Random = Arc<Mutex<ChaCha8Rng>>;
// type Database = sqlx::PgPool;

#[derive(Clone, Copy)]
pub struct SessionToken(u128);

impl FromStr for SessionToken {
    type Err = <u128 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Self)
    }
}

impl SessionToken {
    pub fn generate_new(random: &Random) -> Self {
        let mut u128_pool = [0u8; 16];
        random.lock().unwrap().fill_bytes(&mut u128_pool);
        Self(u128::from_le_bytes(u128_pool))
    }

    pub fn into_cookie_value(self) -> String {
        self.0.to_string()
    }

    pub fn into_database_value(self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

pub async fn new_session(
    database: &Pool<Postgres>,
    random: &Random,
    user_id: Uuid
) -> SessionToken {
    let mut u128_pool = [0u8; 16];

    random.lock().unwrap().fill_bytes(&mut u128_pool);

    let session_token = SessionToken::generate_new(random);

    let _result = sqlx
        ::query("INSERT INTO sessions (session_token, user_id) VALUES ($1, $2);")
        .bind(&session_token.into_database_value())
        .bind(user_id)
        .execute(database).await
        .map_err(|error| {
            eprintln!("Failed to insert session into database: {:?}", error);
        })
        .unwrap();

    return session_token;
}

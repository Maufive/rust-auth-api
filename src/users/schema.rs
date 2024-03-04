use serde::{ Deserialize, Serialize };

// Create
#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserSchema {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

// Update
#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserSchema {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}

// Read/Delete
#[derive(Serialize, Deserialize, Debug)]
pub struct ParamOptions {
    pub id: i32,
}

// List
#[derive(Serialize, Deserialize, Debug)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

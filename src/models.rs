use serde::{Deserialize, Serialize};
use serde_json::Result;


#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub struct Person {
    pub createdAt: String,
    pub name: String,
    pub avatar: String,
    pub id: String,
    pub yep: String,
}

pub fn typed_example(json: &str) -> Result<Vec<Person>> {

    let p: Vec<Person> = serde_json::from_str(json)?;

    Ok(p)
}

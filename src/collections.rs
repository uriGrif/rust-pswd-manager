use std::error::Error;
use std::fs;
use inquire::{ InquireError, Text };
use rand::{ rngs::OsRng, RngCore };
use serde::{ Deserialize, Serialize };
use uuid::Uuid;
use zeroize::Zeroize;

#[derive(Serialize, Deserialize, Debug, Zeroize)]
pub struct Collection {
    id: String,
    pub name: String,
    pub file_path: String,
    pub salt: [u8; 32],
}

impl Collection {
    fn new(name: String, file_path: String) -> Collection {
        let mut salt: [u8; 32] = [0u8; 32];
        OsRng.fill_bytes(&mut salt);

        Collection {
            id: Uuid::new_v4().to_string(),
            name,
            file_path,
            salt,
        }
    }

    pub fn create() -> Result<Collection, InquireError> {
        println!("Creating new collection");
        println!("(Type ESC to go back)");
        let name: String = Text::new("Collection name: ").prompt()?;
        let file_path: String = Text::new("Collection file file_path: ")
            .with_help_message("This is where your passwords will be saved")
            .prompt()?;
        Ok(Collection::new(name, file_path))
    }

    pub fn edit(&mut self) -> Result<(), InquireError> {
        println!("Edit collection");
        println!("(Type ESC to go back)");
        let name: String = Text::new("Collection name: ")
            .with_initial_value(self.name.as_str())
            .prompt()?;
        let file_path: String = Text::new("Collection file file_path: ")
            .with_initial_value(self.file_path.as_str())
            .with_help_message("This is where your passwords will be saved")
            .prompt()?;

        self.name = name;
        self.file_path = file_path;
        Ok(())
    }

    pub fn print_info(&self) {
        println!("Collection: \"{}\" ------- File Path: \"{}\"", self.name, self.file_path);
    }
}

pub fn get_collections(path: &str) -> Result<Vec<Collection>, Box<dyn Error>> {
    let file_stream: Vec<u8> = match fs::read(path) {
        Ok(stream) => stream,
        Err(_) => {
            // file_path does not exist
            _ = fs::write(path, "[]")?;
            fs::read(path).unwrap()
        }
    };

    // println!("{}", String::from_utf8(file_stream.clone()).unwrap());

    match serde_json::from_slice(&file_stream) {
        Ok(v) => Ok(v),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn save_collections(collections: &Vec<Collection>, path: &str) -> Result<(), Box<dyn Error>> {
    let serialized: String = serde_json::to_string(&collections).unwrap();

    fs::write(path, serialized)?;
    Ok(())
}

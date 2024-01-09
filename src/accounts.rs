use inquire::{ InquireError, Password, Text };
use serde::{ Deserialize, Serialize };
use std::error::Error;
use std::fs;
use uuid::Uuid;
use zeroize::Zeroize;
use cli_clipboard;

use crate::cryptography;
use crate::utils::clear_terminal_screen;

#[derive(Serialize, Deserialize, Debug, Zeroize)]
pub struct Account {
    id: String,
    pub name: String,
    email: String,
    username: String,
    hints: String,
    password: String,
}

impl Account {
    fn new(
        name: String,
        email: String,
        username: String,
        hints: String,
        password: String
    ) -> Account {
        Account {
            id: Uuid::new_v4().to_string(),
            name,
            email,
            username,
            hints,
            password,
        }
    }

    pub fn create() -> Result<Account, InquireError> {
        println!("Creating new account");
        println!("(Type ESC to go back)");
        let name: String = Text::new("account name: ").prompt()?;
        let email: String = Text::new("account email: ").prompt()?;
        let username: String = Text::new("account username: ").prompt()?;
        let hints: String = Text::new("password hints: ").prompt()?;
        let password: String = Text::new("account password: ").prompt()?;
        Ok(Account::new(name, email, username, hints, password))
    }

    pub fn edit(&mut self) -> Result<(), InquireError> {
        clear_terminal_screen();
        println!("Edit account");
        println!("(Type ESC to go back)");

        let name: String = Text::new("account name: ").with_initial_value(&self.name).prompt()?;
        let email: String = Text::new("account email: ").with_initial_value(&self.email).prompt()?;
        let username: String = Text::new("account username: ")
            .with_initial_value(&self.username)
            .prompt()?;
        let hints: String = Text::new("password hints: ").with_initial_value(&self.hints).prompt()?;
        let password: String = Text::new("account password: ")
            .with_initial_value(&self.password)
            .prompt()?;

        self.name = name;
        self.email = email;
        self.username = username;
        self.hints = hints;
        self.password = password;

        Ok(())
    }

    pub fn print_info(&self, show_pswd: bool) {
        println!("Account");
        println!("name: \"{}\"", self.name);
        println!("email: \"{}\"", self.email);
        println!("username: \"{}\"", self.username);
        println!("hints: \"{}\"", self.hints);
        if show_pswd {
            println!("password: \"{}\"\n", self.password);
        } else {
            println!(
                "password: \"{}\"\n",
                String::from_iter(std::iter::repeat("*").take(self.password.len()))
            );
        }
    }

    pub fn copy_to_clipboard(&self) {
        cli_clipboard::set_contents(self.password.to_owned()).unwrap();
    }
}

pub fn get_accounts(
    file_path: &String,
    password: &mut String,
    salt: &[u8; 32]
) -> Result<Vec<Account>, Box<dyn Error>> {
    let file_stream: Vec<u8> = match fs::read(file_path) {
        Ok(stream) => stream,
        Err(_) => {
            // file_path does not exist
            // create file with empty encrypted list
            *password = Password::new(
                "This seems to be a new collection. Create a master password: "
            )
                .with_help_message(
                    "This password will then be used to encrypt and decrypt this collection's accounts file. Keep it somewhere safe and don't lose it"
                )
                .with_display_mode(inquire::PasswordDisplayMode::Masked)
                .prompt()?;
            save_accounts(&vec![], file_path, &password, &salt)?;
            return Ok(vec![]);
        }
    };

    let acc: String;

    loop {
        let temp_pswd: String = Password::new("Enter master password: ")
            .without_confirmation()
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()?;
        match
            cryptography::decrypt(
                &std::str::from_utf8(&file_stream).unwrap(),
                temp_pswd.as_bytes(),
                salt
            )
        {
            Ok(decrypted) => {
                acc = decrypted;
                *password = temp_pswd;
                break;
            }
            Err(e) => {
                match e.downcast_ref::<std::io::Error>() {
                    Some(er) => {
                        if er.kind() == std::io::ErrorKind::PermissionDenied {
                            clear_terminal_screen();
                            println!("Incorrect password! Try again");
                        }
                    }
                    None => {
                        return Err(e);
                    }
                }
            }
        };
    }

    match serde_json::from_slice(&acc.as_bytes()) {
        Ok(v) => Ok(v),
        Err(e) => Err(Box::new(e)),
    }
}

pub fn save_accounts(
    accounts: &Vec<Account>,
    file_path: &String,
    password: &String,
    salt: &[u8; 32]
) -> Result<(), Box<dyn Error>> {
    let serialized: String = serde_json::to_string(accounts).unwrap();

    let encrypted: String = cryptography
        ::encrypt(serialized.as_bytes(), password.as_bytes(), salt)
        .unwrap();

    _ = fs::write(file_path, encrypted)?;
    Ok(())
}

use crate::accounts::*;
use crate::collections::*;
use crate::utils::*;
use inquire::{ Confirm, InquireError, Select };
use std::error::Error;
use std::usize;
use std::{ fmt, fs };
use zeroize::Zeroize;
use Action::*;

const COLLECTIONS_FILE_PATH: &str = "./collections.json";

enum Action {
    CollectionSelection(usize, String),
    AccountSelection(usize, String),
    LoadAccounts,
    NewCollection,
    NewAccount,
    EditCollection,
    EditAccount,
    DeleteCollection,
    DeleteAccount,
    GoBackToCollections,
    GoBackToAccounts,
    TogglePasswordView,
    CopyToClipboard,
    Exit,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            CollectionSelection(_, c) => write!(f, "{}", c),
            AccountSelection(_, a) => write!(f, "{}", a),
            LoadAccounts => write!(f, "Load accounts"),
            NewCollection => write!(f, "Add new collection"),
            NewAccount => write!(f, "Add new account"),
            EditCollection => write!(f, "Edit collection"),
            EditAccount => write!(f, "Edit account"),
            DeleteCollection => write!(f, "Delete collection"),
            DeleteAccount => write!(f, "Delete account"),
            GoBackToCollections => write!(f, "Go Back"),
            GoBackToAccounts => write!(f, "Go Back"),
            TogglePasswordView => write!(f, "Show/hide password"),
            CopyToClipboard => write!(f, "Copy password to clipboard"),
            Exit => write!(f, "Exit"),
        }
    }
}

pub struct Manager {
    collections: Vec<Collection>,
    accounts: Option<Vec<Account>>,
    selected_coll_index: Option<usize>,
    selected_acc_index: Option<usize>,
    password: String,
    salt: [u8; 32],
    update_collections: bool,
    update_accounts: bool,
    show_password: bool,
}

impl Manager {
    pub fn new() -> Self {
        Self {
            collections: vec![],
            accounts: None,
            selected_coll_index: None,
            selected_acc_index: None,
            password: String::new(),
            salt: [0u8; 32],
            update_collections: false,
            update_accounts: false,
            show_password: false,
        }
    }

    pub fn display_menu(&mut self) -> Result<(), Box<dyn Error>> {
        self.load_collections()?;

        loop {
            let mut options: Vec<Action> = vec![];

            clear_terminal_screen();

            if self.selected_coll_index.is_some() {
                self.print_collection_info();
                if self.selected_acc_index.is_some() {
                    self.print_account_info();
                    options.push(TogglePasswordView);
                    options.push(CopyToClipboard);
                    options.push(EditAccount);
                    options.push(DeleteAccount);
                    options.push(GoBackToAccounts);
                } else {
                    if self.accounts.is_none() {
                        options.push(LoadAccounts);
                    } else {
                        self.accounts
                            .as_ref()
                            .unwrap()
                            .iter()
                            .enumerate()
                            .for_each(|(i, a)| options.push(AccountSelection(i, a.name.clone())));
                        options.push(NewAccount);
                    }
                    options.push(EditCollection);
                    options.push(DeleteCollection);
                    options.push(GoBackToCollections);
                }
            } else {
                self.collections
                    .iter()
                    .enumerate()
                    .for_each(|(i, c)| options.push(CollectionSelection(i, c.name.clone())));
                options.push(NewCollection);
            }

            options.push(Exit);

            let action: Action = match Select::new("Choose an option:", options).prompt() {
                Ok(act) => act,
                Err(InquireError::OperationCanceled) => {
                    if self.selected_coll_index.is_none() {
                        Exit
                    } else if self.selected_acc_index.is_some() {
                        GoBackToAccounts
                    } else {
                        GoBackToCollections
                    }
                }
                Err(e) => {
                    return Err(Box::new(e));
                }
            };

            _ = (match action {
                CollectionSelection(index, _) => self.select_collection(index),
                AccountSelection(index, _) => self.select_account(index),
                LoadAccounts => self.load_accounts(),
                NewCollection => self.add_collection(),
                NewAccount => self.add_account(),
                EditCollection => self.edit_collection(),
                EditAccount => self.edit_account(),
                DeleteCollection => self.delete_collection(),
                DeleteAccount => self.delete_account(),
                GoBackToCollections => self.unselect_collection(),
                GoBackToAccounts => self.unselect_account(),
                TogglePasswordView => {
                    self.toggle_password_view();
                    Ok(())
                }
                CopyToClipboard => {
                    self.copy_to_clipboard();
                    Ok(())
                }
                Exit => {
                    self.save_and_exit()?;
                    break;
                }
            })?;
        }

        Ok(())
    }

    pub fn save_and_exit(&mut self) -> Result<(), Box<dyn Error>> {
        if self.update_collections {
            save_collections(&self.collections, COLLECTIONS_FILE_PATH)?;
        }

        if self.update_accounts && self.accounts.is_some() {
            save_accounts(
                self.accounts.as_ref().unwrap(),
                &self.collections.get(self.selected_coll_index.unwrap()).unwrap().file_path,
                &self.password,
                &self.salt
            )?;
        }

        self.collections.zeroize();
        self.accounts.zeroize();
        self.password.zeroize();
        self.salt.zeroize();
        clear_terminal_screen();
        println!("Information succesfully saved!");
        Ok(())
    }

    // collections actions
    fn load_collections(&mut self) -> Result<(), Box<dyn Error>> {
        self.collections = get_collections(COLLECTIONS_FILE_PATH)?;
        Ok(())
    }

    fn print_collection_info(&self) {
        self.collections.get(self.selected_coll_index.unwrap()).unwrap().print_info();
    }

    fn select_collection(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        self.selected_coll_index = Some(index);
        self.salt = self.collections[self.selected_coll_index.unwrap()].salt;
        Ok(())
    }

    fn unselect_collection(&mut self) -> Result<(), Box<dyn Error>> {
        if self.update_accounts && self.accounts.is_some() {
            save_accounts(
                self.accounts.as_ref().unwrap(),
                &self.collections.get(self.selected_coll_index.unwrap()).unwrap().file_path,
                &self.password,
                &self.salt
            )?;
        }
        self.accounts.zeroize();
        self.accounts = None;
        self.selected_acc_index = None;
        self.selected_coll_index = None;
        self.salt.zeroize();
        self.password.zeroize();
        self.show_password = false;
        Ok(())
    }

    fn add_collection(&mut self) -> Result<(), Box<dyn Error>> {
        match Collection::create() {
            Ok(c) => {
                self.update_collections = true;
                self.collections.push(c);
            }
            Err(InquireError::OperationCanceled) => {}
            Err(e) => {
                return Err(Box::new(e));
            }
        }
        Ok(())
    }

    fn edit_collection(&mut self) -> Result<(), Box<dyn Error>> {
        match self.collections.get_mut(self.selected_coll_index.unwrap()).unwrap().edit() {
            Ok(_) => {
                self.update_collections = true;
                Ok(())
            }
            Err(InquireError::OperationCanceled) => Ok(()),
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    fn delete_collection(&mut self) -> Result<(), Box<dyn Error>> {
        let ans: bool = match
            Confirm::new("Are you sure you want to delete this collection?")
                .with_default(false)
                .with_help_message(
                    "This collection and the file that contains its passwords will be permanently deleted"
                )
                .prompt()
        {
            Ok(ans) => ans,
            Err(InquireError::OperationCanceled) => false,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        if ans {
            match
                fs::remove_file(
                    &self.collections.get(self.selected_coll_index.unwrap()).unwrap().file_path
                )
            {
                Err(_) => {
                    println!("Passwords file not found!");
                    // Err(InquireError::Custom(Box::new(e)))
                }
                _ => {}
            }
            self.collections.remove(self.selected_coll_index.unwrap());
            self.unselect_collection()?;
            self.update_collections = true;
        }
        Ok(())
    }

    // accounts actions
    fn load_accounts(&mut self) -> Result<(), Box<dyn Error>> {
        match
            get_accounts(
                &self.collections[self.selected_coll_index.unwrap()].file_path,
                &mut self.password,
                &self.salt
            )
        {
            Ok(a) => {
                self.accounts = Some(a);
            }
            Err(e) => {
                match e.downcast_ref::<InquireError>() {
                    Some(er) => {
                        match *er {
                            InquireError::OperationCanceled => {
                                clear_terminal_screen();
                                println!("Incorrect password! Try again");
                            }
                            _ => {
                                return Err(e);
                            }
                        }
                    }
                    None => {
                        return Err(e);
                    }
                }
            }
        }
        Ok(())
    }

    fn print_account_info(&self) {
        self.accounts
            .as_ref()
            .unwrap()
            .get(self.selected_acc_index.unwrap())
            .unwrap()
            .print_info(self.show_password);
    }

    fn toggle_password_view(&mut self) {
        self.show_password = !self.show_password;
    }

    fn select_account(&mut self, index: usize) -> Result<(), Box<dyn Error>> {
        self.selected_acc_index = Some(index);
        Ok(())
    }

    fn unselect_account(&mut self) -> Result<(), Box<dyn Error>> {
        if self.update_accounts && self.accounts.is_some() {
            save_accounts(
                self.accounts.as_ref().unwrap(),
                &self.collections.get(self.selected_coll_index.unwrap()).unwrap().file_path,
                &self.password,
                &self.salt
            )?;
        }
        self.selected_acc_index = None;
        self.show_password = false;
        Ok(())
    }

    fn add_account(&mut self) -> Result<(), Box<dyn Error>> {
        match Account::create() {
            Ok(c) => {
                self.update_accounts = true;
                self.accounts.as_mut().unwrap().push(c);
            }
            Err(InquireError::OperationCanceled) => {}
            Err(e) => {
                return Err(Box::new(e));
            }
        }
        Ok(())
    }

    fn edit_account(&mut self) -> Result<(), Box<dyn Error>> {
        match
            self.accounts
                .as_mut()
                .unwrap()
                .get_mut(self.selected_acc_index.unwrap())
                .unwrap()
                .edit()
        {
            Ok(_) => {
                self.update_accounts = true;
                Ok(())
            }
            Err(InquireError::OperationCanceled) => Ok(()),
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    fn delete_account(&mut self) -> Result<(), Box<dyn Error>> {
        let ans: bool = match
            Confirm::new("Are you sure you want to delete this account?")
                .with_default(false)
                .with_help_message("This account will be permanently deleted")
                .prompt()
        {
            Ok(ans) => ans,
            Err(InquireError::OperationCanceled) => false,
            Err(e) => {
                return Err(Box::new(e));
            }
        };

        if ans {
            self.accounts.as_mut().unwrap().remove(self.selected_acc_index.unwrap());
            self.update_accounts = true;
            self.unselect_account()?;
        }
        Ok(())
    }

    fn copy_to_clipboard(&self) {
        self.accounts
            .as_ref()
            .unwrap()
            .get(self.selected_acc_index.unwrap())
            .unwrap()
            .copy_to_clipboard()
    }
}

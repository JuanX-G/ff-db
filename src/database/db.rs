use std::fs::{read_dir};
use std::path::Path;
use std::env;
use crate::database::{table::Table, errors::DBError};

/* Datebase is the struct holding tables. */

#[derive(Debug)]
pub struct DB {
    tables: Vec<Table>,
}

impl DB {
    pub fn open(dir_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(dir_name);
        let dir_itr = read_dir(path)?;
        env::set_current_dir(path)?;
        let mut db: DB = DB {tables: vec![]};
        for entry in dir_itr {
            if let Ok(entry) = entry {
                let tab = Table::new(&match entry.file_name().into_string() {
                    Ok(s) => s,
                    Err(_) => return Err(Box::new(DBError::GenericLoadingError)),
                })?;
                db.tables.push(tab);
            }
        };
        Ok(db)        
    }
    pub fn get_mut_table(&mut self, table_name: &str) -> Option<&mut Table> {
            for tb in self.tables.iter_mut() {
                if tb.name == table_name {return Some(tb)}
            }
            None
    }
}



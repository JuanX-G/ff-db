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
            let entry = entry?;
            if entry.metadata()?.is_dir() {continue;}
            let string = match entry.file_name().into_string() {
                Ok(s) => s.to_string(),
                Err(_) => continue,
            };
            db.tables.push(Table::new(&string)?);
        };
        Ok(db)        
    }
    pub fn get_mut_table(&mut self, table_name: &str) -> Option<&mut Table> {
            for tb in self.tables.iter_mut() {
                if tb.name == table_name {return Some(tb)}
            }
            None
    }
    pub fn get_table_count(&self) -> usize {
        self.tables.len()
    }
}



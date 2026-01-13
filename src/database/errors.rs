use std::error::Error;
use std::fmt;

use crate::database::database::{DBField, DataTypes};

#[derive(Debug)]
pub enum DBError {
    ColumnNotFound(Vec<String>),
    FileError(Box<dyn std::error::Error>),
    GenericLoadingError,
    MalformedInsertInput,
    MistypedInsertInput(DBField, DataTypes),
    InvalidComparasion,
}
impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            DBError::ColumnNotFound(s) => {
                let mut out_s: String = "Columns '".to_string();
                for to_push in s {
                    out_s.push_str(to_push);
                    out_s.push_str(", ");
                }
                out_s.push_str("' not found"); 
                out_s
            },
            DBError::FileError(e) => {
                let mut out_s = "error occured with the db file. Reported error: ".to_string();
                out_s.push_str(&*e.to_string());
                out_s
            },
            DBError::MalformedInsertInput => "malformed insert input, missing field specified to be inserted".to_string(),
            DBError::MistypedInsertInput(f, exp_type) => {
                let mut out_s = "wrong input type. Got ".to_string();
                    out_s.push_str(match f {
                        DBField::Text(_) => "TEXT",
                        DBField::Int(_) => "INT",
                    });
                    out_s.push_str(" expected ");
                    out_s.push_str(&exp_type.to_file_string());
                    out_s
            },
        DBError::InvalidComparasion => "Invalid comparsion was made".to_string(),
        DBError::GenericLoadingError => "Error loading the db".to_string(),
        })
    }
}
impl Error for DBError {}

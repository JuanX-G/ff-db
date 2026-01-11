use std::fs::{File};
use std::path::Path;
use std::io::{Read, Seek, SeekFrom, Write};
use std::error::Error;
use std::fmt;
pub mod sql;
use sql::lexer::Lexer;

use crate::sql::engine::Engine;
use crate::sql::{engine, parser};

#[derive(Debug)]
enum DBError {
    ColumnNotFound(Vec<String>),
    FileError,
    MalformedInsertInput,
    MistypedInsertInput(DBField, DataTypes),
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
            DBError::FileError => "error occured with the db file".to_string(),
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
            }
        })
    }
}
impl Error for DBError {}

#[derive(Debug, Clone)]
pub enum DBField {
    Text(String),
    Int(i32),
}

impl DBField {
    fn to_file_string(&self) -> String {
        match self {
            DBField::Text(s) => s.clone(),
            DBField::Int(i) => i.to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct DBColumn {
    dt_type: DataTypes,
    name: String,
}
impl DBColumn {
    fn to_file_string(&self) -> String {
        let mut out_str = "".to_string();
        out_str.push_str(&self.name);
        out_str.push_str(": ");
        out_str.push_str(&self.dt_type.to_file_string());
        out_str
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataTypes {
    TEXT,
    INT,
}
impl DataTypes {
    fn to_file_string(&self) -> String {
        match self {
            DataTypes::TEXT => "TEXT".to_string(),
            DataTypes::INT => "INT".to_string(),
        }
    }
}

#[derive(Debug)]
pub struct DB {
    file: File,
    header: Vec<DBColumn>,
    entries: Vec<Vec<DBField>>,
}


impl DB {
    fn load_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.file.seek(SeekFrom::Start(0))?;
        let mut contents = String::new();
        let _ = self.file.read_to_string(&mut contents);
        let lines_itr = contents.split('\n');
        for (ref mut idx, line) in lines_itr.enumerate() {
            if *idx == 0 {
                let line_split_itr = line.split(',');
                for elem in line_split_itr {
                    let (name, dt_type) = match elem.split_once(":") {
                        Some(s) => s,
                        _ => panic!("no type for column"),
                    };
                    let dt_type = match dt_type.trim() {
                        "TEXT" => DataTypes::TEXT,
                        "INT" => DataTypes::INT,
                        _ => panic!("invalid data type"),
                    };
                    let to_push: DBColumn = DBColumn {dt_type: dt_type, name: name.trim().to_string()};
                    self.header.push(to_push);
                }
                *idx += 1;
                continue;
            }
            
            let line_split = line.split(',');
            // if line_split.clone().count() != self.header.len() {continue}
            let mut line_vec: Vec<DBField> = vec![];
            for (ref mut idxb, elem) in line_split.enumerate() {
                if self.header[*idxb].dt_type == DataTypes::TEXT {
                    let field_to_add = DBField::Text(elem.trim().to_string());
                    line_vec.push(field_to_add);
                } else if self.header[*idxb].dt_type == DataTypes::INT {
                    let field_to_add = DBField::Int(match elem.parse::<i32>() {
                        Ok(i) => i,
                        Err(e) => return Err(Box::new(e)),
                    });
                    line_vec.push(field_to_add);
                } 
                *idxb += 1;
            }
            self.entries.push(line_vec);
            *idx += 1;
        }
        Ok(())
    }
    pub fn new(file_name: &str) -> Self {
        let path = Path::new(file_name);
        let f = match File::options().append(true).read(true).open(path) {
            Ok(f) => f,
            Err(e) => panic!("{}", e)
        };  
        DB {file: f, header: vec![], entries: vec![vec![]]}
    }
    pub fn get_where_col(&self, col_name: &str, key: DBField) -> Result<Vec<Vec<DBField>>, ()> {
        let mut col_idx = 0;
        let mut found = false;
        for (idx, col) in self.header.clone().into_iter().enumerate() {
            if col_name == col.name {
                col_idx = idx;
                found = true;
            }
        }
        if !found {
            return Err(())
        }
        let mut k_to_search_i = 0; 
        let mut k_to_search_s = "".to_string(); 
        match key {
            DBField::Text(t) => {k_to_search_s = t;},
            DBField::Int(i) => {k_to_search_i = i;},
        }
        let mut out_vec = vec![vec![]];
        for row in &self.entries {
            if row.len() < col_idx + 1 {continue;}
            match &row[col_idx] {
                DBField::Text(t) => if *t == k_to_search_s {out_vec.push(row.to_vec())}
                DBField::Int(i) => if *i == k_to_search_i {out_vec.push(row.to_vec())}
            }
        }
        Ok(out_vec)
    }
    pub fn insert(&mut self, col_names: Option<Vec<String>>, mut row: Vec<DBField>) -> Result<(), Box<dyn std::error::Error>> {
        let col_names = match col_names {
            Some(c) => c,
            _ => self.header.iter().map(|c| c.name.clone()).collect(),
        };
        for (idx, col_nm) in col_names.clone().into_iter().enumerate() {
            let mut found = false;
            let mut head_idx_to_use = 0;
            for (head_idx, col) in self.header.clone().into_iter().enumerate() {
                if col_nm == col.name {
                    found = true; 
                    head_idx_to_use = head_idx;
                }
            }
            if !found {return Err(Box::new(DBError::ColumnNotFound(col_names)))};

            let field = match row.get(idx) {
                Some(f) => f,
                _ => return Err(Box::new(DBError::MalformedInsertInput)),
            };
            let head_col = &self.header[head_idx_to_use];
            match field {
                DBField::Text(_) => {if head_col.dt_type == DataTypes::TEXT {continue;}}
                DBField::Int(_) => {if head_col.dt_type == DataTypes::INT {continue;}}
            }
            return Err(Box::new(DBError::MistypedInsertInput(field.clone(), head_col.dt_type.clone())))
        }
        for (head_idx, col) in self.header.clone().into_iter().enumerate() {
            let mut covered = false;
            for col_nm in &col_names {
                if *col_nm == col.name {
                    covered = true;
                }
            }
            if covered {continue}
            let field_to_add = match col.dt_type {
                DataTypes::TEXT => DBField::Text("".to_string()),
                DataTypes::INT => DBField::Int(0), //TODO: get rid of the zero, allow no value
            };
            row.insert(head_idx, field_to_add);
        }

        self.entries.push(row);
        self.write_to_file()?;
        Ok(())
    }
    pub fn select_cols(&self, cols: Vec<String>) -> Result<Vec<Vec<DBField>>, ()> {
        let mut col_idx = vec![];
        let mut found = false;
        for (idx, col) in self.header.clone().into_iter().enumerate() {
            for col_requested in &cols {
                if *col_requested == col.name {
                    col_idx.push(idx);
                    found = true;
                }
            }
        }
        if !found {
            return Err(())
        }
        let mut max_i = 0;
        for i in &col_idx {
            if *i > max_i {max_i = *i}
        }
        let mut out_vec = vec![vec![]];
        for row in &self.entries {
            if row.len() < max_i + 1 {continue;}
            let mut row_vec = vec![];
            for idx in &col_idx {
                row_vec.push(row[*idx].clone());
            }
            out_vec.push(row_vec);
        }
        Ok(out_vec)
    }
    fn write_to_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut out_str = "".to_string();
        let e_len = self.header.len();
        for (idx, elem) in self.header.clone().into_iter().enumerate() {
            out_str.push_str(&elem.to_file_string());
            if idx != e_len - 1 {
                out_str.push_str(", ");
            }
        }
        for entry in &self.entries {
            dbg!(entry);
            let e_len = entry.len();
            for (idx, elem) in entry.into_iter().enumerate() {
                out_str.push_str(&elem.to_file_string());
                if idx != e_len - 1 {
                    out_str.push_str(", ");
                }
            }
            out_str.push_str("\n");
        };
        out_str.push_str("\n");
        self.file.set_len(0)?;
        self.file.write(out_str.as_bytes())?;
        Ok(())
    }
}



fn main() {
    println!("Hello, world!");
    let mut db = DB::new("db.txt");
    let _ = db.load_db();
    let _ = db.get_where_col("name", DBField::Text("hello".to_string()));
    let sql_s = "INSERT INTO users (num) VALUES (6) ";
    let mut lx = sql::lexer::Lexer {
        input: sql_s.chars().peekable(),
    };
    let tokens = match lx.lex() {
        Ok(tkns) => tkns,
        Err(e) => panic!("{}", e),
    };
    let mut parser = sql::parser::Parser::new(tokens);
    let ast_root = parser.generate_ast().unwrap();
    let e = Engine{ast_root: ast_root};
    let output = e.run(&mut db).unwrap();
    dbg!(output);
}

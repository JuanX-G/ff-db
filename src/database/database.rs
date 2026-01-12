use itertools::Itertools;
use std::fs::{File};
use std::path::Path;
use std::io::{Read, Seek, SeekFrom, Write};

use crate::sql::ast::Literal;
use crate::sql::engine::{WhereClause, WhereOperator};
use crate::database::errors::DBError;

#[derive(Debug, Clone)]
pub enum DBField {
    Text(String),
    Int(i32),
}

impl DBField {
    pub fn to_file_string(&self) -> String {
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
    pub fn to_file_string(&self) -> String {
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
    pub fn load_db(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
            if line_split.clone().count() <= 1 {continue;}
            let mut line_vec: Vec<DBField> = vec![];
            for (ref mut idxb, elem) in line_split.enumerate() {
                if self.header[*idxb].dt_type == DataTypes::TEXT {
                    let field_to_add = DBField::Text(elem.trim().to_string());
                    line_vec.push(field_to_add);
                } else if self.header[*idxb].dt_type == DataTypes::INT {
                    let field_to_add = DBField::Int(match elem.parse::<i32>() {
                        Ok(i) => i,
                        Err(e) => {return Err(Box::new(e))},
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
    pub fn new(file_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let path = Path::new(file_name);
        let f = match File::options().append(true).read(true).open(path) {
            Ok(f) => f,
            Err(e) => panic!("{}", e)
        };  
        let mut ret_db = DB{file: f, header: vec![], entries: vec![vec![]]};
        match ret_db.load_db() {
            Ok(_) => (),
            Err(e) => return Err(Box::new(DBError::FileError(e))),
        }
        Ok(ret_db)
    }
    pub fn insert(&mut self, col_names: Option<Vec<&str>>, mut row: Vec<DBField>) -> Result<(), Box<dyn std::error::Error>> {
        let col_names = match col_names {
            Some(c) => c,
            _ => self.header.iter().map(|c| c.name.as_str()).collect(),
        };
        for (idx, col_nm) in col_names.iter().enumerate() {
            let mut found = false;
            let mut head_idx_to_use = 0;
            for (head_idx, col) in self.header.iter().enumerate() {
                if *col_nm == col.name {
                    found = true; 
                    head_idx_to_use = head_idx;
                }
            }
            if !found {return Err(Box::new(DBError::ColumnNotFound(col_names.iter().map(|cn| cn.to_string()).collect())))};

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
        for (head_idx, col) in self.header.iter().enumerate() {
            let mut covered = false;
            for col_nm in &col_names {
                if *col_nm == col.name {
                    covered = true;
                }
            }
            if covered {continue}
            let field_to_add = match col.dt_type {
                DataTypes::TEXT => DBField::Text("".to_string()),
                DataTypes::INT => DBField::Int(0), 
            };
            row.insert(head_idx, field_to_add);
        }

        self.entries.push(row);
        self.write_to_file()?;
        Ok(())
    }
    pub fn select_cols(&self, cols: Vec<&str>) -> Result<Vec<Vec<DBField>>, ()> {
        let mut col_idx = vec![];
        let mut found = false;
        for (idx, col) in self.header.iter().enumerate() {
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
    pub fn select_where(&self, cols: Vec<String>, where_clauses: Vec<WhereClause>) -> 
        Result<Vec<Vec<DBField>>, Box<dyn std::error::Error>> {
            let mut col_idx = vec![];
            let mut found = false;
            for (idx, col) in self.header.iter().enumerate() {
                for col_requested in &cols {
                    if *col_requested == col.name {
                        col_idx.push(idx);
                        found = true;
                    }
                }
            }
            if !found {
                return Err(Box::new(DBError::ColumnNotFound(cols)))
            }
            let mut max_i = 0;
            for i in &col_idx {
                if *i > max_i {max_i = *i}
            }
            let mut out_vec = vec![vec![]];
            for row in &self.entries {
                if max_i > row.len() {
                    continue;
                }
                let mut row_vec = vec![];
                let mut wc_satisfied = true;
                for wc in &where_clauses {
                    let col_idx = self.header.iter().position(|c| c.name == wc.col_name)
                        .ok_or(DBError::ColumnNotFound(vec![wc.col_name.clone()]))?;
                    let field = &row[col_idx];

                    let clause_ok = match (&wc.operator, &wc.expected_value, field) {
                        (WhereOperator::Equal, Literal::String(s), DBField::Text(t)) => t == s,
                        (WhereOperator::Equal, Literal::Number(n), DBField::Int(i)) => i == n,
                        (WhereOperator::NotEqual, Literal::String(s), DBField::Text(t)) => t != s,
                        (WhereOperator::NotEqual, Literal::Number(n), DBField::Int(i)) => i != n,
                        (WhereOperator::Greater, Literal::Number(n), DBField::Int(i)) => i > n,
                        (WhereOperator::Smaller, Literal::Number(n), DBField::Int(i)) => i < n,
                        _ => return Err(Box::new(DBError::InvalidComparasion)),
                    };
                    if !clause_ok {
                        wc_satisfied = false;
                        break;
                    }
                }

                if wc_satisfied {
                    for idx in &col_idx {
                        row_vec.push(row[*idx].clone());
                    }
                    out_vec.push(row_vec);
                }

            }
            Ok(out_vec)

        }
    fn write_to_file(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut out_str = "".to_string();
        out_str.push_str(
            &self.header
            .iter()
            .map(|c| c.to_file_string())
            .intersperse(", ".to_string())
            .collect::<String>()
        );
        for entry in &self.entries {
            out_str.push_str(
                &entry
                .iter()
                .map(|f| f.to_file_string())
                .intersperse(", ".to_string())
                .collect::<String>()
            );
            out_str.push_str("\n");
        }
        out_str.push_str("\n");
        match self.file.set_len(0) {
            Ok(_) => (),
            Err(e) => return Err(Box::new(DBError::FileError(Box::new(e)))),
        }
        match self.file.write(out_str.as_bytes()) {
            Ok(_) => (),
            Err(e) => return Err(Box::new(DBError::FileError(Box::new(e)))),
        }
        match self.file.flush() {
            Ok(_) => (),
            Err(e) => return Err(Box::new(DBError::FileError(Box::new(e)))),
        }
        Ok(())
    }
}

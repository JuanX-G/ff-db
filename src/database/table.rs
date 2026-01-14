use std::fs::File;
use std::path::Path;
use std::io::{Seek, SeekFrom, Read, Write};
use itertools::Itertools;
use crate::database::{DBColumn, DBField, DataTypes};
use crate::ast::Expr;
use crate::engine::Engine;
use crate::database::errors::DBError;

#[derive(Debug)]
pub struct Table {
    pub name: String,
    file: File,
    header: Vec<DBColumn>,
    entries: Vec<Vec<DBField>>,
}

impl Table {
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
    pub fn select_cols(&self, cols: Vec<&str>) -> Result<Vec<Vec<DBField>>, Box<dyn std::error::Error>> {
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
            return Err(Box::new(DBError::ColumnNotFound(cols.iter().map(|col| col.to_string()).collect())))
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
    pub fn select_where(
        &self,
        cols: Vec<String>,
        where_exprs: &[Expr],
        engine: &Engine,
    ) -> Result<Vec<Vec<DBField>>, Box<dyn std::error::Error>> {
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
            return Err(Box::new(DBError::ColumnNotFound(cols.iter().map(|col| col.to_string()).collect())))
        }
        let mut max_i = 0;
        for i in &col_idx {
            if *i > max_i {max_i = *i}
        }
        let mut out_vec = vec![vec![]];
        for row in &self.entries {
            if row.len() < max_i + 1 {continue;}
            let mut satisfied = true;

            for expr in where_exprs {
                if !match engine.eval_expr(expr, row, &self.header) {
                    Ok(b) => b,
                    Err(_) => continue,
                }{
                    satisfied = false;
                    break;
                }
            }

            if satisfied {
                let mut out_row = Vec::new();
                for idx in &col_idx {
                    out_row.push(row[*idx].clone());
                }
                out_vec.push(out_row);
            }
        }
        Ok(out_vec)
    }

}


/* 
 * ### ### ###
 * 
 *
 * ### SEPARATOR FOR READABILITY
 *
 *
 * ### ### ###
 */


impl Table {
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
    pub fn load_table(&mut self) -> Result<(), Box<dyn std::error::Error>> {
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
        let mut ret_db = Table{name: file_name.to_string(), file: f, header: vec![], entries: vec![vec![]]};
        match ret_db.load_table() {
            Ok(_) => (),
            Err(e) => return Err(Box::new(DBError::FileError(e))),
        }
        Ok(ret_db)
    }
}

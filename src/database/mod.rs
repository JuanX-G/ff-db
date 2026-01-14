pub mod db;
pub mod errors;
pub mod table;


/*  ## Database ##
 *  Module holding all the constructs relating to the 'physical' database
 */

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
    pub name: String,
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

/* Data primitives for the tables
 * Text -- any length text
 * Int -- i32 int implemented via rusts default i32
 */
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

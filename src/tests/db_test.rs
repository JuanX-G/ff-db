use std::fs::{OpenOptions, create_dir, exists};
use std::path::Path;
use std::io::Write;
use crate::database::{DBField, db};
use super::constants::*;

fn setup_mock_db() {
    let db_dir_path = Path::new(TEST_DB_PATH);
    if !exists(db_dir_path).unwrap() {
        create_dir(db_dir_path).unwrap();
    }
    let table_file_path = Path::new(TEST_TABLE_PATH);
    let mut table_f = OpenOptions::new().create(true).write(true).truncate(true).open(table_file_path).unwrap(); 
    table_f.write(TEST_TABLE_CONTENTS.as_bytes()).unwrap();
}

/// checking if writing a string yields the correct entries in the 'Table' struct
#[test]
fn test_db() {
    setup_mock_db();
    let mut db = db::DB::open(TEST_DB_PATH).unwrap();
    assert_eq!(db.get_table_count(), 1);
    println!("#Test# table count correct");
    
    test_db_content_integrity(&mut db);
    test_db_insert_to_all(&mut db);
}

fn test_db_content_integrity(db: &mut db::DB) {
    let table = db.get_mut_table(TEST_TABLE_NAME).unwrap();
    let table_vec = table.select_all_cols().unwrap();

    let mut test_vec: Vec<Vec<DBField>> = vec![vec![]];
    test_vec.push(vec![DBField::Int(0), DBField::Text("Bob".to_string())]);
    test_vec.push(vec![DBField::Int(1), DBField::Text("Alice".to_string())]);
    test_vec.push(vec![DBField::Int(2), DBField::Text("Rob".to_string())]);
    test_vec.push(vec![DBField::Int(3), DBField::Text("Jane".to_string())]);
    test_vec.push(vec![DBField::Int(4), DBField::Text("Tod".to_string())]);
    test_vec.push(vec![DBField::Int(5), DBField::Text("Ann".to_string())]);
    assert_eq!(test_vec, table_vec);
    println!("#Test# table integrity correct");
}

fn test_db_insert_to_all(db: &mut db::DB) {
    setup_mock_db();
    let table = db.get_mut_table(TEST_TABLE_NAME).unwrap();
    assert_eq!(table.insert(Option::None, vec![DBField::Int(1), DBField::Text("alice".to_string())]).unwrap(), ());
}

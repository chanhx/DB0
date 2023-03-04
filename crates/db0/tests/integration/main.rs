use {
    binder::Binder,
    db0,
    executor::Executor,
    parser::Parser,
    semantic_analyzer::Analyzer,
    std::sync::{Arc, RwLock},
    storage::{buffer::BufferManager, DEFAULT_PAGE_SIZE},
    tempfile::tempdir,
};

#[test]
fn insert_and_query() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path();

    db0::cmd::create_meta_tables(path).unwrap();

    let manager = BufferManager::new(100, DEFAULT_PAGE_SIZE, path.to_path_buf());
    let binder = Binder::new(1, &manager).unwrap();
    let binder = Arc::new(RwLock::new(binder));
    let analyzer = Analyzer::new(binder.clone());
    let executor = Executor::new(1, binder.clone());

    let sql = "
        CREATE TABLE abc (a int PRIMARY KEY, b boolean);
        INSERT INTO abc (a, b) VALUES (1, true);
        SELECT a, b FROM abc;
    ";
    let stmts = Parser::parse(sql).unwrap();

    for stmt in stmts {
        let stmt = analyzer.analyze(stmt).unwrap();
        let values = executor.execute(stmt, &manager).unwrap();

        dbg!(values);
    }

    temp_dir.close().unwrap()
}

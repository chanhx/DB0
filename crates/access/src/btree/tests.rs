use {
    super::*,
    crate::codec::Codec,
    def::{meta::Column, SqlType, Value},
    rand::prelude::*,
    storage::DEFAULT_PAGE_SIZE,
    tempfile::tempdir,
};

#[test]
fn sequential_insertion() -> Result<()> {
    let dir = tempdir().unwrap();

    let attr = Column::new(1, 1, "abc".to_string(), SqlType::TinyUint, 4, false);
    let codec = Codec::new(vec![attr]);

    let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
    let file_node = FileNode::new(1, 2, 3);

    BTree::<Codec>::init(file_node, &manager)?;
    let mut btree = BTree::new(codec, 30, file_node, &manager);

    let range = 0..120;

    for i in range.clone() {
        btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5])?;
    }

    for i in range {
        let (mut cursor, is_matched) = btree.cursor(&vec![Value::TinyUint(i)]).unwrap().unwrap();

        let (_, value) = cursor.next().unwrap();

        assert!(is_matched);
        assert_eq!(&[i * 2 + 5].as_ref(), &value);
    }

    dir.close().unwrap();

    Ok(())
}

#[test]
fn random_insertion() -> Result<()> {
    let dir = tempdir().unwrap();

    let attr = Column::new(1, 1, "abc".to_string(), SqlType::TinyUint, 4, false);
    let codec = Codec::new(vec![attr]);

    let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
    let file_node = FileNode::new(1, 2, 3);

    BTree::<Codec>::init(file_node, &manager)?;
    let mut btree = BTree::new(codec, 30, file_node, &manager);

    let mut rng = rand::thread_rng();
    let mut nums: Vec<u8> = (0..120).collect();
    nums.shuffle(&mut rng);

    for &i in nums.iter() {
        btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5])?;
    }

    for &i in nums.iter() {
        let (mut cursor, is_matched) = btree.cursor(&vec![Value::TinyUint(i)]).unwrap().unwrap();

        let (_, value) = cursor.next().unwrap();

        assert!(is_matched);
        assert_eq!(&[i * 2 + 5].as_ref(), &value);
    }

    dir.close().unwrap();

    Ok(())
}

#[test]
fn flush() -> Result<()> {
    let dir = tempdir().unwrap();

    let attr = Column::new(1, 1, "abc".to_string(), SqlType::TinyUint, 4, false);
    let key_codec = Codec::new(vec![attr]);

    let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
    let file_node = FileNode::new(1, 2, 3);

    BTree::<Codec>::init(file_node, &manager)?;
    let mut btree = BTree::new(key_codec.clone(), 30, file_node, &manager);

    let range = 0..120;

    for i in range.clone() {
        btree.insert(&vec![Value::TinyUint(i)], &[i * 2 + 5])?;
    }

    manager.flush_pages().unwrap();

    let manager = BufferManager::new(10, DEFAULT_PAGE_SIZE, dir.path().to_path_buf());
    let btree2 = BTree::new(key_codec, 30, file_node, &manager);

    for i in range {
        let (mut cursor, is_matched) = btree2.cursor(&vec![Value::TinyUint(i)]).unwrap().unwrap();

        let (_, value) = cursor.next().unwrap();

        assert!(is_matched);
        assert_eq!(&[i * 2 + 5].as_ref(), &value);
    }

    dir.close().unwrap();

    Ok(())
}

extern crate tempdir;
extern crate zbox;

mod common;

use std::{thread, time};
use std::sync::{Arc, RwLock};

#[test]
fn dir_create() {
    let mut env = common::setup();
    let repo = &mut env.repo;

    // #1: basic test
    repo.create_dir("/dir").unwrap();
    assert!(repo.create_dir("/dir").is_err());
    assert!(repo.create_dir("/xxx/yyy").is_err());
    repo.create_dir("/dir2").unwrap();
    repo.create_dir("/dir3").unwrap();
    assert!(repo.is_dir("/dir"));
    assert!(repo.is_dir("/dir2"));
    assert!(repo.is_dir("/dir3"));

    // #2: test create_dir_all
    repo.create_dir_all("/xxx/yyy").unwrap();
    repo.create_dir_all("/xxx/111/222").unwrap();

    // #3: check dir modify time
    let m = repo.metadata("/xxx/111/222").unwrap();
    thread::sleep(time::Duration::from_millis(1500));
    repo.create_dir_all("/xxx/111/222/333").unwrap();
    let m2 = repo.metadata("/xxx/111/222").unwrap();
    assert!(m2.modified() > m.modified());
}

#[test]
fn dir_create_mt() {
    let env = Arc::new(RwLock::new(common::setup()));
    let worker_cnt = 4;
    let task_cnt = 8;

    let mut workers = Vec::new();
    for i in 0..worker_cnt {
        let env = env.clone();
        workers.push(thread::spawn(move || {
            let base = i * task_cnt;
            for j in base..base + task_cnt {
                let path = format!("/mt/{}", j);
                let mut env = env.write().unwrap();
                env.repo.create_dir_all(&path).unwrap();
            }
        }));
    }
    for w in workers {
        w.join().unwrap();
    }

    // verify
    let env = env.read().unwrap();
    let dirs = env.repo.read_dir("/mt").unwrap();
    assert_eq!(dirs.len(), worker_cnt * task_cnt);
}

#[test]
fn dir_read() {
    let mut env = common::setup();
    let repo = &mut env.repo;

    repo.create_dir_all("/aaa/aaa1/aaa11").unwrap();
    repo.create_dir_all("/aaa/aaa1/aaa12").unwrap();
    repo.create_dir_all("/aaa/aaa2/").unwrap();
    repo.create_dir("/aaa/aaa2/xxx").unwrap();
    repo.create_dir_all("/bbb/bbb1").unwrap();
    repo.create_dir("/bbb/xxx").unwrap();
    repo.create_dir_all("/ccc").unwrap();

    let dirs = repo.read_dir("/").unwrap();
    assert_eq!(dirs.len(), 3);
    let dirs = repo.read_dir("/aaa").unwrap();
    assert_eq!(dirs.len(), 2);
    let dirs = repo.read_dir("/bbb").unwrap();
    assert_eq!(dirs.len(), 2);
    let dirs = repo.read_dir("/ccc").unwrap();
    assert_eq!(dirs.len(), 0);
}

#[test]
fn dir_remove() {
    let mut env = common::setup();
    let repo = &mut env.repo;

    repo.create_dir_all("/aaa/bbb/ccc").unwrap();
    repo.create_dir_all("/aaa/bbb/ddd").unwrap();
    assert!(repo.remove_dir("/aaa").is_err());
    assert!(repo.remove_dir("/aaa/bbb").is_err());
    repo.remove_dir("/aaa/bbb/ccc").unwrap();
    assert!(repo.remove_dir("/not_exist").is_err());
    repo.remove_dir_all("/aaa").unwrap();
    assert!(repo.remove_dir("/aaa").is_err());
    assert!(repo.remove_dir("/").is_err());
}

#[test]
fn dir_rename() {
    let mut env = common::setup();
    let repo = &mut env.repo;

    assert!(repo.rename("/", "/xxx").is_err());
    assert!(repo.rename("/not_exist", "/xxx").is_err());

    repo.create_dir_all("/aaa/bbb/ccc").unwrap();
    repo.rename("/aaa/bbb/ccc", "/aaa/ddd").unwrap();
    let dirs = repo.read_dir("/aaa/ddd").unwrap();
    assert_eq!(dirs.len(), 0);
    let dirs = repo.read_dir("/aaa").unwrap();
    assert_eq!(dirs.len(), 2);

    repo.create_dir_all("/3/8").unwrap();
    repo.rename("/3/8", "/3/14").unwrap();
    let dirs = repo.read_dir("/3").unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0].path().to_str().unwrap(), "/3/14");

    repo.create_dir("/15").unwrap();
    repo.create_dir("/10").unwrap();
    repo.rename("/10", "/15/21").unwrap();
    let dirs = repo.read_dir("/15").unwrap();
    assert_eq!(dirs.len(), 1);
    assert_eq!(dirs[0].path().to_str().unwrap(), "/15/21");
    repo.remove_dir("/15/21").unwrap();

    repo.create_dir_all("/0/3").unwrap();
    repo.create_dir_all("/0/4").unwrap();
    repo.create_dir("/17").unwrap();
    assert!(repo.rename("/17", "/0/4").is_err());
}

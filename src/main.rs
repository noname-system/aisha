

#[cfg(any(target_os = "linux"))]
use std::os::unix::prelude::OsStrExt;
//#[cfg(any(target_os = "windows"))]
//use std::os::windows::ffi::OsStrExt;
//use std::os::windows::prelude::OsStrExt;

use std::path::{PathBuf, Path};
use std::time::Duration;
use notify::event::{CreateKind, RemoveKind, ModifyKind, RenameMode};
use notify::event::{AccessKind, AccessMode};

use notify::{EventKind, Config};
use notify::*;
use sled::{self, Db};
/// Path of Sled DB in disk.
pub const SLED_PATH: &str = "./sled";



fn main() {
  
    let sled_db = {
        let config = sled::Config::default()
            .path(SLED_PATH)
            .use_compression(true)
            .print_profile_on_drop(true);
        
        let db = config.open().unwrap();
        //db.set_merge_operator(sled_cat);
        db
    };


    let (tx, rx) = std::sync::mpsc::channel();
    
    let mut watcher: Box<dyn Watcher> = if RecommendedWatcher::kind() == WatcherKind::PollWatcher  {
        let config = Config::default()
            //.with_compare_contents(true) // crucial part for pseudo filesystems 
            .with_poll_interval(Duration::from_secs(2));
        Box::new(PollWatcher::new(tx, config).unwrap())
    } else {
        Box::new(RecommendedWatcher::new(tx, Config::default()).unwrap())
    };

    watcher
        .watch(Path::new("."), RecursiveMode::NonRecursive)
        .unwrap();

    // all events, this blocks forever
    for e in rx {
        
        match e {
            Ok(ev) =>{
                println!("{:?}", ev);
                let p = ev.paths;
                match ev.kind {
                    EventKind::Any => {},
                    EventKind::Access(k) => {
                        match k {
                            AccessKind::Any => {},
                            AccessKind::Read => {},
                            AccessKind::Open(_) => {},
                            AccessKind::Close(m) => {
                                match m {
                                    AccessMode::Any => {},
                                    AccessMode::Execute => {},
                                    AccessMode::Read => {},
                                    AccessMode::Write => {}, //
                                    AccessMode::Other => {},
                                }
                            },
                            AccessKind::Other => {},
                        }
                    },
                    EventKind::Create(k) => {
                        match k {
                            CreateKind::Any => {},
                            CreateKind::File => { sled_db.add_index(p) }, //
                            CreateKind::Folder => { sled_db.add_index(p) }, //
                            CreateKind::Other => {},
                        }
                    },
                    EventKind::Modify(k) => {
                        match k {
                            ModifyKind::Any => {},
                            ModifyKind::Data(_) => {},
                            ModifyKind::Metadata(_) => {},
                            ModifyKind::Name(m) => {
                                match m {
                                    RenameMode::Any => {},
                                    RenameMode::To => {},
                                    RenameMode::From => {sled_db.delete_index(p)},
                                    RenameMode::Both => {},
                                    RenameMode::Other => {},
                                }
                            },
                            ModifyKind::Other => {},
                        }
                    },
                    EventKind::Remove(k) => {
                        match k {
                            RemoveKind::Any => {},
                            RemoveKind::File => {sled_db.delete_index(p)}, //
                            RemoveKind::Folder => {sled_db.delete_index(p)}, //
                            RemoveKind::Other => {},
                        }
                    },
                    EventKind::Other => {},
                }
            },
            Err(e) => { println!("{:?}", e) },
        }
    }


    
}


trait Index {
    fn add_index(&self, paths: Vec<PathBuf>);
    fn delete_index(&self, paths: Vec<PathBuf>);
}

impl Index for Db {

    fn add_index(&self, paths: Vec<PathBuf>) {

        for p in paths {
            let file_name = p.file_name().unwrap_or_default();
            let key = p.as_os_str().as_bytes();
            let old_v = self.insert(key, file_name.as_bytes()).unwrap().unwrap_or_default();
            let old_v = String::from_utf8_lossy(&old_v);
            println!("add_index: {file_name:?} old: {old_v:?}");
        }
    
    }
    
    fn delete_index(&self, paths: Vec<PathBuf>) {

        for p in paths {
            let key = p.as_os_str().as_bytes();
            let old_v = self.remove(key).unwrap().unwrap_or_default();
            let old_v = String::from_utf8_lossy(&old_v);
            println!("delete_index: {old_v:?}");
        }
    
    }

}


//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015-2018 Matthias Beyer <mail@beyermatthias.de> and contributors
//
// This library is free software; you can redistribute it and/or
// modify it under the terms of the GNU Lesser General Public
// License as published by the Free Software Foundation; version
// 2.1 of the License.
//
// This library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
// Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public
// License along with this library; if not, write to the Free Software
// Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA  02110-1301  USA
//

use std::path::PathBuf;
use std::fmt::Debug;
use std::collections::HashMap;
use std::sync::Arc;

use failure::Fallible as Result;

use store::Entry;
use storeid::StoreIdWithBase;

mod fs;
mod inmemory;
pub(crate) mod iter;

pub use self::fs::FSFileAbstraction;
pub use self::fs::FSFileAbstractionInstance;
pub use self::inmemory::InMemoryFileAbstraction;
pub use self::inmemory::InMemoryFileAbstractionInstance;
use self::iter::PathIterator;

/// An abstraction trait over filesystem actions
pub trait FileAbstraction : Debug {
    fn remove_file(&self, path: &PathBuf) -> Result<()>;
    fn copy(&self, from: &PathBuf, to: &PathBuf) -> Result<()>;
    fn rename(&self, from: &PathBuf, to: &PathBuf) -> Result<()>;
    fn create_dir_all(&self, _: &PathBuf) -> Result<()>;

    fn exists(&self, &PathBuf) -> Result<bool>;
    fn is_file(&self, &PathBuf) -> Result<bool>;

    fn new_instance(&self, p: PathBuf) -> Box<FileAbstractionInstance>;

    fn drain(&self) -> Result<Drain>;
    fn fill<'a>(&'a mut self, d: Drain) -> Result<()>;

    fn pathes_recursively<'a>(&self, basepath: PathBuf, storepath: &'a PathBuf, backend: Arc<FileAbstraction>) -> Result<PathIterator<'a>>;
}

/// An abstraction trait over actions on files
pub trait FileAbstractionInstance : Debug {

    /// Get the contents of the FileAbstractionInstance, as Entry object.
    ///
    /// The `StoreIdWithBase` is passed because the backend does not know where the Entry lives, but the
    /// Entry type itself must be constructed with the id.
    fn get_file_content<'a>(&mut self, id: StoreIdWithBase<'a>) -> Result<Option<Entry>>;
    fn write_file_content(&mut self, buf: &Entry) -> Result<()>;
}

pub struct Drain(HashMap<PathBuf, Entry>);

impl Drain {

    pub fn new(hm: HashMap<PathBuf, Entry>) -> Drain {
        Drain(hm)
    }

    pub fn empty() -> Drain {
        Drain::new(HashMap::new())
    }

    pub fn iter<'a>(&'a mut self) -> DrainIter<'a> {
        DrainIter(self.0.drain())
    }

}

pub struct DrainIter<'a>(::std::collections::hash_map::Drain<'a, PathBuf, Entry>);

impl<'a> Iterator for DrainIter<'a> {
    type Item = (PathBuf, Entry);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::FileAbstractionInstance;
    use super::inmemory::InMemoryFileAbstraction;
    use super::inmemory::InMemoryFileAbstractionInstance;
    use storeid::StoreIdWithBase;
    use store::Entry;

    #[test]
    fn lazy_file() {
        let store_path = PathBuf::from("/");
        let fs = InMemoryFileAbstraction::default();

        let mut path = PathBuf::from("tests");
        path.set_file_name("test1");
        let mut lf = InMemoryFileAbstractionInstance::new(fs.backend().clone(), path.clone());

        let loca = StoreIdWithBase::new(&store_path, path);
        let file = Entry::from_str(loca.clone(), &format!(r#"---
[imag]
version = "{}"
---
Hello World"#, env!("CARGO_PKG_VERSION"))).unwrap();

        lf.write_file_content(&file).unwrap();
        let bah = lf.get_file_content(loca).unwrap().unwrap();
        assert_eq!(bah.get_content(), "Hello World");
    }

    #[test]
    fn lazy_file_multiline() {
        let store_path = PathBuf::from("/");
        let fs = InMemoryFileAbstraction::default();

        let mut path = PathBuf::from("tests");
        path.set_file_name("test1");
        let mut lf = InMemoryFileAbstractionInstance::new(fs.backend().clone(), path.clone());

        let loca = StoreIdWithBase::new(&store_path, path);
        let file = Entry::from_str(loca.clone(), &format!(r#"---
[imag]
version = "{}"
---
Hello World
baz"#, env!("CARGO_PKG_VERSION"))).unwrap();

        lf.write_file_content(&file).unwrap();
        let bah = lf.get_file_content(loca).unwrap().unwrap();
        assert_eq!(bah.get_content(), "Hello World\nbaz");
    }

    #[test]
    fn lazy_file_multiline_trailing_newlines() {
        let store_path = PathBuf::from("/");
        let fs = InMemoryFileAbstraction::default();

        let mut path = PathBuf::from("tests");
        path.set_file_name("test1");
        let mut lf = InMemoryFileAbstractionInstance::new(fs.backend().clone(), path.clone());

        let loca = StoreIdWithBase::new(&store_path, path);
        let file = Entry::from_str(loca.clone(), &format!(r#"---
[imag]
version = "{}"
---
Hello World
baz

"#, env!("CARGO_PKG_VERSION"))).unwrap();

        lf.write_file_content(&file).unwrap();
        let bah = lf.get_file_content(loca).unwrap().unwrap();
        assert_eq!(bah.get_content(), "Hello World\nbaz\n\n");
    }

}


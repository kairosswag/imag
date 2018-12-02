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
use std::collections::HashMap;
use std::sync::Mutex;
use std::cell::RefCell;
use std::sync::Arc;
use std::ops::Deref;

use libimagerror::errors::ErrorMsg as EM;

use failure::Fallible as Result;
use failure::Error;

use super::FileAbstraction;
use super::FileAbstractionInstance;
use super::Drain;
use store::Entry;
use storeid::StoreIdWithBase;
use file_abstraction::iter::PathIterator;
use file_abstraction::iter::PathIterBuilder;

type Backend = Arc<Mutex<RefCell<HashMap<PathBuf, Entry>>>>;

/// `FileAbstraction` type, this is the Test version!
///
/// A lazy file is either absent, but a path to it is available, or it is present.
#[derive(Debug)]
pub struct InMemoryFileAbstractionInstance {
    fs_abstraction: Backend,
    absent_path: PathBuf,
}

impl InMemoryFileAbstractionInstance {

    pub fn new(fs: Backend, pb: PathBuf) -> InMemoryFileAbstractionInstance {
        InMemoryFileAbstractionInstance {
            fs_abstraction: fs,
            absent_path: pb
        }
    }

}

impl FileAbstractionInstance for InMemoryFileAbstractionInstance {

    /**
     * Get the mutable file behind a InMemoryFileAbstraction object
     */
    fn get_file_content(&mut self, _: StoreIdWithBase<'_>) -> Result<Option<Entry>> {
        debug!("Getting lazy file: {:?}", self);

        self.fs_abstraction
            .lock()
            .map_err(|_| Error::from(EM::LockError))
            .map(|mut mtx| {
                mtx.get_mut()
                    .get(&self.absent_path)
                    .cloned()
            })
            .map_err(Error::from)
    }

    fn write_file_content(&mut self, buf: &Entry) -> Result<()> {
        match *self {
            InMemoryFileAbstractionInstance { ref absent_path, .. } => {
                let mut mtx = self.fs_abstraction.lock().expect("Locking Mutex failed");
                let backend = mtx.get_mut();
                let _ = backend.insert(absent_path.clone(), buf.clone());
                return Ok(());
            },
        };
    }
}

#[derive(Debug, Default)]
pub struct InMemoryFileAbstraction {
    virtual_filesystem: Backend,
}

impl InMemoryFileAbstraction {

    pub fn backend(&self) -> &Backend {
        &self.virtual_filesystem
    }

    fn backend_cloned<'a>(&'a self) -> Result<HashMap<PathBuf, Entry>> {
        self.virtual_filesystem
            .lock()
            .map_err(|_| Error::from(EM::LockError))
            .map(|mtx| mtx.deref().borrow().clone())
            .into()
    }

}

impl FileAbstraction for InMemoryFileAbstraction {

    fn remove_file(&self, path: &PathBuf) -> Result<()> {
        debug!("Removing: {:?}", path);
        self.backend()
            .lock()
            .expect("Locking Mutex failed")
            .get_mut()
            .remove(path)
            .map(|_| ())
            .ok_or_else(|| EM::FileNotFound.into())
    }

    fn copy(&self, from: &PathBuf, to: &PathBuf) -> Result<()> {
        debug!("Copying : {:?} -> {:?}", from, to);
        let mut mtx = self.backend().lock().expect("Locking Mutex failed");
        let backend = mtx.get_mut();

        let a = backend.get(from).cloned().ok_or_else(|| EM::FileNotFound)?;
        backend.insert(to.clone(), a);
        debug!("Copying: {:?} -> {:?} worked", from, to);
        Ok(())
    }

    fn rename(&self, from: &PathBuf, to: &PathBuf) -> Result<()> {
        debug!("Renaming: {:?} -> {:?}", from, to);
        let mut mtx = self.backend().lock().expect("Locking Mutex failed");
        let backend = mtx.get_mut();

        let a = backend.remove(from).ok_or_else(|| EM::FileNotFound)?;
        backend.insert(to.clone(), a);
        debug!("Renaming: {:?} -> {:?} worked", from, to);
        Ok(())
    }

    fn create_dir_all(&self, _: &PathBuf) -> Result<()> {
        Ok(())
    }

    fn exists(&self, pb: &PathBuf) -> Result<bool> {
        let mut mtx = self.backend().lock().expect("Locking Mutex failed");
        let backend = mtx.get_mut();

        Ok(backend.contains_key(pb))
    }

    fn is_file(&self, pb: &PathBuf) -> Result<bool> {
        // Because we only store Entries in the memory-internal backend, we only have to check for
        // existance here, as if a path exists in the inmemory storage, it is always mapped to an
        // entry. hence it is always a path to a file
        self.exists(pb)
    }

    fn new_instance(&self, p: PathBuf) -> Box<FileAbstractionInstance> {
        Box::new(InMemoryFileAbstractionInstance::new(self.backend().clone(), p))
    }

    fn drain(&self) -> Result<Drain> {
        self.backend_cloned().map(Drain::new)
    }

    fn fill<'a>(&'a mut self, mut d: Drain) -> Result<()> {
        debug!("Draining into : {:?}", self);
        let mut mtx = self.backend()
            .lock()
            .map_err(|_| EM::LockError)?;
        let backend = mtx.get_mut();

        for (path, element) in d.iter() {
            debug!("Drain into {:?}: {:?}", self, path);
            backend.insert(path, element);
        }

        Ok(())
    }

    fn pathes_recursively<'a>(&self, _basepath: PathBuf, storepath: &'a PathBuf, backend: Arc<FileAbstraction>) -> Result<PathIterator<'a>> {
        trace!("Building PathIterator object (inmemory implementation)");
        let keys : Vec<PathBuf> = self
            .backend()
            .lock()
            .map_err(|_| EM::LockError)?
            .get_mut()
            .keys()
            .map(PathBuf::from)
            .map(Ok)
            .collect::<Result<_>>()?; // we have to collect() because of the lock() above.

        Ok(PathIterator::new(Box::new(InMemPathIterBuilder(keys)), storepath, backend))
    }
}

pub struct InMemPathIterBuilder(Vec<PathBuf>);

impl PathIterBuilder for InMemPathIterBuilder {
    fn build_iter(&self) -> Box<Iterator<Item = Result<PathBuf>>> {
        Box::new(self.0.clone().into_iter().map(Ok))
    }

    fn in_collection(&mut self, c: &str) {
        self.0.retain(|p| p.starts_with(c));
    }
}


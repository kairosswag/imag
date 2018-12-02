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

use std::fs::{File, OpenOptions, create_dir_all, remove_file, copy, rename};
use std::io::{Seek, SeekFrom, Read};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use libimagerror::errors::ErrorMsg as EM;

use super::FileAbstraction;
use super::FileAbstractionInstance;
use super::Drain;
use store::Entry;
use storeid::StoreIdWithBase;
use file_abstraction::iter::PathIterator;
use file_abstraction::iter::PathIterBuilder;

use walkdir::WalkDir;
use failure::ResultExt;
use failure::Fallible as Result;
use failure::Error;

#[derive(Debug)]
pub struct FSFileAbstractionInstance(PathBuf);

impl FileAbstractionInstance for FSFileAbstractionInstance {

    /**
     * Get the content behind this file
     */
    fn get_file_content<'a>(&mut self, id: StoreIdWithBase<'a>) -> Result<Option<Entry>> {
        debug!("Getting lazy file: {:?}", self);

        let mut file = match open_file(&self.0) {
            Err(err)       => return Err(Error::from(err)),
            Ok(None)       => return Ok(None),
            Ok(Some(file)) => file,
        };

        file.seek(SeekFrom::Start(0)).context(EM::FileNotSeeked)?;

        let mut s = String::new();

        file.read_to_string(&mut s)
            .context(EM::IO)
            .map_err(Error::from)
            .map(|_| s)
            .and_then(|s: String| Entry::from_str(id, &s))
            .map(Some)
    }

    /**
     * Write the content of this file
     */
    fn write_file_content(&mut self, buf: &Entry) -> Result<()> {
        use std::io::Write;

        let buf      = buf.to_str()?.into_bytes();
        let mut file = create_file(&self.0).context(EM::FileNotCreated)?;

        file.seek(SeekFrom::Start(0)).context(EM::FileNotCreated)?;
        file.set_len(buf.len() as u64).context(EM::FileNotWritten)?;
        file.write_all(&buf)
            .context(EM::FileNotWritten)
            .map_err(Error::from)
    }
}

/// `FSFileAbstraction` state type
///
/// A lazy file is either absent, but a path to it is available, or it is present.
#[derive(Debug, Default)]
pub struct FSFileAbstraction {}

impl FileAbstraction for FSFileAbstraction {

    fn remove_file(&self, path: &PathBuf) -> Result<()> {
        remove_file(path)
            .context(EM::FileNotRemoved)
            .map_err(Error::from)
    }

    fn copy(&self, from: &PathBuf, to: &PathBuf) -> Result<()> {
        copy(from, to)
            .map(|_| ())
            .context(EM::FileNotCopied)
            .map_err(Error::from)
    }

    fn rename(&self, from: &PathBuf, to: &PathBuf) -> Result<()> {
        if let Some(p) = to.parent() {
            if !p.exists() {
                debug!("Creating: {:?}", p);
                let _ = create_dir_all(&p).context(EM::DirNotCreated)?;
            }
        } else {
            debug!("Failed to find parent. This looks like it will fail now");
            //nothing
        }

        debug!("Renaming {:?} to {:?}", from, to);
        rename(from, to)
            .context(EM::FileNotRenamed)
            .map_err(Error::from)
    }

    fn create_dir_all(&self, path: &PathBuf) -> Result<()> {
        debug!("Creating: {:?}", path);
        create_dir_all(path)
            .context(EM::DirNotCreated)
            .map_err(Error::from)
    }

    fn exists(&self, path: &PathBuf) -> Result<bool> {
        Ok(path.exists())
    }

    fn is_file(&self, path: &PathBuf) -> Result<bool> {
        Ok(path.is_file())
    }

    fn new_instance(&self, p: PathBuf) -> Box<FileAbstractionInstance> {
        Box::new(FSFileAbstractionInstance(p))
    }

    /// We return nothing from the FS here.
    fn drain(&self) -> Result<Drain> {
        Ok(Drain::empty())
    }

    /// FileAbstraction::fill implementation that consumes the Drain and writes everything to the
    /// filesystem
    fn fill(&mut self, mut d: Drain) -> Result<()> {
        d.iter().fold(Ok(()), |acc, (path, element)| {
            acc.and_then(|_| self.new_instance(path).write_file_content(&element))
        })
    }

    fn pathes_recursively<'a>(&self,
                          basepath: PathBuf,
                          storepath: &'a PathBuf,
                          backend: Arc<FileAbstraction>)
        -> Result<PathIterator<'a>>
    {
        trace!("Building PathIterator object");
        Ok(PathIterator::new(Box::new(WalkDirPathIterBuilder { basepath }), storepath, backend))
    }
}

pub struct WalkDirPathIterBuilder {
    basepath: PathBuf
}

impl PathIterBuilder for WalkDirPathIterBuilder {
    fn build_iter(&self) -> Box<Iterator<Item = Result<PathBuf>>> {
        Box::new(WalkDir::new(self.basepath.clone())
            .min_depth(1)
            .max_open(100)
            .into_iter()
            .map(|r| {
                r.map(|e| PathBuf::from(e.path()))
                    .context(format_err!("Error in Walkdir"))
                    .map_err(Error::from)
            }))
    }

    fn in_collection(&mut self, c: &str) {
        self.basepath.push(c);
    }
}

fn open_file<A: AsRef<Path>>(p: A) -> ::std::io::Result<Option<File>> {
    match OpenOptions::new().write(true).read(true).open(p) {
        Err(e) => match e.kind() {
            ::std::io::ErrorKind::NotFound => return Ok(None),
            _ => return Err(e),
        },
        Ok(file) => Ok(Some(file))
    }
}

fn create_file<A: AsRef<Path>>(p: A) -> ::std::io::Result<File> {
    if let Some(parent) = p.as_ref().parent() {
        trace!("'{}' is directory = {}", parent.display(), parent.is_dir());
        if !parent.is_dir() {
            trace!("Implicitely creating directory: {:?}", parent);
            let _ = create_dir_all(parent)?;
        }
    }
    OpenOptions::new().write(true).read(true).create(true).open(p)
}


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

use std::path::Path;
use std::path::PathBuf;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreId;

use toml::Value;
use failure::Fallible as Result;


pub trait RefStore<'a> {
    fn get_ref(&self, id: StoreId) -> Result<Option<FileLockEntry<'a>>>;

    fn create_ref(&self, id: StoreId, config: &Value, pathpart: &PathBuf)
        -> Result<FileLockEntry<'a>>;

    fn retrieve_ref(&self, id: StoreId, config: &Value, pathpart: &PathBuf)
        -> Result<FileLockEntry<'a>>;
}

impl<'a> RefStore<'a> for Store {
    /// Get a Ref object from the Store.
    ///
    /// If the entry exists in the store, but the entry is not a ref, this returns an error.
    fn get_ref(&self, id: StoreId) -> Result<Option<FileLockEntry<'a>>> {
    }

    fn create_ref(&self, id: StoreId, config: &Value, pathpart: &PathBuf) -> Result<FileLockEntry<'a>> {
    }

    /// Retrieve a Ref object from the Store.
    ///
    /// If the entry exists in the store, but the entry is not a ref, this returns an error.
    fn retrieve_ref(&self, id: StoreId, config: &Value, pathpart: &PathBuf) -> Result<FileLockEntry<'a>> {
        match self.get_ref(id)? {
            Some(r) => Ok(r),
            None    => self.create_ref(id, config, pathpart),
        }
    }
}

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

use libimagentryutil::isa::Is;
use libimagentryutil::isa::IsKindHeaderPathProvider;
use libimagstore::store::Entry;
use libimagerror::errors::ErrorMsg as EM;

use toml::Value;
use toml_query::read::TomlValueReadExt;
use toml_query::delete::TomlValueDeleteExt;
use toml_query::insert::TomlValueInsertExt;
use failure::Fallible as Result;
use failure::Error;

use refstore::UniqueRefPathGenerator;

pub<H, P> trait Ref<H, P>
    where H: Hasher = Sha1Hasher,
          P: ConfigPathProvider = DefaultConfigPathProvider,
{

    /// Check whether the underlying object is actually a ref
    fn is_ref(&self) -> Result<bool>;

    /// Get the stored hash.
    fn get_stored_hash(&self) -> Result<&str>;

    /// Get the stored hash.
    fn get_stored_path(&self) -> Result<PathBuf>;

    /// Check whether the referenced file still matches its hash
    fn hash_valid<C>(&self) -> Result<bool>
        where C: AsRef<str>;

    fn remove_ref(&mut self) -> Result<()>;

    /// Make a ref out of a normal (non-ref) entry.
    ///
    /// If the entry is already a ref, this fails if `force` is false
    fn make_ref<P, C>(&mut self, force: bool, path: P, config: &Value) -> Result<()>
        where P: AsRef<Path>,
              C: AsRef<str>;
}

provide_kindflag_path!(pub IsRef, "ref.is_ref");

pub<H, P> impl Ref<H, P> for Entry {
    where H: Hasher = Sha1Hasher,
          P: ConfigPathProvider = DefaultConfigPathProvider,

    /// Check whether the underlying object is actually a ref
    fn is_ref(&self) -> Result<bool> {
        self.is::<IsRef>().map_err(Error::from)
    }

    fn get_hash(&self) -> Result<&str> {
        let header_path = format!("ref.hash.{}", H::NAME);
        self.get_header()
            .read(&header_path)
            .map_err(Error::from)?
            .ok_or_else(|| {
                Error::from(EM::EntryHeaderFieldMissing(header_path.to_owned()))
            })
            .and_then(|v| {
                v.as_str().ok_or_else(|| {
                    Error::from(EM::EntryHeaderTypeError2(header_path.to_owned(), "string"))
                })
            })
    }

    fn get_path(&self) -> Result<PathBuf> {
        self.get_header()
            .read("ref.path")
            .map_err(Error::from)?
            .ok_or_else(|| Error::from(EM::EntryHeaderFieldMissing("ref.path")))
            .and_then(|v| {
                v.as_str()
                    .ok_or_else(|| EM::EntryHeaderTypeError2("ref.path", "string"))
                    .map_err(Error::from)
            })
            .map(PathBuf::from)
    }

    fn hash_valid<C>(&self, collection_name: C) -> Result<bool>
        where C: AsRef<str>
    {
        let header_path = format!("ref.hash.{}", H::NAME);
        let path = self.get_path()?;

        self.get_header()
            .read(&header_path)
            .map_err(Error::from)?
            .ok_or_else(|| {
                Error::from(EM::EntryHeaderFieldMissing(header_path.to_owned()))
            })
            .and_then(|v| {
                v.as_str().ok_or_else(|| {
                    Error::from(EM::EntryHeaderTypeError2(header_path.to_owned(), "string"))
                })
            })
            .and_then(|hash| H::hash(path)? == hash)
    }

    fn remove_ref(&mut self) -> Result<()>; {
        unimplemented!()
    }

    /// Make a ref out of a normal (non-ref) entry.
    ///
    /// `path` is the path to refer to,
    ///
    /// # Warning
    ///
    /// If the entry is already a ref, this fails if `force` is false
    ///
    fn make_ref<P, C>(&mut self, path: P, collection_name: S, config: &Value, force: bool) -> Result<()>
        where P: AsRef<Path>,
              C: AsRef<str>
    {
        if self.is_ref()? {
            return Err(Error::from(err_msg("Entry is already a reference")))
        }

        let collection_config_path = format!("{}/{}", P::CONFIG_COLLECTIONS_PATH, collection_name);
        let file_path_directory = config.read(collection_config_path)?; // TODO

        let filepath = format!("{}/{}", file_path_directory, path);
        let hash = H::hash(filepath);

        // TODO

        entry.set_isflag::<IsRef>()?;
        Ok(())
    }

}


pub trait Hasher {
    const NAME: &'static str;

    /// hash the file at path `path`
    fn hash<P: AsRef<Path>>(path: P) -> Result<String>;
}

struct Sha1Hasher;
impl Hasher for Sha1Hasher {
    const NAME : &'static str = "sha1";

    fn hash<P: AsRef<Path>>(path: P) -> Result<String> {
        unimplemented!()
    }
}



/// A trait for providing the path (as in "toml-query") to the collections configuration for the
/// entryref library.
pub trait ConfigPathProvider {
    const CONFIG_PATH: &'static str;
}

struct DefaultConfigPathProvider;
impl ConfigPathProvider for DefaultConfigPathProvider {
    const CONFIG_COLLECTIONS_PATH: &'static str = "entryref.collections";
}

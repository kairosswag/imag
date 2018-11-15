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
use std::collections::BTreeMap;
use std::fmt::Debug;

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
use failure::err_msg;
use failure::ResultExt;

pub trait Ref<H = Sha1Hasher, C = DefaultConfigPathProvider>
    where H: Hasher,
          C: ConfigPathProvider,
{

    /// Check whether the underlying object is actually a ref
    fn is_ref(&self) -> Result<bool>;

    /// Get the stored hash.
    fn get_path(&self) -> Result<PathBuf>;

    /// Get the stored hash.
    fn get_hash(&self) -> Result<&str>;

    /// Check whether the referenced file still matches its hash
    fn hash_valid(&self, config: &Value) -> Result<bool>;

    fn remove_ref(&mut self) -> Result<()>;

    /// Make a ref out of a normal (non-ref) entry.
    ///
    /// If the entry is already a ref, this fails if `force` is false
    fn make_ref<P, Coll>(&mut self, path: P, collection_name: Coll, config: &Value, force: bool)
        -> Result<()>
        where P: AsRef<Path> + Debug,
              Coll: AsRef<str> + Debug;
}

provide_kindflag_path!(pub IsRef, "ref.is_ref");

impl<H, C> Ref<H, C> for Entry
    where H: Hasher,
          C: ConfigPathProvider,
{

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
                Error::from(EM::EntryHeaderFieldMissing("ref.hash.<hash>"))
            })
            .and_then(|v| {
                v.as_str().ok_or_else(|| {
                    Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string"))
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

    fn hash_valid(&self, config: &Value) -> Result<bool> {
        let ref_header = self.get_header()
            .read("ref")?
            .ok_or_else(|| err_msg("Header missing at 'ref'"))?;

        let collection_name = ref_header
            .read("collection")
            .map_err(Error::from)?
            .ok_or_else(|| err_msg("Header missing at 'ref.collection'"))?
            .as_str()
            .ok_or_else(|| Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string")))?;

        let path = ref_header
            .read("path")
            .map_err(Error::from)?
            .ok_or_else(|| err_msg("Header missing at 'ref.path'"))?
            .as_str()
            .map(PathBuf::from)
            .ok_or_else(|| Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string")))?;


        let file_path = get_file_path::<C, _, _>(config, collection_name, &path)?;

        ref_header
            .read(H::NAME)
            .map_err(Error::from)?
            .ok_or_else(|| format_err!("Header missing at 'ref.{}'", H::NAME))
            .and_then(|v| {
                v.as_str().ok_or_else(|| {
                    Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string"))
                })
            })
            .and_then(|hash| H::hash(path).map(|h| h == hash))
    }

    fn remove_ref(&mut self) -> Result<()> {
        debug!("Removing 'ref' header section");
        let _ = self.get_header_mut().delete("ref").context("Removing ref")?;

        debug!("Removing 'ref' header marker");
        self.set_isflag::<IsRef>().context("Removing ref").map_err(Error::from)
    }

    /// Make a ref out of a normal (non-ref) entry.
    ///
    /// `path` is the path to refer to,
    ///
    /// # Warning
    ///
    /// If the entry is already a ref, this fails if `force` is false
    ///
    fn make_ref<P, Coll>(&mut self, path: P, collection_name: Coll, config: &Value, force: bool)
        -> Result<()>
        where P: AsRef<Path> + Debug,
              Coll: AsRef<str> + Debug
    {
        if self.is::<IsRef>()? {
            let _ = Err(err_msg("Entry is already a reference")).context("Making ref out of entry")?;
        }

        let file_path = get_file_path::<C, _, _>(config, &collection_name, &path)?;

        if !file_path.exists() {
            let msg = format_err!("File '{:?}' does not exist", file_path);
            let _   = Err(msg).context("Making ref out of entry")?;
        }

        let _ = H::hash(&file_path)
            .and_then(|hash| make_header_section(hash, H::NAME, path, collection_name))
            .and_then(|h| self.get_header_mut().insert("ref", h).map_err(Error::from))
            .and_then(|_| self.set_isflag::<IsRef>())
            .context("Making ref out of entry")?;

        Ok(())
    }

}


pub trait Hasher {
    const NAME: &'static str;

    /// hash the file at path `path`
    fn hash<P: AsRef<Path>>(path: P) -> Result<String>;
}

pub struct Sha1Hasher;
impl Hasher for Sha1Hasher {
    const NAME : &'static str = "sha1";

    fn hash<P: AsRef<Path>>(path: P) -> Result<String> {
        unimplemented!()
    }
}



/// A trait for providing the path (as in "toml-query") to the collections configuration for the
/// entryref library.
pub trait ConfigPathProvider {
    const CONFIG_COLLECTIONS_PATH: &'static str;
}

pub struct DefaultConfigPathProvider;
impl ConfigPathProvider for DefaultConfigPathProvider {
    const CONFIG_COLLECTIONS_PATH: &'static str = "entryref.collections";
}

/// Create a new header section for a "ref".
///
/// # Warning
///
/// The `relpath` _must_ be relative to the configured path for that collection.
pub(crate) fn make_header_section<P, C, H>(hash: String, hashname: H, relpath: P, collection: C)
    -> Result<Value>
    where P: AsRef<Path> + Debug,
          C: AsRef<str>,
          H: AsRef<str>,
{
    let mut header_section = Value::Table(BTreeMap::new());
    {
        let relpath = relpath
            .as_ref()
            .to_str()
            .map(String::from)
            .ok_or_else(|| {
                let msg = format_err!("UTF Error in '{:?}'", relpath);
                Error::from(msg)
            })?;

        let _ = header_section.insert("relpath", Value::String(relpath))?;
    }

    {
        let mut hash_table = Value::Table(BTreeMap::new());
        let _ = hash_table.insert(hashname.as_ref(), Value::String(hash))?;
        let _ = header_section.insert("hash", hash_table)?;
    }

    let _ = header_section.insert("collection", Value::String(String::from(collection.as_ref())));

    Ok(header_section)
}

fn get_file_path<C, Coll, P>(config: &Value, collection_name: Coll, path: P) -> Result<PathBuf>
        where P: AsRef<Path> + Debug,
              Coll: AsRef<str> + Debug,
              C: ConfigPathProvider
{
    let collection_config_path = PathBuf::from(C::CONFIG_COLLECTIONS_PATH)
        .join(PathBuf::from(collection_name.as_ref()));

    let file_path = config
        .read(&collection_config_path.to_str().ok_or_else(|| Error::from(EM::UTF8Error))?)
        .context("Making ref out of entry")?
        .ok_or_else(|| {
            format_err!("Configuration missing at '{:?}'", collection_config_path)
        })
        .and_then(|v| v.as_str().ok_or_else(|| {
            format_err!("Configuration type at '{:?}' should be 'string'", collection_config_path)
        }))
        .map(String::from)
        .map(PathBuf::from)
        .context("Making ref out of entry")?
        .join(&path);

    Ok(file_path)
}


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
use std::ops::Deref;

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

use hasher::Hasher;
use hasher::sha1::Sha1Hasher;

/// A configuration of "collection name" -> "collection path" mappings
///
/// Should be deserializeable from the configuration file right away, because we expect a
/// configuration like this in the config file:
///
/// ```toml
/// [ref.collections]
/// music = "/home/alice/music"
/// documents = "/home/alice/doc"
/// ```
///
/// for example.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config(BTreeMap<String, PathBuf>);

impl Deref for Config {
    type Target = BTreeMap<String, PathBuf>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Ref<H : Hasher = Sha1Hasher> {

    /// Check whether the underlying object is actually a ref
    fn is_ref(&self) -> Result<bool>;

    /// Get the stored hash.
    fn get_path(&self) -> Result<PathBuf>;

    /// Get the stored hash.
    fn get_hash(&self) -> Result<&str>;

    /// Check whether the referenced file still matches its hash
    fn hash_valid(&self, config: &Config) -> Result<bool>;

    fn remove_ref(&mut self) -> Result<()>;

    /// Make a ref out of a normal (non-ref) entry.
    ///
    /// If the entry is already a ref, this fails if `force` is false
    fn make_ref<P, Coll>(&mut self, path: P, collection_name: Coll, config: &Config, force: bool)
        -> Result<()>
        where P: AsRef<Path> + Debug,
              Coll: AsRef<str> + Debug;
}

provide_kindflag_path!(pub IsRef, "ref.is_ref");

impl<H: Hasher> Ref<H> for Entry {

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

    fn hash_valid(&self, config: &Config) -> Result<bool> {
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


        let file_path = get_file_path(config, collection_name.as_ref(), &path)?;

        ref_header
            .read(H::NAME)
            .map_err(Error::from)?
            .ok_or_else(|| format_err!("Header missing at 'ref.{}'", H::NAME))
            .and_then(|v| {
                v.as_str().ok_or_else(|| {
                    Error::from(EM::EntryHeaderTypeError2("ref.hash.<hash>", "string"))
                })
            })
            .and_then(|hash| H::hash(file_path).map(|h| h == hash))
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
    fn make_ref<P, Coll>(&mut self, path: P, collection_name: Coll, config: &Config, force: bool)
        -> Result<()>
        where P: AsRef<Path> + Debug,
              Coll: AsRef<str> + Debug
    {
        if !force && self.is::<IsRef>()? {
            let _ = Err(err_msg("Entry is already a reference")).context("Making ref out of entry")?;
        }

        let file_path = get_file_path(config, collection_name.as_ref(), &path)?;

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

fn get_file_path<P>(config: &Config, collection_name: &str, path: P) -> Result<PathBuf>
        where P: AsRef<Path> + Debug
{
    config
        .get(collection_name)
        .map(PathBuf::clone)
        .ok_or_else(|| {
            format_err!("Configuration missing for collection: '{}'", collection_name)
        })
        .context("Making ref out of entry")
        .map_err(Error::from)
        .map(|p| p.join(&path))
}


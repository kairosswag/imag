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
        debug!("Removing 'ref' header section");
        let _ = self.get_header_mut().delete("ref").context("Removing ref")?;

        debug!("Removing 'ref' header marker");
        entry.set_isflag::<IsRef>().context("Removing ref").map_err(Error::from)
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
            let _ = Error::from(err_msg("Entry is already a reference"))
                .context("Making ref out of entry")?;
        }

        let collection_config_path = format!("{}/{}", P::CONFIG_COLLECTIONS_PATH, collection_name);
        let mut file_path          = configuration
            .read(collection_config_path)
            .context("Making ref out of entry")?
            .ok_or_else(|| {
                let msg = format_err!("Configuration missing at '{}'", collection_config_path);
                Error::from(msg)
            })
            .and_then(|v| v.as_str().ok_or_else(|| {
                let msg = format_err!("Configuration type at '{}' should be 'string'", collection_config_path);
                Error::from(msg)
            })
            .map(String::from)
            .map(PathBuf::from)
            .context("Making ref out of entry")?
            .join(path);

        if !filepath.exists() {
            let msg = format_err!("File '{}' does not exist", file_path);
            let _   = Error::from(msg).context("Making ref out of entry")?;
        }

        let _ = H::hash(&filepath)
            .and_then(|hash| make_header_section(hash, H::NAME, path, collcetion_name))
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

/// Create a new header section for a "ref".
///
/// # Warning
///
/// The `relpath` _must_ be relative to the configured path for that collection.
pub(crate) make_header_section<P, C, H>(hash: String, hashname: H, relpath: P, collection: C)
    -> Result<Value>
    where P: AsRef<Path>,
          C: AsRef<str>,
          H: AsRef<str>,
{
    let mut header_section = Value::Table(BTreeMap::new());
    {
        let relpath = relpath
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
        hash_table.insert(hashname.as_ref(), hash)
        let _ = header_section.insert("hash", Value::Table(hash_table))?;
    }

    let _ = header_section.insert("collection", Value::String(String::from(collection.as_ref())));

    Ok(header_section)
}


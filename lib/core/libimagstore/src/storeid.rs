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

use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;

use std::fmt::{Display, Debug, Formatter};
use std::fmt::Error as FmtError;
use std::result::Result as RResult;
use std::path::Components;

use failure::ResultExt;
use failure::Fallible as Result;
use failure::err_msg;
use failure::Error;

use store::Store;

use iter::create::StoreCreateIterator;
use iter::delete::StoreDeleteIterator;
use iter::get::StoreGetIterator;
use iter::retrieve::StoreRetrieveIterator;

/// The Index into the Store
///
/// A StoreId object is a unique identifier for one entry in the store which might be present or
/// not.
///
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct StoreId(PathBuf);

impl StoreId {

    pub fn new(id: PathBuf) -> Result<StoreId> {
        debug!("Trying to get a new baseless id from: {:?}", id);
        if id.is_absolute() {
            debug!("Error: Id is absolute!");
            Err(format_err!("Store Id local part is absolute: {}", id.display()))
        } else {
            debug!("Building Storeid object baseless");
            Ok(StoreId(id))
        }
    }

    pub fn with_base<'a>(self, base: &'a PathBuf) -> StoreIdWithBase<'a> {
        StoreIdWithBase(base, self.0)
    }

    pub fn to_str(&self) -> Result<String> {
        Ok(self.0.display().to_string())
    }

    /// Helper function for creating a displayable String from StoreId
    ///
    /// This is safe because the
    ///
    /// ```ignore
    ///     impl<T: fmt::Display + ?Sized> ToString for T
    /// ```
    ///
    /// does only fail if Display::display() failed. The implementation of ::std::path::Display and
    /// the implementation ::std::fmt::Display for ::std::path::Display do not return errors though.
    pub fn local_display_string(&self) -> String {
        self.local().display().to_string()
    }

    /// Returns the components of the `id` part of the StoreId object.
    ///
    /// Can be used to check whether a StoreId points to an entry in a specific collection of
    /// StoreIds.
    pub fn components(&self) -> Components {
        self.0.components()
    }

    /// Get the _local_ part of a StoreId object, as in "the part from the store root to the entry".
    pub fn local(&self) -> &PathBuf {
        &self.0
    }

    /// Check whether a StoreId points to an entry in a specific collection.
    ///
    /// A "collection" here is simply a directory. So `foo/bar/baz` is an entry which is in
    /// collection ["foo", "bar", "baz"], but also in ["foo", "bar"] and ["foo"].
    ///
    /// # Warning
    ///
    /// The collection specification _has_ to start with the module name. Otherwise this function
    /// may return false negatives.
    ///
    pub fn is_in_collection<S: AsRef<str>, V: AsRef<[S]>>(&self, colls: &V) -> bool {
        use std::path::Component;

        self.0
            .components()
            .zip(colls.as_ref().iter())
            .all(|(component, pred_coll)| match component {
                Component::Normal(ref s) => s
                    .to_str()
                    .map(|ref s| s == &pred_coll.as_ref())
                    .unwrap_or(false),
                _ => false
            })
    }

    pub fn local_push<P: AsRef<Path>>(&mut self, path: P) {
        self.0.push(path)
    }

}

impl Display for StoreId {

    fn fmt(&self, fmt: &mut Formatter) -> RResult<(), FmtError> {
        write!(fmt, "{}", self.0.display())
    }

}

/// This Trait allows you to convert various representations to a single one
/// suitable for usage in the Store
pub trait IntoStoreId {
    fn into_storeid(self) -> Result<StoreId>;
}

impl IntoStoreId for StoreId {
    fn into_storeid(self) -> Result<StoreId> {
        Ok(self)
    }
}

impl IntoStoreId for PathBuf {
    fn into_storeid(self) -> Result<StoreId> {
        StoreId::new(self)
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct StoreIdWithBase<'a>(&'a PathBuf, PathBuf);

impl<'a> StoreIdWithBase<'a> {
    pub(crate) fn new(base: &'a PathBuf, path: PathBuf) -> Self {
        StoreIdWithBase(base, path)
    }

    pub(crate) fn without_base(self) -> StoreId {
        StoreId(self.1)
    }

    /// Transform the StoreId object into a PathBuf, error if the base of the StoreId is not
    /// specified.
    pub(crate) fn into_pathbuf(self) -> Result<PathBuf> {
        let mut base = self.0.clone();
        base.push(self.1);
        Ok(base)
    }

    /// Check whether the StoreId exists (as in whether the file exists)
    pub fn exists(&self) -> Result<bool> {
        self.clone().into_pathbuf().map(|pb| pb.exists())
    }

    pub fn to_str(&self) -> Result<String> {
        let mut base = self.0.clone();
        base.push(self.1.clone());
        Ok(base.display().to_string())
    }

    /// Try to create a StoreId object from a filesystem-absolute path.
    ///
    /// Automatically creates a StoreId object which has a `base` set to `store_part` if stripping
    /// the `store_part` from the `full_path` succeeded.
    pub(crate) fn from_full_path<D>(store_part: &'a PathBuf, full_path: D) -> Result<StoreIdWithBase<'a>>
        where D: Deref<Target = Path>
    {
        let p = full_path
            .strip_prefix(store_part)
            .map_err(Error::from)
            .context(err_msg("Error building Store Id from full path"))?;
        Ok(StoreIdWithBase(store_part, PathBuf::from(p)))
    }
}

impl<'a> IntoStoreId for StoreIdWithBase<'a> {
    fn into_storeid(self) -> Result<StoreId> {
        Ok(StoreId(self.1))
    }
}

impl<'a> Into<StoreId> for StoreIdWithBase<'a> {
    fn into(self) -> StoreId {
        StoreId(self.1)
    }
}

impl<'a> Display for StoreIdWithBase<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> RResult<(), FmtError> {
        write!(fmt, "{}/{}", self.0.display(), self.1.display())
    }
}


#[macro_export]
macro_rules! module_entry_path_mod {
    ($name:expr) => (
        #[deny(missing_docs,
                missing_copy_implementations,
                trivial_casts, trivial_numeric_casts,
                unstable_features,
                unused_import_braces, unused_qualifications,
                unused_imports)]
        /// A helper module to create valid module entry paths
        pub mod module_path {
            use std::convert::AsRef;
            use std::path::Path;
            use std::path::PathBuf;

            use $crate::storeid::StoreId;
            use failure::Fallible as Result;

            /// A Struct giving you the ability to choose store entries assigned
            /// to it.
            ///
            /// It is created through a call to `new`.
            pub struct ModuleEntryPath(PathBuf);

            impl ModuleEntryPath {
                /// Path has to be a valid UTF-8 string or this will panic!
                pub fn new<P: AsRef<Path>>(pa: P) -> ModuleEntryPath {
                    let mut path = PathBuf::new();
                    path.push(format!("{}", $name));
                    path.push(pa.as_ref().clone());
                    let name = pa.as_ref().file_name().unwrap()
                        .to_str().unwrap();
                    path.set_file_name(name);
                    ModuleEntryPath(path)
                }
            }

            impl $crate::storeid::IntoStoreId for ModuleEntryPath {
                fn into_storeid(self) -> Result<$crate::storeid::StoreId> {
                    StoreId::new(self.0)
                }
            }
        }
    )
}

pub struct StoreIdIterator {
    iter: Box<Iterator<Item = Result<StoreId>>>,
}

impl Debug for StoreIdIterator {

    fn fmt(&self, fmt: &mut Formatter) -> RResult<(), FmtError> {
        write!(fmt, "StoreIdIterator")
    }

}

impl StoreIdIterator {

    pub fn new(iter: Box<Iterator<Item = Result<StoreId>>>) -> StoreIdIterator {
        StoreIdIterator { iter }
    }

    pub fn with_store<'a>(self, store: &'a Store) -> StoreIdIteratorWithStore<'a> {
        StoreIdIteratorWithStore(self, store)
    }

}

impl Iterator for StoreIdIterator {
    type Item = Result<StoreId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }

}

pub struct StoreIdIteratorWithStore<'a>(StoreIdIterator, &'a Store);

impl<'a> Deref for StoreIdIteratorWithStore<'a> {
    type Target = StoreIdIterator;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Iterator for StoreIdIteratorWithStore<'a> {
    type Item = Result<StoreId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<'a> StoreIdIteratorWithStore<'a> {

    pub fn new(iter: Box<Iterator<Item = Result<StoreId>>>, store: &'a Store) -> Self {
        StoreIdIteratorWithStore(StoreIdIterator::new(iter), store)
    }

    pub fn without_store(self) -> StoreIdIterator {
        self.0
    }

    /// Transform the iterator into a StoreCreateIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_create_iter(self) -> StoreCreateIterator<'a> {
        StoreCreateIterator::new(Box::new(self.0), self.1)
    }

    /// Transform the iterator into a StoreDeleteIterator
    ///
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_delete_iter(self) -> StoreDeleteIterator<'a> {
        StoreDeleteIterator::new(Box::new(self.0), self.1)
    }

    /// Transform the iterator into a StoreGetIterator
    ///
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_get_iter(self) -> StoreGetIterator<'a> {
        StoreGetIterator::new(Box::new(self.0), self.1)
    }

    /// Transform the iterator into a StoreRetrieveIterator
    ///
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_retrieve_iter(self) -> StoreRetrieveIterator<'a> {
        StoreRetrieveIterator::new(Box::new(self.0), self.1)
    }

}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use storeid::StoreId;
    use storeid::StoreIdWithBase;
    use storeid::IntoStoreId;

    module_entry_path_mod!("test");

    #[test]
    fn test_correct_path() {
        let p = module_path::ModuleEntryPath::new("test");

        assert_eq!(p.into_storeid().unwrap().to_str().unwrap(), "test/test");
    }

    #[test]
    fn test_baseless_path() {
        let id = StoreId::new(PathBuf::from("test"));
        assert!(id.is_ok());
        assert_eq!(id.unwrap(), StoreId(PathBuf::from("test")));
    }

    #[test]
    fn test_base_path() {
        let id = StoreId::new(PathBuf::from("test"));
        assert!(id.is_ok());
        assert_eq!(id.unwrap(), StoreId(PathBuf::from("test")));
    }

    #[test]
    fn test_adding_base_to_baseless_path() {
        let id = StoreId::new(PathBuf::from("test"));

        assert!(id.is_ok());
        let id = id.unwrap();

        assert_eq!(id, StoreId(PathBuf::from("test")));

        let storebase = PathBuf::from("/tmp/");
        let id = id.with_base(&storebase);
        assert_eq!(id, StoreIdWithBase(&PathBuf::from("/tmp/"), PathBuf::from("test")));
    }

    #[test]
    fn test_removing_base_from_base_path() {
        let id = StoreId::new(PathBuf::from("/tmp/test"));

        assert!(id.is_ok());
        let storebase = PathBuf::from("/tmp/");
        let id = id.unwrap().with_base(&storebase);

        assert_eq!(id, StoreIdWithBase(&PathBuf::from("/tmp/"), PathBuf::from("test")));

        let id = id.without_base();
        assert_eq!(id, StoreId(PathBuf::from("test")));
    }

    #[test]
    fn test_basefull_into_pathbuf_is_ok() {
        let id = StoreId::new(PathBuf::from("/tmp/test"));
        assert!(id.is_ok());

        let storebase = PathBuf::from("/tmp/");
        let id = id.unwrap().with_base(&storebase);
        assert!(id.into_pathbuf().is_ok());
    }

    #[test]
    fn test_basefull_into_pathbuf_is_correct() {
        let id = StoreId::new(PathBuf::from("/tmp/test"));
        assert!(id.is_ok());

        let storebase = PathBuf::from("/tmp/");
        let pb = id.unwrap().with_base(&storebase).into_pathbuf();
        assert!(pb.is_ok());

        assert_eq!(pb.unwrap(), PathBuf::from("/tmp/test"));
    }

    #[test]
    fn storeid_in_collection() {
        let p = module_path::ModuleEntryPath::new("1/2/3/4/5/6/7/8/9/0").into_storeid().unwrap();

        assert!(p.is_in_collection(&["test", "1"]));
        assert!(p.is_in_collection(&["test", "1", "2"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7", "8"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7", "8", "9"]));
        assert!(p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0"]));

        assert!(!p.is_in_collection(&["test", "0", "2", "3", "4", "5", "6", "7", "8", "9", "0"]));
        assert!(!p.is_in_collection(&["test", "1", "2", "3", "4", "5", "6", "8"]));
        assert!(!p.is_in_collection(&["test", "1", "2", "3", "leet", "5", "6", "7"]));
    }

}

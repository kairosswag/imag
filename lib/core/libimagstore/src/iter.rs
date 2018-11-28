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

macro_rules! mk_iterator_mod {
    {
        modname   = $modname:ident,
        itername  = $itername:ident,
        iteryield = $yield:ty,
        extname   = $extname:ident,
        extfnname = $extfnname:ident,
        fun       = $fun:expr
    } => {
        pub mod $modname {
            use storeid::StoreId;
            #[allow(unused_imports)]
            use store::FileLockEntry;
            use store::Store;
            use failure::Fallible as Result;

            pub struct $itername<'a>(Box<Iterator<Item = Result<StoreId>> + 'a>, &'a Store);

            impl<'a> $itername<'a>
            {
                pub fn new(inner: Box<Iterator<Item = Result<StoreId>> + 'a>, store: &'a Store) -> Self {
                    $itername(inner, store)
                }
            }

            impl<'a> Iterator for $itername<'a>
            {
                type Item = Result<$yield>;

                fn next(&mut self) -> Option<Self::Item> {
                    self.0.next().map(|id| $fun(id?, self.1))
                }
            }

            pub trait $extname<'a> {
                fn $extfnname(self, store: &'a Store) -> $itername<'a>;
            }

            impl<'a, I> $extname<'a> for I
                where I: Iterator<Item = Result<StoreId>> + 'a
            {
                fn $extfnname(self, store: &'a Store) -> $itername<'a> {
                    $itername(Box::new(self), store)
                }
            }
        }
    }
}

mk_iterator_mod! {
    modname   = create,
    itername  = StoreCreateIterator,
    iteryield = FileLockEntry<'a>,
    extname   = StoreIdCreateIteratorExtension,
    extfnname = into_create_iter,
    fun       = |id: StoreId, store: &'a Store| store.create(id)
}

mk_iterator_mod! {
    modname   = delete,
    itername  = StoreDeleteIterator,
    iteryield = (),
    extname   = StoreIdDeleteIteratorExtension,
    extfnname = into_delete_iter,
    fun       = |id: StoreId, store: &'a Store| store.delete(id)
}

mk_iterator_mod! {
    modname   = get,
    itername  = StoreGetIterator,
    iteryield = Option<FileLockEntry<'a>>,
    extname   = StoreIdGetIteratorExtension,
    extfnname = into_get_iter,
    fun       = |id: StoreId, store: &'a Store| store.get(id)
}

mk_iterator_mod! {
    modname   = retrieve,
    itername  = StoreRetrieveIterator,
    iteryield = FileLockEntry<'a>,
    extname   = StoreIdRetrieveIteratorExtension,
    extfnname = into_retrieve_iter,
    fun       = |id: StoreId, store: &'a Store| store.retrieve(id)
}

#[cfg(test)]
#[allow(dead_code)]
mod compile_test {

    // This module contains code to check whether this actually compiles the way we would like it to
    // compile

    use store::Store;
    use storeid::StoreId;

    fn store() -> Store {
        unimplemented!("Not implemented because in compile-test")
    }

    fn test_compile_get() {
        let store = store();
        let _ = store
            .entries()
            .unwrap()
            .into_get_iter();
    }

    fn test_compile_get_result() {
        fn to_result(e: StoreId) -> Result<StoreId, ()> {
            Ok(e)
        }

        let store = store();
        let _ = store
            .entries()
            .unwrap()
            .into_get_iter();
    }
}

use storeid::StoreId;
use self::delete::StoreDeleteIterator;
use self::get::StoreGetIterator;
use self::retrieve::StoreRetrieveIterator;
use file_abstraction::iter::PathIterator;
use store::Store;
use failure::Fallible as Result;

/// Iterator for iterating over all (or a subset of all) entries
///
/// The iterator now has functionality to optimize the iteration, if only a subdirectory of the
/// store is required, for example `$STORE/foo`.
///
/// This is done via functionality where the underlying iterator gets
/// altered.
///
/// As the (for the filesystem backend underlying) `walkdir::WalkDir` type is not as nice as it
/// could be, iterating over two subdirectories with one iterator is not possible. Thus, iterators
/// for two collections in the store should be build like this (untested):
///
/// ```ignore
///     store
///         .entries()?
///         .in_collection("foo")
///         .chain(store.entries()?.in_collection("bar"))
/// ```
///
/// Functionality to exclude subdirectories is not possible with the current implementation and has
/// to be done during iteration, with filtering (as usual).
pub struct Entries<'a>(PathIterator<'a>, &'a Store);

impl<'a> Entries<'a> {

    pub(crate) fn new(pi: PathIterator<'a>, store: &'a Store) -> Self {
        Entries(pi, store)
    }

    pub fn in_collection(self, c: &str) -> Self {
        Entries(self.0.in_collection(c), self.1)
    }

    // TODO: Remove or fix me
    //pub fn without_store(self) -> StoreIdIterator {
    //    StoreIdIterator::new(Box::new(self.0.map(|r| r.map(|id| id.without_base()))))
    //}

    /// Transform the iterator into a StoreDeleteIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_delete_iter(self) -> StoreDeleteIterator<'a> {
        StoreDeleteIterator::new(Box::new(self.0.map(|r| r.map(|id| id.without_base()))), self.1)
    }

    /// Transform the iterator into a StoreGetIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_get_iter(self) -> StoreGetIterator<'a> {
        StoreGetIterator::new(Box::new(self.0.map(|r| r.map(|id| id.without_base()))), self.1)
    }

    /// Transform the iterator into a StoreRetrieveIterator
    ///
    /// This immitates the API from `libimagstore::iter`.
    pub fn into_retrieve_iter(self) -> StoreRetrieveIterator<'a> {
        StoreRetrieveIterator::new(Box::new(self.0.map(|r| r.map(|id| id.without_base()))), self.1)
    }

}

impl<'a> Iterator for Entries<'a> {
    type Item = Result<StoreId>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|r| r.map(|id| id.without_base()))
    }
}


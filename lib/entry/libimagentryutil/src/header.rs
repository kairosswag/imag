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

//! Utilities to access entry headers without boilerplate
//!
//! With this utility, a library can define a part of an imag entry header with a normal struct
//! which has to implement `Serialize` and `Deserialize` (which is trivial and can be derived
//! in most cases), and the user of the library can then use the provided interface to fetch a rust
//! object from a header.
//!
//!
//! # Example
//!
//! A library wants to store the following header:
//!
//! ```norun
//! [foo]
//! bar = <number>
//! content = <string>
//! ```
//!
//! It can therefore define the following type:
//!
//! ```
//! #[derive(Serialize, Deserialize, Debug)]
//! struct FooHeader {
//!     pub bar: usize,
//!     pub content: String,
//! }
//! ```
//!
//! and provide an accessor type:
//!
//! ```
//! impl HeaderPartial for FooHeader {
//!     const HEADER_LOCATION: &'static str = "foo";
//!     type Output = Self;
//! }
//! ```
//!
//! and then use the `HeaderAccessor` extension to access this data:
//!
//! ```norun
//! let foohdr = entry.get_header().read_partial::<FooHeaderHAS>().unwrap().unwrap(); // should handle errors
//! // foohdr.bar
//! // foohdr.content
//! ```
//!
//!
//! # Discussion
//!
//! The `toml_query` crate provides the functionality for reading and writing header partials. The
//! traits in this module only add a thin layer over the functionality from `toml_query` so that the
//! usage is more convenient.
//!
//! The value this module adds to the ecosystem is that the _path_ of the header partial can be
//! encoded (via the `HeaderPartial` trait` into the partial type itself.
//!
//! One might move this functionality into `toml_query` at some point in time.
//!
//!
//! # Additional details
//!
//! By using this library, type-check boilerplate code in the header-processing of the libraries
//! can be removed.
//!
//! Using zero-sized types for the `HeaderPartial` should result in very little runtime
//! overhead.
//!
//! Of course it is also possible to only partially serialize headers (for example "sub-headers" or
//! only single fields, by specifying `HeaderPartial::Output = String` and
//! `HeaderPartial::HEADER_LOCATION = "foo.content"`.
//!

use std::fmt::Debug;

use failure::Error;
use failure::Fallible as Result;
use serde::{Serialize, Deserialize};
use toml::Value;
use toml_query::read::TomlValueReadExt;

/// Describes a _part_ of a header
///
pub trait HeaderPartial<'a> {
    // The location ("section") of the header where to find the struct
    const HEADER_LOCATION: &'static str;

    // The type which represents the data
    type Output: Serialize + Deserialize<'a> + Debug;
}

pub trait HeaderPartialAccessor {

    fn read_partial<'a, HAS: HeaderPartial<'a>>(&self) -> Result<Option<HAS::Output>>;

}

impl HeaderPartialAccessor for Value {

    fn read_partial<'a, HAS: HeaderPartial<'a>>(&self) -> Result<Option<HAS::Output>> {
        trace!("Reading header of {:?} at '{}'", self, HAS::HEADER_LOCATION);
        self.read_deserialized::<HAS::Output>(HAS::HEADER_LOCATION).map_err(Error::from)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    use std::path::PathBuf;
    use std::sync::Arc;
    use std::collections::BTreeMap;

    use toml::Value;
    use toml_query::insert::TomlValueInsertExt;

    use libimagstore::store::Store;

    #[derive(Debug, Deserialize, Serialize)]
    struct TestHeader {
        pub value: String,
    }

    impl<'a> HeaderPartial<'a> for TestHeader {
        const HEADER_LOCATION: &'static str = "foo";
        type Output                         = Self;
    }

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    pub fn get_store() -> Store {
        use libimagstore::file_abstraction::InMemoryFileAbstraction;
        let backend = Arc::new(InMemoryFileAbstraction::default());
        Store::new_with_backend(PathBuf::from("/"), &None, backend).unwrap()
    }

    #[test]
    fn test_compiles() {
        setup_logging();
        let store     = get_store();
        let id        = PathBuf::from("test_compiles");
        let mut entry = store.retrieve(id).unwrap();
        {
            let mut tbl = BTreeMap::new();
            tbl.insert(String::from("value"), Value::String(String::from("foobar")));
            let tbl = Value::Table(tbl);
            entry.get_header_mut().insert(TestHeader::HEADER_LOCATION, tbl).unwrap();
        }

        let header : TestHeader = entry.get_header().read_partial::<TestHeader>().unwrap().unwrap();
        assert_eq!(header.value, "foobar");
    }
}


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
//! which has to implemenet `Serialize` and `Deserialize` (which is trivial and can be derived
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
//! struct FooHeaderHAS;
//! impl HeaderAccessorSpec for FooHeaderHAS {
//!     const HEADER_LOCATION: &'static str = "foo";
//!     type Output = FooHeader;
//! }
//! ```
//!
//! and then use the `HeaderAccessor` extension to access this data:
//!
//! ```norun
//! let foohdr = entry.header_read::<FooHeaderHAS>().unwrap().unwrap(); // should handle errors
//! // foohdr.bar
//! // foohdr.content
//! ```
//!
//!
//! # Discussion
//!
//! It might be a good idea to implement `HeaderAccessorSpec` directly on the header type itself:
//!
//! ```norun
//! impl HeaderAccessorSpec for FooHeader {
//!     const HEADER_LOCATION: &'static str = "foo";
//!     type Output = FooHeader;
//! }
//! ```
//!
//! Which would result in the following calling code:
//!
//! ```norun
//! let foohdr = entry.header_read::<FooHeader>().unwrap().unwrap(); // should handle errors
//! ```
//!
//! which automatically describes the resulting type in the calling code. Because of the zero-sized
//! nature of the API, this should be possible.
//!
//! It is not yet tested yet, though.
//!
//!
//! # Additional details
//!
//! By using this library, type-check boilerplate code in the header-processing of the libraries
//! can be removed.
//!
//! Using zero-sized types for the `HeaderAccessorSpec` should result in very little runtime
//! overhead.
//!
//! Of course it is also possible to only partially serialize headers (for example "sub-headers" or
//! only single fields, by specifying `HeaderAccessorSpec::Output = String` and
//! `HeaderAccessorSpec::HEADER_LOCATION = "foo.content"`.
//!

use std::ops::Debug;

/// Describes a _part_ of a headerdeser
///
pub trait HeaderAccessorSpec {
    // The location ("section") of the header where to find the struct
    const HEADER_LOCATION: &'static str;

    // The type which represents the data
    type Output: Serialize + Deserialize + Debug;
}

pub trait HeaderAccessor {

    fn read<HAS: HeaderAccessorSpec>(&self) -> Result<Option<HAS::Output>>;

}

impl HeaderAccessor for Entry {

    fn header_read<HAS: HeaderAccessorSpec>(&self) -> Result<Option<HAS::Output>> {
        trace!("Reading header of {:?} at '{}'", self, HAS::HEADER_LOCATION);

        self.get_header()
            .read_deserialized::<HAS::Output>(HAS::HEADER_LOCATION)
            .map_err(Error::from)
    }

}


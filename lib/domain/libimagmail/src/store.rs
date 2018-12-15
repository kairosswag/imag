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

use failure::Error;
use failure::Fallible as Result;
use failure::ResultExt;
use failure::err_msg;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;

pub trait MailStore<'a> {
    fn get_mail_from_path<P: AsRef<Path>>(&'a self, p: P)      -> Result<Option<FileLockEntry<'a>>>;
    fn retrieve_mail_from_path<P: AsRef<Path>>(&'a self, p: P) -> Result<FileLockEntry<'a>>;
    fn get_mail(&'a self, mid: MessageId)                      -> Result<Option<FileLockEntry<'a>>>;
    fn all_mails(&'a self)                                     -> Result<StoreIdIterator>;
}

impl<'a> MailStore<'a> for Store {
    fn get_mail_from_path<P: AsRef<Path>>(&'a self, p: P) -> Result<Option<FileLockEntry<'a>>> {
        unimplemented!()
    }

    fn retrieve_mail_from_path<P: AsRef<Path>>(&'a self, p: P) -> Result<FileLockEntry<'a>> {
        unimplemented!()
    }

    fn get_mail(&'a self, mid: MessageId) -> Result<Option<FileLockEntry<'a>>> {
        unimplemented!()
    }

    fn all_mails(&'a self) -> Result<StoreIdIterator> {
        unimplemented!()
    }
}

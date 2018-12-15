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
use std::fs::File;
use std::io::Read;
use std::fs::OpenOptions;

use libimagstore::store::Store;
use libimagstore::storeid::StoreId;
use libimagstore::store::FileLockEntry;
use libimagentryref::reference::Ref;
use libimagerror::errors::ErrorMsg as EM;

use email::MimeMessage;
use email::results::ParsingResult as EmailParsingResult;

use failure::Fallible as Result;
use failure::ResultExt;
use failure::Error;
use failure::err_msg;

pub trait Mail {
    fn mail_header(&self)            -> Result<MailHeader>;
    fn get_field(&self, field: &str) -> Result<Option<String>>;
    fn get_from(&self)               -> Result<Option<String>>;
    fn get_to(&self)                 -> Result<Option<String>>;
    fn get_subject(&self)            -> Result<Option<String>>;
    fn get_message_id(&self)         -> Result<Option<String>>;
    fn get_in_reply_to(&self)        -> Result<Option<String>>;
}

impl Mail for Entry {

    /// Get a complete map of the header of that mailheader
    ///
    /// Much more performant than `Mail::get_field()` because it does not open-close-open-close the
    /// mail file.
    fn mail_header(&self) -> Result<MailHeader> {
        unimplemented!()
    }

    /// Get a value of a single field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_field(&self, field: &str) -> Result<Option<String>> {
        debug!("Getting field in mail: {:?}", field);
        let mail_file_location = self.get_header()
            .read("mail.file")?
            .ok_or_else(|| unimplemented!() /* missing header field */)?
            .as_str()
            .ok_or_else(|| unimplemented!() /* wrong header type */)?;

        unimplemented!()
        /*
         * Read the mail file
         * parse it
         * find the field
         * return the field
         */
    }

    /// Get a value of the `From` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_from(&self) -> Result<Option<String>> {
        self.get_field("From")
    }

    /// Get a value of the `To` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_to(&self) -> Result<Option<String>> {
        self.get_field("To")
    }

    /// Get a value of the `Subject` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_subject(&self) -> Result<Option<String>> {
        self.get_field("Subject")
    }

    /// Get a value of the `Message-ID` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_message_id(&self) -> Result<Option<String>> {
        self.get_field("Message-ID")
    }

    /// Get a value of the `In-Reply-To` field of the mail file
    ///
    /// # Note
    ///
    /// Use `Mail::mail_header()` if you need to read more than one field.
    fn get_in_reply_to(&self) -> Result<Option<String>> {
        self.get_field("In-Reply-To")
    }

}

#[derive(Debug)]
pub struct MailHeader(BTreeMap<String, String>);


impl Deref for MailHeader {
    type Target = BTreeMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl MailHeader {
    /// Get a value of a single field of the mail file
    fn get_field(&self, field: &str) -> Result<Option<String>> {
        unimplemented!()
    }

    /// Get a value of the `From` field of the mail file
    fn get_from(&self) -> Result<Option<String>> {
        self.get_field("From")
    }

    /// Get a value of the `To` field of the mail file
    fn get_to(&self) -> Result<Option<String>> {
        self.get_field("To")
    }

    /// Get a value of the `Subject` field of the mail file
    fn get_subject(&self) -> Result<Option<String>> {
        self.get_field("Subject")
    }

    /// Get a value of the `Message-ID` field of the mail file
    fn get_message_id(&self) -> Result<Option<String>> {
        self.get_field("Message-ID")
    }

    /// Get a value of the `In-Reply-To` field of the mail file
    fn get_in_reply_to(&self) -> Result<Option<String>> {
        self.get_field("In-Reply-To")
    }
}

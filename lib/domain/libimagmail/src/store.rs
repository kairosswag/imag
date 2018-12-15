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
use std::fs::OpenOptions;
use std::io::Read;
use std::fmt::Debug;

use failure::Error;
use failure::Fallible as Result;
use failure::ResultExt;
use toml::Value;
use toml_query::insert::TomlValueInsertExt;
use email::MimeMessage;

use libimagstore::store::FileLockEntry;
use libimagstore::store::Store;
use libimagstore::storeid::StoreIdIterator;
use libimagentryref::reference::Config;
use libimagentryref::reference::Ref;

use module_path::ModuleEntryPath;
use mid::MessageId;

pub trait MailStore<'a> {
    fn get_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<Option<FileLockEntry<'a>>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug;

    fn retrieve_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<FileLockEntry<'a>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug;

    fn get_mail(&'a self, mid: MessageId)                      -> Result<Option<FileLockEntry<'a>>>;
    fn all_mails(&'a self)                                     -> Result<StoreIdIterator>;
}

impl<'a> MailStore<'a> for Store {
    fn get_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<Option<FileLockEntry<'a>>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug
    {
        let message_id = get_message_id_for_mailfile(p)?;
        let new_sid    = ModuleEntryPath::new(message_id.clone()).into_storeid()?;

        match self.get(new_sid)? {
            Some(mut entry) => {
                if !entry.is_ref()? {
                    unimplemented!()
                    // TODO: FLE is not a ref.. something went wrong ...
                    // error out here
                }
                let _ = entry.get_header_mut().insert("mail.message-id", Value::String(message_id))?;
                Ok(Some(entry))
            },
            None => Ok(None),
        }
    }

    fn retrieve_mail_from_path<P, CollName>(&'a self, p: P, collection_name: CollName, config: &Config)
        -> Result<FileLockEntry<'a>>
        where P: AsRef<Path> + Debug,
              CollName: AsRef<str> + Debug
    {
        let message_id = get_message_id_for_mailfile(&p)?;
        let new_sid    = ModuleEntryPath::new(message_id.clone()).into_storeid()?;
        let mut entry  = self.retrieve(new_sid)?;
        let _ = entry.get_header_mut().insert("mail.message-id", Value::String(message_id))?;
        let _ = entry.make_ref(p, collection_name, config, false)?;

        Ok(entry)
    }

    fn get_mail(&'a self, mid: MessageId) -> Result<Option<FileLockEntry<'a>>> {
        unimplemented!()
    }

    fn all_mails(&'a self) -> Result<StoreIdIterator> {
        unimplemented!()
    }
}

fn get_message_id_for_mailfile<P: AsRef<Path> + Debug>(p: P) -> Result<String> {
    let mut s = String::new();
    let _     = OpenOptions::new()
        .read(true)
        .write(false)
        .create(false)
        .open(p)?
        .read_to_string(&mut s)?;

    MimeMessage::parse(&s)
        .context(format_err!("Cannot parse Email {:?}", p))?
        .headers
        .get(String::from("Message-Id"))
        .ok_or_else(format_err!("Message has no 'Message-Id': {:?}", p))?
        .get_value::<String>()
        .context(format_err!("Cannot decode header value in 'Message-Id': {:?}", p))
        .map_err(Error::from)
}

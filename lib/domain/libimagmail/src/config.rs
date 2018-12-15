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

/// A struct representing a full mail configuration, required for working with this library
///
/// For convenience reasons, this implements Serialize and Deserialize, so it can be fetched from a
/// configuration file for example
///
/// # TODO
///
/// Figure out how to use handlebars with variables on this. Right now the support for that is not
/// implemented yet.
///
#[derive(Serialize, Deserialize, Debug)]
pub struct MailConfig {
    default_account  : String,
    accounts         : Vec<MailAccountConfig>,
    fetchcommand     : String,
    postfetchcommand : Option<String>,
    sendcommand      : String,
    postsendcommand  : Option<String>,
}

impl MailConfig {
    pub fn default_account(&self) -> &String {
        &self.default_account
    }

    pub fn accounts(&self) -> &Vec<MailAccountConfig> {
        &self.accounts
    }

    pub fn fetchcommand -> &String {
        &self.fetchcommand
    }

    pub fn postfetchcommand -> Option<&String> {
        &self.postfetchcommand
    }

    pub fn sendcommand -> &String {
        &self.sendcommand
    }

    pub fn postsendcommand -> Option<&String> {
        &self.postsendcommand
    }
}

/// A configuration for a single mail accounts
///
/// If one of the keys `fetchcommand`, `postfetchcommand`, `sendcommand` or `postsendcommand` is
/// not available, the implementation of the `MailConfig` will automatically use the global
/// configuration if applicable.
#[derive(Serialize, Deserialize, Debug)]
pub struct MailAccountConfig {
    pub name             : String,
    pub outgoingbox      : PathBuf,
    pub draftbox         : PathBuf,
    pub sentbox          : PathBuf,
    pub maildirroot      : PathBuf,
    pub fetchcommand     : Option<String>,
    pub postfetchcommand : Option<String>,
    pub sendcommand      : Option<String>,
    pub postsendcommand  : Option<String>,
}


//
// imag - the personal information management suite for the commandline
// Copyright (C) 2015, 2016 Matthias Beyer <mail@beyermatthias.de> and contributors
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

use clap::{Arg, App, SubCommand};

use libimagutil::cli_validators::*;

pub fn build_ui<'a>(app: App<'a, 'a>) -> App<'a, 'a> {
    app
        .subcommand(SubCommand::with_name("list")
                   .about("List contacts")
                   .version("0.1")
                   .arg(Arg::with_name("filter")
                        .index(1)
                        .takes_value(true)
                        .required(false)
                        .multiple(true)
                        .value_name("FILTER")
                        .help("Filter by these properties (not implemented yet)"))
                   )

        .subcommand(SubCommand::with_name("import")
                   .about("Import contacts")
                   .version("0.1")
                   .arg(Arg::with_name("path")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(false)
                        .value_name("PATH")
                        .help("Import from this file/directory"))
                   )

        .subcommand(SubCommand::with_name("show")
                   .about("Show contact")
                   .version("0.1")
                   .arg(Arg::with_name("ref")
                        .index(1)
                        .takes_value(true)
                        .required(true)
                        .multiple(false)
                        .value_name("REF")
                        .help("Show the contact pointed to by this reference hash"))
                   )
}

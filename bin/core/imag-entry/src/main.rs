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

#![deny(
    non_camel_case_types,
    non_snake_case,
    path_statements,
    trivial_numeric_casts,
    unstable_features,
    unused_allocation,
    unused_import_braces,
    unused_imports,
    unused_must_use,
    unused_mut,
    unused_qualifications,
    while_true,
)]

extern crate clap;
#[macro_use] extern crate log;
extern crate toml;
extern crate toml_query;
extern crate filters;

extern crate libimagentryedit;
extern crate libimagerror;
#[macro_use] extern crate libimagrt;
extern crate libimagstore;
extern crate libimagutil;

use std::path::PathBuf;
use std::io::Read;

use libimagrt::setup::generate_runtime_setup;
use libimagerror::trace::MapErrTrace;
use libimagstore::storeid::IntoStoreId;
use libimagstore::storeid::StoreIdIterator;
use libimagstore::iter::get::StoreIdGetIteratorExtension;
use libimagerror::iter::TraceIterator;

mod ui;
mod header;
mod content;
mod exec;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-entry",
                                    &version,
                                    "Plumbing tool for reading/writing structured data in entries",
                                    ui::build_ui);

    let list_output_with_ids = rt.cli().is_present("list-id");
    let list_output_with_ids_fmt = rt.cli().value_of("list-id-format");

    let sids = match rt.cli().value_of("entry") {
        Some(path) => vec![PathBuf::from(path).into_storeid().map_err_trace_exit_unwrap(1)],
        None => if rt.cli().is_present("entries-from-stdin") {
            let stdin = rt.stdin().unwrap_or_else(|| {
                error!("Cannot get handle to stdin");
                ::std::process::exit(1)
            });

            let mut buf = String::new();
            let _ = stdin.lock().read_to_string(&mut buf).unwrap_or_else(|_| {
                error!("Failed to read from stdin");
                ::std::process::exit(1)
            });

            buf.lines()
                .map(PathBuf::from)
                .map(|p| p.into_storeid().map_err_trace_exit_unwrap(1))
                .collect()
        } else {
            error!("Something weird happened. I was not able to find the path of the entries to edit");
            ::std::process::exit(1)
        }
    };

    let iter = StoreIdIterator::new(Box::new(sids.into_iter().map(Ok)))
        .into_get_iter(rt.store())
        .trace_unwrap_exit(1)
        .filter_map(|x| x);

    rt.cli()
        .subcommand_name()
        .map(|name| {
            match name {
                "header"  => header::process_headers(&rt, iter),
                "content" => content::process_content(&rt, iter),
                "exec"    => exec::process_exec(&rt, iter),
                other     => {
                    debug!("Unknown command");
                    let _ = rt.handle_unknown_subcommand("imag-category", other, rt.cli())
                        .map_err_trace_exit_unwrap(1)
                        .code()
                        .map(::std::process::exit);
                },
            }
        });
}


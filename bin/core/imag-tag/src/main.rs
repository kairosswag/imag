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

#![forbid(unsafe_code)]

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

#[cfg(test)] extern crate toml;
#[cfg(test)] extern crate failure;

extern crate libimagstore;
#[macro_use] extern crate libimagrt;
extern crate libimagentrytag;
extern crate libimagerror;

#[cfg(test)]
#[macro_use]
extern crate libimagutil;

#[cfg(not(test))]
extern crate libimagutil;

#[cfg(test)]
extern crate toml_query;

#[cfg(test)]
extern crate env_logger;

use std::io::Write;

use libimagrt::runtime::Runtime;
use libimagrt::setup::generate_runtime_setup;
use libimagentrytag::tagable::Tagable;
use libimagentrytag::tag::Tag;
use libimagerror::trace::trace_error;
use libimagerror::trace::MapErrTrace;
use libimagerror::io::ToExitCode;
use libimagerror::exit::ExitUnwrap;
use libimagstore::storeid::StoreId;
use libimagutil::warn_exit::warn_exit;

use clap::ArgMatches;

mod ui;

use ui::build_ui;

fn main() {
    let version = make_imag_version!();
    let rt = generate_runtime_setup("imag-tag",
                                    &version,
                                    "Direct interface to the store. Use with great care!",
                                    build_ui);

    let ids = rt.ids::<::ui::PathProvider>().map_err_trace_exit_unwrap(1);

    rt.cli()
        .subcommand_name()
        .map(|name| match name {
            "list" => for id in ids {
                list(id, &rt)
            },
            "remove" => for id in ids {
                let add = None;
                let rem = get_remove_tags(rt.cli());
                debug!("id = {:?}, add = {:?}, rem = {:?}", id, add, rem);
                alter(&rt, id, add, rem);
            },
            "add" => for id in ids {
                let add = get_add_tags(rt.cli());
                let rem = None;
                debug!("id = {:?}, add = {:?}, rem = {:?}", id, add, rem);
                alter(&rt, id, add, rem);
            },
            other => {
                debug!("Unknown command");
                let _ = rt.handle_unknown_subcommand("imag-tag", other, rt.cli())
                    .map_err_trace_exit_unwrap(1)
                    .code()
                    .map(::std::process::exit);
            },
        });
}

fn alter(rt: &Runtime, path: StoreId, add: Option<Vec<Tag>>, rem: Option<Vec<Tag>>) {
    match rt.store().get(path.clone()) {
        Ok(Some(mut e)) => {
            debug!("Entry header now = {:?}", e.get_header());

            add.map(|tags| {
                    debug!("Adding tags = '{:?}'", tags);
                    for tag in tags {
                        debug!("Adding tag '{:?}'", tag);
                        if let Err(e) = e.add_tag(tag) {
                            trace_error(&e);
                        } else {
                            debug!("Adding tag worked");
                        }
                    }
                }); // it is okay to ignore a None here

            debug!("Entry header now = {:?}", e.get_header());

            rem.map(|tags| {
                debug!("Removing tags = '{:?}'", tags);
                for tag in tags {
                    debug!("Removing tag '{:?}'", tag);
                    if let Err(e) = e.remove_tag(tag) {
                        trace_error(&e);
                    }
                }
            }); // it is okay to ignore a None here

            debug!("Entry header now = {:?}", e.get_header());

        },

        Ok(None) => {
            info!("No entry found.");
        },

        Err(e) => {
            info!("No entry.");
            trace_error(&e);
        },
    }

    let _ = rt
        .report_touched(&path)
        .map_err_trace_exit_unwrap(1);
}

fn list(path: StoreId, rt: &Runtime) {
    let entry = match rt.store().get(path.clone()).map_err_trace_exit_unwrap(1) {
        Some(e) => e,
        None => warn_exit("No entry found.", 1),
    };

    let scmd = rt.cli().subcommand_matches("list").unwrap(); // safe, we checked in main()

    let json_out = scmd.is_present("json");
    let line_out = scmd.is_present("linewise");
    let sepp_out = scmd.is_present("sep");
    let mut comm_out = scmd.is_present("commasep");

    if !vec![json_out, line_out, comm_out, sepp_out].iter().any(|v| *v) {
        // None of the flags passed, go to default
        comm_out = true;
    }

    let tags = entry.get_tags().map_err_trace_exit_unwrap(1);

    if json_out {
        unimplemented!()
    }

    if line_out {
        for tag in &tags {
            let _ = writeln!(rt.stdout(), "{}", tag)
                .to_exit_code()
                .unwrap_or_exit();
        }
    }

    if sepp_out {
        let sepp = scmd.value_of("sep").unwrap(); // we checked before
        let _ = writeln!(rt.stdout(), "{}", tags.join(sepp))
            .to_exit_code()
            .unwrap_or_exit();
    }

    if comm_out {
        let _ = writeln!(rt.stdout(), "{}", tags.join(", "))
            .to_exit_code()
            .unwrap_or_exit();
    }

    let _ = rt
        .report_touched(&path)
        .map_err_trace_exit_unwrap(1);
}

/// Get the tags which should be added from the commandline
///
/// Returns none if the argument was not specified
fn get_add_tags(matches: &ArgMatches) -> Option<Vec<Tag>> {
    retrieve_tags(matches, "add", "add-tags")
}

/// Get the tags which should be removed from the commandline
///
/// Returns none if the argument was not specified
fn get_remove_tags(matches: &ArgMatches) -> Option<Vec<Tag>> {
    retrieve_tags(matches, "remove", "remove-tags")
}

fn retrieve_tags(m: &ArgMatches, s: &'static str, v: &'static str) -> Option<Vec<Tag>> {
    Some(m
         .subcommand_matches(s)
         .unwrap_or_else(|| {
             error!("Expected subcommand '{}', but was not specified", s);
             ::std::process::exit(1)
         })
         .values_of(v)
         .unwrap() // enforced by clap
         .into_iter()
         .map(String::from)
         .collect())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::ffi::OsStr;

    use toml::value::Value;
    use toml_query::read::TomlValueReadExt;
    use failure::Fallible as Result;
    use failure::Error;

    use libimagrt::runtime::Runtime;
    use libimagstore::storeid::StoreId;
    use libimagstore::store::{FileLockEntry, Entry};

    use super::*;

    make_mock_app! {
        app "imag-tag";
        modulename mock;
        version env!("CARGO_PKG_VERSION");
        with help "imag-tag mocking app";
    }
    use self::mock::generate_test_runtime;

    fn create_test_default_entry<'a, S: AsRef<OsStr>>(rt: &'a Runtime, name: S) -> Result<StoreId> {
        let mut path = PathBuf::new();
        path.set_file_name(name);

        let default_entry = Entry::new(StoreId::new(PathBuf::from("")).unwrap())
            .to_str()
            .unwrap();

        let id = StoreId::new(path)?;
        let mut entry = rt.store().create(id.clone())?;
        entry.get_content_mut().push_str(&default_entry);

        Ok(id)
    }

    fn get_entry_tags<'a>(entry: &'a FileLockEntry<'a>) -> Result<Option<&'a Value>> {
        entry.get_header().read(&"tag.values".to_owned()).map_err(Error::from)
    }

    fn tags_toml_value<'a, I: IntoIterator<Item = &'static str>>(tags: I) -> Value {
        Value::Array(tags.into_iter().map(|s| Value::String(s.to_owned())).collect())
    }

    fn setup_logging() {
        let _ = ::env_logger::try_init();
    }

    #[test]
    fn test_tag_add_adds_tag() {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-add-adds-tags";
        let rt = generate_test_runtime(vec![name, "add", "foo"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        debug!("Getting 'add' tags");
        let add = get_add_tags(rt.cli());
        debug!("Add-tags: {:?}", add);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, None);
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();

        let test_tags  = get_entry_tags(&test_entry);
        assert!(test_tags.is_ok(), "Should be Ok(_) = {:?}", test_tags);

        let test_tags  = test_tags.unwrap();
        assert!(test_tags.is_some(), "Should be Some(_) = {:?}", test_tags);
        let test_tags  = test_tags.unwrap();

        assert_ne!(*test_tags, tags_toml_value(vec![]));
        assert_eq!(*test_tags, tags_toml_value(vec!["foo"]));
    }

    #[test]
    fn test_tag_remove_removes_tag() {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-remove-removes-tag";
        let rt = generate_test_runtime(vec![name, "remove", "foo"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        // Manually add tags
        let add = Some(vec![ "foo".to_owned() ]);

        debug!("Getting 'remove' tags");
        let rem = get_remove_tags(rt.cli());
        debug!("Rem-tags: {:?}", rem);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, rem);
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();
        let test_tags  = get_entry_tags(&test_entry).unwrap().unwrap();

        assert_eq!(*test_tags, tags_toml_value(vec![]));
    }

    #[test]
    fn test_tag_remove_removes_only_to_remove_tag() {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-remove-removes-only-to-remove-tag-doesnt-crash-on-nonexistent-tag";
        let rt = generate_test_runtime(vec![name, "remove", "foo"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        // Manually add tags
        let add = Some(vec![ "foo".to_owned(), "bar".to_owned() ]);

        debug!("Getting 'remove' tags");
        let rem = get_remove_tags(rt.cli());
        debug!("Rem-tags: {:?}", rem);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, rem);
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();
        let test_tags  = get_entry_tags(&test_entry).unwrap().unwrap();

        assert_eq!(*test_tags, tags_toml_value(vec!["bar"]));
    }

    #[test]
    fn test_tag_remove_removes_but_doesnt_crash_on_nonexistent_tag() {
        setup_logging();
        debug!("Generating runtime");
        let name = "test-tag-remove-removes-but-doesnt-crash-on-nonexistent-tag";
        let rt = generate_test_runtime(vec![name, "remove", "foo", "bar"]).unwrap();

        debug!("Creating default entry");
        create_test_default_entry(&rt, name).unwrap();
        let id = PathBuf::from(String::from(name));

        // Manually add tags
        let add = Some(vec![ "foo".to_owned() ]);

        debug!("Getting 'remove' tags");
        let rem = get_remove_tags(rt.cli());
        debug!("Rem-tags: {:?}", rem);

        debug!("Altering things");
        alter(&rt, StoreId::new(id.clone()).unwrap(), add, rem);
        debug!("Altered");

        let test_entry = rt.store().get(id).unwrap().unwrap();
        let test_tags  = get_entry_tags(&test_entry).unwrap().unwrap();

        assert_eq!(*test_tags, tags_toml_value(vec![]));
    }

}


#![allow(warnings)]
//! Library for parsing and building metainfo files.
//!
//! # Examples
//!
//! Building and parsing a metainfo file from a directory:
//!
//! ```rust
//!     extern crate bip_metainfo;
//!
//!     use bip_metainfo::{MetainfoBuilder, Metainfo};
//!
//!     fn main() {
//!         let builder = MetainfoBuilder::new()
//!             .set_created_by(Some("bip_metainfo example"))
//!             .set_comment(Some("Metainfo File From A File"));
//!
//!         // Build the file from the crate's src folder
//!         let bytes = builder.build(1, "src", |progress| {
//!             // Progress Is A Value Between 0.0 And 1.0
//!             assert!(progress <= 1.0f64);
//!         }).unwrap();
//!         let file = Metainfo::from_bytes(&bytes).unwrap();
//!
//!         assert_eq!(file.info().directory(), Some("src".as_ref()));
//!     }
//! ```
//!
//! Building and parsing a metainfo file from direct data:
//!
//! ```rust
//!     extern crate bip_metainfo;
//!
//!     use bip_metainfo::{MetainfoBuilder, Metainfo, DirectAccessor};
//!
//!     fn main() {
//!         let builder = MetainfoBuilder::new()
//!             .set_created_by(Some("bip_metainfo example"))
//!             .set_comment(Some("Metainfo File From A File"));
//!
//!         let file_name = "FileName.txt";
//!         let file_data = b"This is our file data, it is already in memory!!!";
//!         let accessor = DirectAccessor::new(file_name, file_data);
//!
//!         // Build the file from some data that is already in memory
//!         let bytes = builder.build(1, accessor, |progress| {
//!             // Progress Is A Value Between 0.0 And 1.0
//!             assert!(progress <= 1.0f64);
//!         }).unwrap();
//!         let file = Metainfo::from_bytes(&bytes).unwrap();
//!
//!         assert_eq!(file.info().directory(), None);
//!         assert_eq!(file.info().files().count(), 1);
//!
//!         let single_file = file.info().files().next().unwrap();
//!         assert_eq!(single_file.length() as usize, file_data.len());
//!         assert_eq!(single_file.path().iter().count(), 1);
//!         assert_eq!(single_file.path().to_str().unwrap(), file_name);
//!     }
//! ```

#[macro_use]
extern crate bip_bencode;
extern crate bip_util;
extern crate walkdir;
#[macro_use]
extern crate error_chain;

#[cfg(test)]
extern crate rand;

mod accessor;
mod builder;
pub mod error;
mod metainfo;
mod parse;

pub mod iter;

pub use bip_util::bt::InfoHash;

pub use accessor::{Accessor, IntoAccessor, DirectAccessor, FileAccessor, PieceAccess};
pub use builder::{MetainfoBuilder, PieceLength, InfoBuilder};
pub use metainfo::{Info, Metainfo, File};

//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! AnimPacks are a container-type file, that usually contains anim-related files, such as Anim Tables,
//! Anim Fragments and Matched Combat Tables.
//!
//! It's usually found in the `anim` folder of the game, under the extension `.animpack`, hence their name.
//!
//! # AnimPack Structure
//!
//! | Bytes | Type | Data |
//! | ----- | ---- | ---- |
//! | 4     | [u32] | File Count. |
//! | X * File Count | [File](#file-structure) List | List of files inside the AnimPack File. |
//!
//!
//! # File Structure
//!
//! | Bytes | Type | Data |
//! | ----- | ---- | ---- |
//! | *     | StringU8 | File Path. |
//! | 4     | [u32]  | File Length in bytes. |
//! | File Lenght | &\[[u8]\] | File Data. |

use std::collections::HashMap;

use crate::error::Result;
use crate::{binary::{ReadBytes, WriteBytes}, schema::Schema};
use crate::files::*;

/// Extension used by AnimPacks.
pub const EXTENSION: &str = ".animpack";

//---------------------------------------------------------------------------//
//                              Enum & Structs
//---------------------------------------------------------------------------//

/// This holds an entire AnimPack PackedFile decoded in memory.
#[derive(PartialEq, Clone, Debug, Default)]
pub struct AnimPack {
    files: HashMap<String, RFile>,
}

//---------------------------------------------------------------------------//
//                           Implementation of AnimPack
//---------------------------------------------------------------------------//

/// Implementation of `AnimPack`.
impl AnimPack {
/*

    pub fn get_as_pack_file_info(&self, path: &[String]) -> (PackFileInfo, Vec<PackedFileInfo>) {
        let pack_file_info = PackFileInfo {
            file_name: path.last().unwrap().to_owned(),
            ..Default::default()
        };

        let packed_file_info = self.files.iter().map(From::from).collect();
        (pack_file_info, packed_file_info)
    }

    /// This function returns a reference of the paths of all the `PackedFiles` in the provided `PackFile` under the provided path.
    pub fn get_ref_packed_files_paths_by_path_start(&self, path: &[String]) -> Vec<&[String]> {
        self.files.par_iter().map(|x| x.get_ref_path()).filter(|x| x.starts_with(path) && !path.is_empty() && x.len() > path.len()).collect()
    }

    pub fn get_file_paths_from_path_types(&self, path_types: &[PathType]) -> Vec<Vec<String>> {

        // Keep the PathTypes added so we can return them to the UI easily.
        let path_types = PathType::dedup(path_types);

        // As this can get very slow very quickly, we do here some... optimizations.
        // First, we get if there are PackFiles or folders in our list of PathTypes.
        let we_have_packfile = path_types.par_iter().any(|item| {
            matches!(item, PathType::PackFile)
        });

        let we_have_folder = path_types.par_iter().any(|item| {
            matches!(item, PathType::Folder(_))
        });

        // Then, if we have a PackFile,... just import all PackedFiles.
        if we_have_packfile {
            self.get_anim_packed_paths_all()
        }

        // If we only have files, get all the files we have at once, then add them all together.
        else if !we_have_folder {
            path_types.par_iter().filter_map(|x| {
                if let PathType::File(path) = x { Some(path.to_vec()) } else { None }
            }).collect::<Vec<Vec<String>>>()
        }

        // Otherwise, we have a mix of Files and Folders (or folders only).
        // In this case, we get all the individual files, then the ones inside folders.
        // Then we merge them, and add all of them together.
        else {
            let mut paths_files = path_types.par_iter().filter_map(|x| {
                if let PathType::File(path) = x { Some(path.to_vec()) } else { None }
            }).collect::<Vec<Vec<String>>>();

            paths_files.append(&mut path_types.par_iter()
                .filter_map(|x| {
                    if let PathType::Folder(path) = x { Some(path.to_vec()) } else { None }
                })
            .map(|path| self.get_ref_packed_files_paths_by_path_start(&path).iter().map(|x| x.to_vec()).collect::<Vec<Vec<String>>>())
            .flatten()
            .collect::<Vec<Vec<String>>>());
            paths_files
        }
    }

    */
}


impl Decodeable for AnimPack {

    fn file_type(&self) -> FileType {
        FileType::AnimPack
    }

    fn decode<R: ReadBytes>(data: &mut R, _extra_data: Option<(&Schema, &str, bool)>) -> Result<Self> {

        let file_count = data.read_u32()?;
        let mut files: HashMap<String, RFile> = if file_count < 50_000 { HashMap::with_capacity(file_count as usize) } else { HashMap::new() };

        for _ in 0..file_count {
            let path = data.read_sized_string_u8()?;
            let byte_count = data.read_u32()? as usize;
            let data = data.read_slice(byte_count, false)?;

            let file = RFile {
                path: path.to_owned(),
                timestamp: None,
                data: RFileInnerData::Catched(data),
            };

            files.insert(path, file);
        }

        // If we've reached this, we've successfully decoded the entire AnimPack.
        Ok(Self {
            files,
        })
    }
}

impl Encodeable for AnimPack {
    fn encode<W: WriteBytes>(&self, buffer: &mut W) -> Result<()> {
        buffer.write_u32(self.files.len() as u32)?;

        // TODO: check if sorting is needed.
        for file in self.files.values() {
            buffer.write_sized_string_u8(&file.path_raw())?;
            buffer.write_u32(file.data().len() as u32)?;
            buffer.write_all(&file.data())?;
        }

        Ok(())
    }
}


impl<T: Decodeable> Container<T> for AnimPack {
    fn files(&self) -> &HashMap<std::string::String, RFile> {
        &self.files
    }

    fn files_mut(&mut self) -> &mut HashMap<std::string::String, RFile> {
        &mut self.files
    }
}

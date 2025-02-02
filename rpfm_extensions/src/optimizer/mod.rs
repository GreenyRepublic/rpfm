//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

//! This module contains the [Optimizable] and [OptimizableContainer] trait.

use rayon::prelude::*;

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use rpfm_lib::error::{RLibError, Result};
use rpfm_lib::files::{Container, ContainerPath, DecodeableExtraData, db::DB, FileType, loc::Loc, pack::Pack, RFileDecoded, table::DecodedData};
use rpfm_lib::schema::Schema;

use crate::dependencies::Dependencies;

//-------------------------------------------------------------------------------//
//                             Trait definitions
//-------------------------------------------------------------------------------//

/// This trait marks an struct (mainly structs representing decoded files) as `Optimizable`, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait Optimizable {

    /// This function optimizes the provided struct to reduce its size and improve compatibility.
    ///
    /// It returns if the struct has been left in an state where it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies) -> bool;
}

/// This trait marks a [Container](rpfm_lib::files::Container) as an `Optimizable` container, meaning it can be cleaned up to reduce size and improve compatibility.
pub trait OptimizableContainer: Container {

    /// This function optimizes the provided [Container](rpfm_lib::files::Container) to reduce its size and improve compatibility.
    ///
    /// It returns the list of files that has been safetly deleted during the optimization process.
    fn optimize(&mut self, dependencies: &mut Dependencies, schema: &Schema, optimize_datacored_tables: bool) -> Result<HashSet<String>>;
}

//-------------------------------------------------------------------------------//
//                           Trait implementations
//-------------------------------------------------------------------------------//

impl OptimizableContainer for Pack {

    /// This function optimizes the provided [Pack](rpfm_lib::files::pack::Pack) file in order to make it smaller and more compatible.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - DB/Loc tables (except if the table has the same name as his vanilla/parent counterpart and `optimize_datacored_tables` is false):
    ///     - Removal of duplicated entries.
    ///     - Removal of ITM (Identical To Master) entries.
    ///     - Removal of ITNR (Identical To New Row) entries.
    ///     - Removal of empty tables.
    ///
    /// NOTE: due to a consequence of the optimization, all tables are also sorted by their first key.
    ///
    /// Not yet working:
    /// - Remove XML files in map folders.
    /// - Remove files identical to Parent/Vanilla files (if is identical to vanilla, but a parent mod overwrites it, it ignores it).
    fn optimize(&mut self, dependencies: &mut Dependencies, schema: &Schema, optimize_datacored_tables: bool) -> Result<HashSet<String>> {

        // We can only optimize if we have vanilla data available.
        if !dependencies.is_vanilla_data_loaded(true) {
            return Err(RLibError::DependenciesCacheNotGeneratedorOutOfDate);
        }

        // List of files to delete.
        let mut files_to_delete: HashSet<String> = HashSet::new();
        /*
        // First, do a hash pass over all the files, and mark for removal those that match by path and hash with vanilla/parent ones.
        let packedfiles_paths = self.get_ref_packed_files_all_paths().iter().map(|x| PathType::File(x.to_vec())).collect::<Vec<PathType>>();
        let mut dependencies_overwritten_files = dependencies.get_most_relevant_files_by_paths(&packedfiles_paths);
        files_to_delete.append(&mut dependencies_overwritten_files.iter_mut().filter_map(|dep_packed_file| {
            if let Some(packed_file) = self.get_ref_mut_packed_file_by_path(dep_packed_file.get_path()) {
                if let Ok(local_hash) = packed_file.get_hash_from_data() {
                    if let Ok(dependency_hash) = dep_packed_file.get_hash_from_data() {
                        if local_hash == dependency_hash {
                            Some(packed_file.get_path().to_vec())
                        } else { None }
                    } else { None }
                } else { None }
            } else { None }
        }).collect());
        */

        let mut extra_data = DecodeableExtraData::default();
        extra_data.set_schema(Some(schema));
        let extra_data = Some(extra_data);

        // Then, do a second pass, this time over the decodeable files that we can optimize.
        files_to_delete.extend(self.files_mut().iter_mut().filter_map(|(path, rfile)| {

            // Only check it if it's not already marked for deletion.
            if files_to_delete.get(path).is_none() {

                match rfile.file_type() {
                    FileType::DB => {

                        // Unless we specifically wanted to, ignore the same-name-as-vanilla-or-parent files,
                        // as those are probably intended to overwrite vanilla files, not to be optimized.
                        if optimize_datacored_tables || !dependencies.file_exists(path, true, true, true) {
                            if let Ok(Some(RFileDecoded::DB(mut db))) = rfile.decode(&extra_data, false, true) {
                                if db.optimize(dependencies) {
                                    return Some(path.to_owned());
                                }
                            }
                        }
                    }

                    FileType::Loc => {

                        // Same as with tables, don't optimize them if they're overwriting.
                        if optimize_datacored_tables || !dependencies.file_exists(path, true, true, true) {
                            if let Ok(Some(RFileDecoded::Loc(mut loc))) = rfile.decode(&extra_data, false, true) {
                                if loc.optimize(dependencies) {
                                    return Some(path.to_owned());
                                }
                            }
                        }
                    }

                    /*
                    PackedFileType::Text(text_type) => {
                        if !path.is_empty() && path.starts_with(&Self::get_terry_map_path()) && text_type == TextType::Xml {
                            return Some(path.to_vec());
                        }
                    }*/

                    // Ignore the rest.
                    _ => {}
                }
            }

            None
        }).collect::<Vec<String>>());

        // Delete all the files marked for deletion.
        files_to_delete.iter().for_each(|x| { self.remove(&ContainerPath::File(x.to_owned())); });

        // Return the deleted files, so the caller can know what got removed.
        Ok(files_to_delete)
    }
}

impl Optimizable for DB {

    /// This function optimizes the provided [DB](rpfm_lib::files::db::DB) file in order to make it smaller and more compatible.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - Removal of duplicated entries.
    /// - Removal of ITM (Identical To Master) entries.
    /// - Removal of ITNR (Identical To New Row) entries.
    ///
    /// It returns if the DB is empty, meaning it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies) -> bool {
        match self.data(&None) {
            Ok(entries) => {

                // Get a manipulable copy of all the entries, so we can optimize it.
                let mut entries = entries.to_vec();
                let definition = self.definition();
                let first_key = definition.fields_processed_sorted(true).iter().position(|x| x.is_key()).unwrap_or(0);

                match dependencies.db_data(self.table_name(), true, true) {
                    Ok(mut vanilla_tables) => {

                        // First, merge all vanilla and parent db fragments into a single HashSet.
                        let vanilla_table = vanilla_tables.iter_mut()
                            .filter_map(|file| {
                                if let Ok(RFileDecoded::DB(table)) = file.decoded() {
                                    table.data(&None).ok().map(|x| x.to_vec())
                                } else { None }
                            })
                            .flatten()
                            .map(|x| {

                                // We map all floats here to string representations of floats, so we can actually compare them reliably.
                                let json = x.iter().map(|data|
                                    if let DecodedData::F32(value) = data {
                                        DecodedData::StringU8(format!("{:.4}", value))
                                    } else {
                                        data.to_owned()
                                    }
                                ).collect::<Vec<DecodedData>>();
                                serde_json::to_string(&json).unwrap()
                            })
                            .collect::<HashSet<String>>();

                        // Remove ITM and ITNR entries.
                        let new_row = self.new_row().iter().map(|data|
                            if let DecodedData::F32(value) = data {
                                DecodedData::StringU8(format!("{:.4}", value))
                            } else {
                                data.to_owned()
                            }
                        ).collect::<Vec<DecodedData>>();

                        entries.retain(|entry| {
                            let entry_json = entry.iter().map(|data|
                                if let DecodedData::F32(value) = data {
                                    DecodedData::StringU8(format!("{:.4}", value))
                                } else {
                                    data.to_owned()
                                }
                            ).collect::<Vec<DecodedData>>();
                            !vanilla_table.contains(&serde_json::to_string(&entry_json).unwrap()) && entry != &new_row
                        });

                        // Sort the table so it can be dedup. Sorting floats is a pain in the ass.
                        entries.par_sort_by(|a, b| {
                            let ordering = if let DecodedData::F32(x) = a[first_key] {
                                if let DecodedData::F32(y) = b[first_key] {
                                    if float_eq::float_eq!(x, y, abs <= 0.0001) {
                                        Some(Ordering::Equal)
                                    } else { None }
                                } else { None }
                            } else { None };

                            match ordering {
                                Some(ordering) => ordering,
                                None => a[first_key].data_to_string().partial_cmp(&b[first_key].data_to_string()).unwrap_or(Ordering::Equal)
                            }
                        });

                        entries.dedup();

                        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at the Container level.
                        //
                        // NOTE: This may fail, but in that case the table will not be left empty, which we check in the next line.
                        let _ = self.set_data(None, &entries);
                        self.data(&None).unwrap().is_empty()
                    }
                    Err(_) => false,
                }
            }

            // We don't optimize sql-backed data.
            Err(_) => false,
        }
    }
}

impl Optimizable for Loc {

    /// This function optimizes the provided [Loc](rpfm_lib::files::loc::Loc) file in order to make it smaller and more compatible.
    ///
    /// Specifically, it performs the following optimizations:
    ///
    /// - Removal of duplicated entries.
    /// - Removal of ITM (Identical To Master) entries.
    /// - Removal of ITNR (Identical To New Row) entries.
    ///
    /// It returns if the Loc is empty, meaning it can be safetly deleted.
    fn optimize(&mut self, dependencies: &mut Dependencies) -> bool {
        match self.data(&None) {
            Ok(entries) => {

                // Get a manipulable copy of all the entries, so we can optimize it.
                let mut entries = entries.to_vec();
                match dependencies.loc_data(true, true) {
                    Ok(mut vanilla_tables) => {

                        // First, merge all vanilla and parent locs into a single HashMap<key, value>. We don't care about the third column.
                        let vanilla_table = vanilla_tables.iter_mut()
                            .filter_map(|file| {
                                if let Ok(RFileDecoded::Loc(table)) = file.decoded() {
                                    table.data(&None).ok().map(|x| x.to_vec())
                                } else { None }
                            })
                            .flat_map(|data| data.iter()
                                .map(|data| (data[0].data_to_string().to_string(), data[1].data_to_string().to_string()))
                                .collect::<Vec<(String, String)>>())
                            .collect::<HashMap<String, String>>();

                        // Remove ITM and ITNR entries.
                        let new_row = self.new_row();
                        entries.retain(|entry| {
                            if entry == &new_row {
                                return false;
                            }

                            match vanilla_table.get(&*entry[0].data_to_string()) {
                                Some(vanilla_value) => &*entry[1].data_to_string() != vanilla_value,
                                None => true
                            }
                        });

                        // Sort the table so it can be dedup.
                        entries.par_sort_by(|a, b| a[0].data_to_string().partial_cmp(&b[0].data_to_string()).unwrap_or(Ordering::Equal));
                        entries.dedup();

                        // Then we overwrite the entries and return if the table is empty or now, so we can optimize it further at the Container level.
                        //
                        // NOTE: This may fail, but in that case the table will not be left empty, which we check in the next line.
                        let _ = self.set_data(&entries);
                        self.data(&None).unwrap().is_empty()
                    }
                    Err(_) => false,
                }
            }

            // We don't optimize sql-backed data.
            Err(_) => false,
        }
    }
}

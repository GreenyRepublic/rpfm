//---------------------------------------------------------------------------//
// Copyright (c) 2017-2022 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
This module defines the code used for thread communication.
!*/

use crate::packedfile_views::DataSource;
use rpfm_lib::files::pack::PackSettings;
use qt_core::QEventLoop;

use anyhow::Error;
use crossbeam::channel::{Receiver, Sender, unbounded};

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt::Debug;
use std::path::PathBuf;


//use rpfm_extensions::dependencies::DependenciesInfo;
use rpfm_extensions::diagnostics::Diagnostics;
use rpfm_extensions::search::{GlobalSearch, MatchHolder};

use rpfm_lib::files::{anim_fragment::AnimFragment, anims_table::AnimsTable, ContainerPath, ca_vp8::{CaVp8, SupportedFormats}, db::DB, esf::ESF, FileType, image::Image, loc::Loc, matched_combat::MatchedCombat, RFileDecoded, rigidmodel::RigidModel, text::Text, uic::UIC, unit_variant::UnitVariant};
use rpfm_lib::games::pfh_file_type::PFHFileType;
use rpfm_lib::integrations::git::GitResponse;
use rpfm_lib::schema::Definition;

/*

use rpfm_lib::packedfile::ca_vp8::{CaVp8, SupportedFormats};
use rpfm_lib::packedfile::{DecodedPackedFile, PackedFileType};
use rpfm_lib::packedfile::esf::ESF;
use rpfm_lib::packedfile::image::Image;
use rpfm_lib::packedfile::table::{DependencyData, anim_fragment::AnimFragment, animtable::AnimTable, db::{DB, CascadeEdition}, loc::Loc, matched_combat::MatchedCombat};
use rpfm_lib::packedfile::text::Text;
use rpfm_lib::packedfile::rigidmodel::RigidModel;
use rpfm_lib::packedfile::uic::UIC;
use rpfm_lib::packfile::{ContainerInfo, PackFileSettings, ContainerPath, PFHFileType};
use rpfm_lib::packfile::packedfile::{PackedFile, RFileInfo};
use rpfm_lib::schema::{APIResponseSchema, Definition, Schema, patch::SchemaPatch};
use rpfm_lib::settings::*;
use rpfm_lib::tips::{APIResponseTips, Tip};
use rpfm_lib::updater::APIResponse;

*/
use crate::app_ui::NewPackedFile;
use crate::backend::*;
//use crate::packedfile_views::DataSource;
use crate::ui_state::shortcuts::Shortcuts;
use crate::updater::APIResponse;
//use crate::views::table::TableType;

/// This const is the standard message in case of message communication error. If this happens, crash the program.
pub const THREADS_COMMUNICATION_ERROR: &str = "Error in thread communication system. Response received: ";
pub const THREADS_SENDER_ERROR: &str = "Error in thread communication system. Sender failed to send message.";

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains the senders and receivers necessary to communicate both, backend and frontend threads.
///
/// You can use them by using the send/recv functions implemented for it.
pub struct CentralCommand<T: Send + Sync + Debug> {
    sender_background: Sender<(Sender<T>, Command)>,
    sender_network:  Sender<(Sender<T>, Command)>,

    receiver_background: Receiver<(Sender<T>, Command)>,
    receiver_network:  Receiver<(Sender<T>, Command)>,
}

/// This enum defines the commands (messages) you can send to the background thread in order to execute actions.
///
/// Each command should include the data needed for his own execution. For a more detailed explanation, check the
/// docs of each command.
#[derive(Debug)]
pub enum Command {

    /// This command is used to close a thread.
    Exit,

    /// This command is used when we want to reset the open `PackFile` to his original state.
    ResetPackFile,

    /// This command is used when we want to remove from memory the extra packfile with the provided path.
    RemovePackFileExtra(PathBuf),

    /// This command is used to "clean" a Packfile from corrupted files and save it to disk.
    CleanAndSavePackFileAs(PathBuf),

    /// This command is used when we want to create a new `PackFile`.
    NewPackFile,

    /// This command is used when we want to save our currently open `PackFile`.
    SavePackFile,

    /// This command is used when we want to save our currently open `PackFile` as another `PackFile`.
    SavePackFileAs(PathBuf),

    /// This command is used when we want to save our settings to disk. It requires the settings to save.
    //SetSettings(Settings),

    /// This command is used when we want to save our shortcuts to disk. It requires the shortcuts to save.
    SetShortcuts(Shortcuts),

    /// This command is used when we want to get the data used to build the `TreeView`.
    GetPackFileDataForTreeView,

    /// Same as the one before, but for the extra `PackFile`. It requires the pathbuf of the PackFile.
    GetPackFileExtraDataForTreeView(PathBuf),

    /// This command is used to open one or more `PackFiles`. It requires the paths of the `PackFiles`.
    OpenPackFiles(Vec<PathBuf>),

    /// This command is used to open an extra `PackFile`. It requires the path of the `PackFile`.
    OpenPackFileExtra(PathBuf),

    /// This command is used to open all the CA PackFiles for the game selected as one.
    LoadAllCAPackFiles,

    /// This command is used when we want to get the `RFileInfo` of one or more `PackedFiles`.
    GetPackedFilesInfo(Vec<String>),

    /// This command is used when we want to perform a `Global Search`. It requires the search info.
    GlobalSearch(GlobalSearch),

    /// This command is used when we want to change the `Game Selected`. It contains the name of the game to select.
    SetGameSelected(String),

    /// This command is used when we want to change the `Type` of the currently open `PackFile`. It contains the new type.
    SetPackFileType(PFHFileType),

    /// This command is used when we want to generate the dependencies cache for a game. It contains the path of the
    /// source raw db files, the `Raw DB Version` of the currently selected game, and if we should has the files or not.
    GenerateDependenciesCache,

    /// This command is used when we want to update the currently loaded Schema with data from the game selected's Assembly Kit.
    /// It contains the path of the source files, if needed.
    UpdateCurrentSchemaFromAssKit,

    /// This command is used when we want to trigger an optimization pass over the currently open `PackFile`.
    OptimizePackFile,

    /// This command is used to patch the SiegeAI of a Siege Map for warhammer games.
    PatchSiegeAI,

    /// This command is used when we want to change the `Index Includes Timestamp` flag in the currently open `PackFile`
    ChangeIndexIncludesTimestamp(bool),

    /// This command is used when we want to change the `Data is Compressed` flag in the currently open `PackFile`
    ChangeDataIsCompressed(bool),

    /// This command is used when we want to know the current path of our currently open `PackFile`.
    GetPackFilePath,

    /// This command is used when we want to get the info of the provided `PackedFile`.
    GetRFileInfo(String),

    /// This command is used when we want to check if there is an RPFM update available.
    CheckUpdates,

    /// This command is used when we want to check if there is an Schema update available.
    CheckSchemaUpdates,

    /// This command is used when we want to update our schemas.
    UpdateSchemas,

    /// This command is used when we want to know if there is a Dependency Database loaded in memory.
    ///
    /// Pass true if you want to ensure the dependencies were built with the AssKit.
    IsThereADependencyDatabase(bool),

    /// This command is used when we want to create a new `PackedFile` inside the currently open `PackFile`.
    ///
    /// It requires the path of the new PackedFile, and the `NewPackedFile` with the new PackedFile's info.
    NewPackedFile(String, NewPackedFile),

    /// This command is used when we want to add one or more Files to our currently open `PackFile`.
    ///
    /// It requires the list of filesystem paths to add, their path once they're inside the `PackFile`, and if the TSV files found must be imported or not.
    AddPackedFiles(Vec<PathBuf>, Vec<String>, Option<Vec<PathBuf>>, bool),

    /// This command is used when we want to decode a PackedFile to be shown on the UI. It contains the path of the file, and were it is.
    DecodePackedFile(String, DataSource),

    // This command is used when we want to save an edited `PackedFile` back to the `PackFile`.
    SavePackedFileFromView(String, RFileDecoded),

    // This command is used when we want to add a PackedFile from one PackFile into another.
    AddPackedFilesFromPackFile((PathBuf, Vec<ContainerPath>)),

    // This command is used when we want to add a PackedFile from our PackFile to an Animpack.
    AddPackedFilesFromPackFileToAnimpack(String, Vec<ContainerPath>),

    // This command is used when we want to add a PackedFile from an AnimPack to our PackFile.
    AddPackedFilesFromAnimpack(String, Vec<ContainerPath>),

    // This command is used when we want to delete a PackedFile from an AnimPack.
    DeleteFromAnimpack((String, Vec<ContainerPath>)),

    // This command is used when we want to delete one or more PackedFiles from a PackFile. It contains the ContainerPath of each PackedFile to delete.
    DeletePackedFiles(Vec<ContainerPath>),

    // This command is used when we want to extract one or more PackedFiles from a PackFile. It contains the ContainerPaths to extract and the extraction path, and a bool to know if tables must be exported to tsv on extract or not.
    ExtractPackedFiles(Vec<ContainerPath>, PathBuf, bool),

    // This command is used when we want to rename one or more PackedFiles in a PackFile. It contains a Vec with their original ContainerPath and their new name.
    RenamePackedFiles(Vec<(ContainerPath, String)>),

    // This command is used when we want to import a large amount of table-like files from TSV files.
    //MassImportTSV(Vec<PathBuf>, Option<String>),

    // This command is used when we want to export a large amount of table-like files as TSV files.
    //MassExportTSV(Vec<ContainerPath>, PathBuf),

    /// This command is used when we want to know if a folder exists in the currently open PackFile.
    FolderExists(String),

    /// This command is used when we want to know if a PackedFile exists in the currently open PackFile.
    PackedFileExists(String),

    /// This command is used when we want to get the table names (the folder of the tables) of all DB files in our dependency PackFiles.
    GetTableListFromDependencyPackFile,

    /// This command is used when we want to get the version of the table provided that's compatible with the version of the game we currently have installed.
    GetTableVersionFromDependencyPackFile(String),

    /// This command is used when we want to get the definition of the table provided that's compatible with the version of the game we currently have installed.
    GetTableDefinitionFromDependencyPackFile(String),

    /// This command is used when we want to merge multiple compatible tables into one. The contents of this are as follows:
    /// - Vec<Vec<String>>: List of paths to merge.
    /// - String: Name of the new merged table.
    /// - Bool: Should we delete the source files after merging them?
    MergeTables(Vec<Vec<String>>, String, bool),

    // This command is used when we want to update a table to a newer version.
    //UpdateTable(ContainerPath),

    /// This command is used when we want to replace some specific matches in a Global Search.
    GlobalSearchReplaceMatches(GlobalSearch, Vec<MatchHolder>),

    /// This command is used when we want to replace all matches in a Global Search.
    GlobalSearchReplaceAll(GlobalSearch),

    /// This command is used when we want to add entire folders to the PackFile. It contains their path in disk and their starting path in the PackFile,
    /// the list of paths to ignore, if any, and if any tsv found should be imported as tables.
    AddPackedFilesFromFolder(Vec<(PathBuf, String)>, Option<Vec<PathBuf>>, bool),

    /// This command is used to decode all tables referenced by columns in the provided definition and return their data.
    /// It requires the table name, the definition of the table to get the reference data from and the list of PackedFiles to ignore.
    GetterserenceDataFromDefinition(String, Definition, Vec<Vec<String>>),

    /// This command is used to get the list of PackFiles that are marked as dependency of our PackFile.
    GetDependencyPackFilesList,

    /// This command is used to set the list of PackFiles that are marked as dependency of our PackFile.
    SetDependencyPackFilesList(Vec<String>),

    /// This command is used to get a full PackedFile to the UI. Requires the path of the PackedFile.
    GetPackedFile(Vec<String>),

    // This command is used to get a full list of PackedFile from all known sources to the UI. Requires the path of the PackedFile.
    //GetPackedFilesFromAllSources(Vec<ContainerPath>),

    // This command is used to change the format of a ca_vp8 video packedfile. Requires the path of the PackedFile and the new format.
    SetCaVp8Format(String, SupportedFormats),

    // This command is used to save the provided schema to disk.
    //SaveSchema(Schema),

    /// This command is used to save to encoded data the cache of the provided paths, and then clean up the cache.
    CleanCache(Vec<Vec<String>>),

    /// This command is used to export a table as TSV. Requires the internal and destination paths for the PackedFile.
    ExportTSV(String, PathBuf),

    /// This command is used to import a TSV as a table. Requires the internal and destination paths for the PackedFile.
    ImportTSV(String, PathBuf),

    /// This command is used to open in the defaul file manager the folder of the currently open PackFile.
    OpenContainingFolder,

    /// This command is used to open a PackedFile on a external program. Requires the internal path of the PackedFile.
    OpenPackedFileInExternalProgram(String),

    /// This command is used to save a PackedFile from an external program. Requires both, internal and external paths of the PackedFile.
    SavePackedFileFromExternalView(String, PathBuf),

    /// This command is used to update the program to the last version available, if possible.
    UpdateMainProgram,

    /// This command is used to trigger an autosave to a backup from time to time.
    TriggerBackupAutosave,

    /// This command is used to trigger a full diagnostics check over the open PackFile.
    DiagnosticsCheck,

    // This command is used to trigger a partial diagnostics check over the open PackFile.
    DiagnosticsUpdate(Diagnostics, Vec<ContainerPath>),

    /// This command is used to get the settings of the currently open PackFile.
    GetPackSettings,

    // This command is used to set the settings of the currently open PackFile.
    SetPackSettings(PackSettings),

    /// This command is used to trigger the debug missing table definition's code.
    GetMissingDefinitions,

    /// This command is used to rebuild the dependencies of a PackFile. The bool is for rebuilding the whole dependencies, or just the mod-specific ones.
    RebuildDependencies(bool),

    // This command is used to trigger a cascade edition on all referenced data.
    //CascadeEdition(CascadeEdition),

    /// This command is used for the Go To Definition feature. Contains table, column, and value to search.
    GoToDefinition(String, String, String),

    /// This command is used to get the source data of a loc key. Contains the loc key to search.
    GetSourceDataFromLocKey(String),

    /// This command is used to get the loc file/column/row of a key. Contains the loc key to search.
    GoToLoc(String),

    /// This command is used for the Find References feature. Contains list of table/columns to search, and value to search.
    SearchReferences(HashMap<String, Vec<String>>, String),

    /// This command is used to get the type of a File.
    GetFileType(String),

    /// This command is used to get the name of the currently open PackFile.
    GetPackFileName,

    /// This command is used to get the raw data of a PackedFile.
    GetPackedFileRawData(String),

    /// This command is used to import files from the dependencies into out PackFile.
    ImportDependenciesToOpenPackFile(BTreeMap<DataSource, Vec<ContainerPath>>),

    /// This command is used to save all provided PackedFiles into the current PackFile, then merge them and optimize them if possible.
    //SavePackedFilesToPackFileAndClean(Vec<PackedFile>),

    /// This command is used to get all the file names under a path in all dependencies.
    //GetPackedFilesNamesStartingWitPathFromAllSources(ContainerPath),

    /// This command is used to request all tips under a path, no matter their source.
    GetTipsForPath(Vec<String>),

    // This command is used to add a tip to the list of local tips.
    //AddTipToLocalTips(Tip),

    /// This command is used to delete a tip with an specific id.
    DeleteTipById(u64),

    /// This command is used to check if there are message updates available.
    CheckMessageUpdates,

    /// This command is used to download new message updates.
    UpdateMessages,

    /// This command is used to publish a tip to github.
    PublishTipById(u64),

    /// This command is used to upload a schema patch.
    //UploadSchemaPatch(SchemaPatch),

    /// This command is used to import a schema patch in the local schema patches.
    //ImportSchemaPatch(SchemaPatch),

    /// This command is used to generate all missing loc entries for the currently open PackFile.
    GenerateMissingLocData,

    /// This command is used to check for updates on the tw_autogen thing.
    CheckLuaAutogenUpdates,

    /// This command is used to update the tw_autogen thing.
    UpdateLuaAutogen,

    /// This command is used to initialize a MyMod Folder.
    InitializeMyModFolder(String, String),
}

/// This enum defines the responses (messages) you can send to the to the UI thread as result of a command.
///
/// Each response should be named after the types of the items it carries.
#[derive(Debug)]
pub enum Response {

    /// Generic response for situations of success.
    Success,

    /// Generic response for situations that returned an error.
    Error(Error),

    /// Response to return (bool).
    Bool(bool),

    /// Response to return (i32).
    I32(i32),

    /// Response to return (PathBuf).
    PathBuf(PathBuf),

    /// Response to return (String)
    String(String),

    // Response to return (ContainerInfo, Vec<RFileInfo>).
    ContainerInfoVecRFileInfo((ContainerInfo, Vec<RFileInfo>)),

    // Response to return (ContainerInfo).
    ContainerInfo(ContainerInfo),

    // Response to return (Option<RFileInfo>).
    OptionRFileInfo(Option<RFileInfo>),

    // Response to return (Vec<Option<RFileInfo>>).
    VecRFileInfo(Vec<RFileInfo>),

    // Response to return (GlobalSearch, Vec<RFileInfo>).
    GlobalSearchVecRFileInfo((GlobalSearch, Vec<RFileInfo>)),

    /// Response to return (Vec<Vec<String>>).
    VecVecString(Vec<Vec<String>>),

    // Response to return (Vec<ContainerPath>).
    VecContainerPath(Vec<ContainerPath>),

    // Response to return (Vec<(ContainerPath, Vec<String>)>).
    VecContainerPathContainerPath(Vec<(ContainerPath, ContainerPath)>),

    /// Response to return (String, Vec<Vec<String>>).
    StringVecVecString((String, Vec<Vec<String>>)),

    /// Response to return `APIResponse`.
    APIResponse(APIResponse),

    /// Response to return `APIResponseGit`.
    APIResponseGit(GitResponse),

    /// Response to return `(AnimFragment, RFileInfo)`.
    AnimFragmentRFileInfo(AnimFragment, RFileInfo),

    /// Response to return `(Vec<String>, RFileInfo)`.
    AnimPackRFileInfo(ContainerInfo, Vec<RFileInfo>, RFileInfo),

    /// Response to return `(AnimTable, RFileInfo)`.
    AnimsTableRFileInfo(AnimsTable, RFileInfo),

    /// Response to return `(CaVp8, RFileInfo)`.
    CaVp8RFileInfo(CaVp8, RFileInfo),

    /// Response to return `(ESF, RFileInfo)`.
    ESFRFileInfo(ESF, RFileInfo),

    /// Response to return `(Image, RFileInfo)`.
    ImageRFileInfo(Image, RFileInfo),

    /// Response to return `(Text, RFileInfo)`.
    TextRFileInfo(Text, RFileInfo),

    /// Response to return `(DB, RFileInfo)`.
    DBRFileInfo(DB, RFileInfo),

    /// Response to return `(Loc, RFileInfo)`.
    LocRFileInfo(Loc, RFileInfo),

    /// Response to return `(MatchedCombat, RFileInfo)`.
    MatchedCombatRFileInfo(MatchedCombat, RFileInfo),

    /// Response to return `(RigidModel, RFileInfo)`.
    RigidModelRFileInfo(RigidModel, RFileInfo),

    /// Response to return `(UIC, RFileInfo)`.
    UICRFileInfo(UIC, RFileInfo),

    UnitVariantRFileInfo(UnitVariant, RFileInfo),

    /// Response to return `(DecodedPackedFile, RFileInfo)`. For debug views.
    RFileDecodedRFileInfo(RFileDecoded, RFileInfo),

    /// Response to return `Text`.
    Text(Text),

    /// Response to return `Unknown`.
    Unknown,

    /// Response to return `(Vec<Vec<String>>, Vec<Vec<String>>)`.
    VecVecStringVecVecString((Vec<Vec<String>>, Vec<Vec<String>>)),

    /// Response to return `Vec<String>`.
    VecString(Vec<String>),

    /// Response to return `(i32, i32)`.
    I32I32((i32, i32)),

    /// Response to return `BTreeMap<i32, DependencyData>`.
    //BTreeMapI32DependencyData(BTreeMap<i32, DependencyData>),

    /// Response to return `Option<PackedFile>`.
    //OptionPackedFile(Option<PackedFile>),

    /// Response to return `TableType`.
    //TableType(TableType),

    /// Response to return `PackFileSettings`.
    PackSettings(PackSettings),

    /// Response to return `Vec<Vec<String>>, Vec<RFileInfo>`.
    //VecVecStringVecRFileInfo(Vec<Vec<String>>, Vec<RFileInfo>),

    /// Response to return `DataSource, Vec<String>, usize, usize`.
    //DataSourceVecStringUsizeUsize(DataSource, Vec<String>, usize, usize),

    /// Response to return `Vec<(DataSource, Vec<String>, String, usize, usize)>`.
    //VecDataSourceVecStringStringUsizeUsize(Vec<(DataSource, Vec<String>, String, usize, usize)>),

    /// Response to return `Option<(String, String, String)>`.
    OptionStringStringString(Option<(String, String, String)>),

    /// Response to return `FileType`.
    FileType(FileType),

    /// Response to return `Vec<u8>`.
    VecU8(Vec<u8>),

    /// Response to return `DependenciesInfo`.
    DependenciesInfo(DependenciesInfo),

    /// Response to return `HashMap<DataSource, HashMap<Vec<String>, PackedFile>>`.
    //HashMapDataSourceHashMapVecStringPackedFile(HashMap<DataSource, HashMap<Vec<String>, PackedFile>>),
    //HashMapDataSourceHashSetVecString(HashMap<DataSource, HashSet<Vec<String>>>),
    Diagnostics(Diagnostics),
    //DiagnosticsVecRFileInfo(Diagnostics, Vec<RFileInfo>),
    Definition(Definition),
    //VecTipVecTip(Vec<Tip>, Vec<Tip>),
    HashSetString(HashSet<String>),
    StringHashSetString(String, HashSet<String>)
}

//-------------------------------------------------------------------------------//
//                              Implementations
//-------------------------------------------------------------------------------//

/// Default implementation of `CentralCommand`.
impl<T: Send + Sync + Debug> Default for CentralCommand<T> {
    fn default() -> Self {
        let (sender_background, receiver_background) = unbounded();
        let (sender_network, receiver_network) = unbounded();
        Self {
            sender_background,
            sender_network,
            receiver_background,
            receiver_network,
        }
    }
}

/// Implementation of `CentralCommand`.
impl<T: Send + Sync + Debug> CentralCommand<T> {

    /// This function serves as a generic way for commands to be sent to the backend.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    fn send<S: Send + Sync + Debug>(sender: &Sender<(Sender<T>, S)>, data: S) -> Receiver<T> {
        let (sender_back, receiver_back) = unbounded();
        if let Err(error) = sender.send((sender_back, data)) {
            panic!("{}: {}", THREADS_SENDER_ERROR, error);
        }

        receiver_back
    }

    /// This function serves to send a message from the main thread to the background thread.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    pub fn send_background(&self, data: Command) -> Receiver<T> {
        Self::send(&self.sender_background, data)
    }

    /// This function serves to send a message from the main thread to the network thread.
    ///
    /// It returns the receiver which will receive the answers for the command, if any.
    pub fn send_network(&self, data: Command) -> Receiver<T> {
        Self::send(&self.sender_network, data)
    }

    /// This function serves to send a message back through a generated channel.
    pub fn send_back(sender: &Sender<T>, data: T) {
        if let Err(error) = sender.send(data) {
            panic!("{}: {}", THREADS_SENDER_ERROR, error);
        }
    }

    /// This functions serves to receive messages on the background thread.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv_background(&self) -> (Sender<T>, Command) {
        let response = self.receiver_background.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This functions serves to receive messages on the network thread.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv_network(&self) -> (Sender<T>, Command) {
        let response = self.receiver_network.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function does only try once, and it locks the thread. Panics if the response fails.
    pub fn recv(receiver: &Receiver<T>) -> T {
        let response = receiver.recv();
        match response {
            Ok(data) => data,
            Err(_) => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
        }
    }

    /// This functions serves to receive messages from a generated channel.
    ///
    /// This function will keep asking for a response, keeping the UI responsive. Use it for heavy tasks.
    ///
    /// NOTE: Beware of other events triggering when this keeps the UI enabled. It can lead to crashes.
    pub fn recv_try(receiver: &Receiver<T>) -> T {
        let event_loop = unsafe { QEventLoop::new_0a() };
        loop {

            // Check the response and, in case of error, try again. If the error is "Disconnected", CTD.
            let response = receiver.try_recv();
            match response {
                Ok(data) => return data,
                Err(error) => if error.is_disconnected() {
                    panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                }
            }
            unsafe { event_loop.process_events_0a(); }
        }
    }
}

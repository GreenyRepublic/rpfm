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
Module with all the code related to the main `AppUISlot`.
!*/

use qt_widgets::QAction;
use qt_widgets::QDialog;
use qt_widgets::{QFileDialog, q_file_dialog::FileMode};
use qt_widgets::QGridLayout;
use qt_widgets::{QMessageBox, q_message_box};
use qt_widgets::QPushButton;
use qt_widgets::QTextEdit;
use qt_widgets::SlotOfQPoint;

use qt_gui::QCursor;
use qt_gui::QDesktopServices;

use qt_core::QBox;
use qt_core::{SlotOfBool, SlotOfInt, SlotNoArgs};
use qt_core::QFlags;
use qt_core::QPtr;
use qt_core::QString;
use qt_core::QUrl;
use qt_core::WidgetAttribute;

use std::collections::BTreeMap;
use std::fs::{copy, remove_file, remove_dir_all};
use std::path::PathBuf;
use std::rc::Rc;

use rpfm_lib::files::ContainerPath;
use rpfm_lib::games::{pfh_file_type::PFHFileType, supported_games::*};
use rpfm_lib::integrations::log::*;

use crate::app_ui::AppUI;
use crate::backend::*;
use crate::CENTRAL_COMMAND;
use crate::communications::{CentralCommand, THREADS_COMMUNICATION_ERROR, Command, Response};
use crate::dependencies_ui::DependenciesUI;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::DOCS_BASE_URL;
use crate::GAME_SELECTED;
use crate::global_search_ui::GlobalSearchUI;
use crate::locale::{qtr, tr, tre};
use crate::mymod_ui::MyModUI;
use crate::pack_tree::*;
use crate::packedfile_views::{DataSource, View, ViewType};
use crate::packfile_contents_ui::PackFileContentsUI;
use crate::PATREON_URL;
use crate::references_ui::ReferencesUI;
use crate::settings_ui::{backend::*, SettingsUI};
use crate::ui::GameSelectedIcons;
use crate::{ui_state::OperationalMode, UI_STATE};
use crate::utils::*;
use crate::VERSION;
use crate::VERSION_SUBTITLE;
use crate::views::table::utils::{get_reference_data, setup_item_delegates};

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the slots we need to respond to signals of EVERY widget/action created at the start of the program.
///
/// This means everything you can do with the stuff you have in the `AppUI` goes here.
pub struct AppUISlots {

    //-----------------------------------------------//
    // `PackFile` menu slots.
    //-----------------------------------------------//
    pub packfile_open_menu: QBox<SlotNoArgs>,
    pub packfile_new_packfile: QBox<SlotOfBool>,
    pub packfile_open_packfile: QBox<SlotOfBool>,
    pub packfile_save_packfile: QBox<SlotOfBool>,
    pub packfile_save_packfile_as: QBox<SlotOfBool>,
    pub packfile_install: QBox<SlotOfBool>,
    pub packfile_uninstall: QBox<SlotOfBool>,
    pub packfile_load_all_ca_packfiles: QBox<SlotOfBool>,
    pub packfile_change_packfile_type: QBox<SlotOfBool>,
    pub packfile_index_includes_timestamp: QBox<SlotOfBool>,
    pub packfile_data_is_compressed: QBox<SlotOfBool>,
    pub packfile_preferences: QBox<SlotOfBool>,
    pub packfile_quit: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `MyMod` menu slots.
    //-----------------------------------------------//
    pub mymod_open_menu: QBox<SlotNoArgs>,
    pub mymod_open_mymod_folder: QBox<SlotOfBool>,
    pub mymod_new: QBox<SlotOfBool>,
    pub mymod_delete_selected: QBox<SlotOfBool>,
    pub mymod_import: QBox<SlotOfBool>,
    pub mymod_export: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `View` menu slots.
    //-----------------------------------------------//
    pub view_open_menu: QBox<SlotNoArgs>,
    pub view_toggle_packfile_contents: QBox<SlotOfBool>,
    pub view_toggle_global_search_panel: QBox<SlotOfBool>,
    pub view_toggle_diagnostics_panel: QBox<SlotOfBool>,
    pub view_toggle_dependencies_panel: QBox<SlotOfBool>,
    pub view_toggle_references_panel: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Game Selected` menu slots.
    //-----------------------------------------------//
    pub game_selected_launch_game: QBox<SlotOfBool>,
    pub game_selected_open_game_data_folder: QBox<SlotOfBool>,
    pub game_selected_open_game_assembly_kit_folder: QBox<SlotOfBool>,
    pub game_selected_open_config_folder: QBox<SlotOfBool>,
    pub change_game_selected: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Special Stuff` menu slots.
    //-----------------------------------------------//
    pub special_stuff_generate_dependencies_cache: QBox<SlotOfBool>,
    pub special_stuff_optimize_packfile: QBox<SlotOfBool>,
    pub special_stuff_patch_siege_ai: QBox<SlotOfBool>,
    pub special_stuff_rescue_packfile: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Tools` menu slots.
    //-----------------------------------------------//
    pub tools_faction_painter: QBox<SlotNoArgs>,
    pub tools_unit_editor: QBox<SlotNoArgs>,

    //-----------------------------------------------//
    // `About` menu slots.
    //-----------------------------------------------//
    pub about_about_qt: QBox<SlotOfBool>,
    pub about_about_rpfm: QBox<SlotOfBool>,
    pub about_open_manual: QBox<SlotOfBool>,
    pub about_patreon_link: QBox<SlotOfBool>,
    pub about_check_updates: QBox<SlotOfBool>,
    pub about_check_schema_updates: QBox<SlotOfBool>,
    pub about_check_message_updates: QBox<SlotOfBool>,
    pub about_check_lua_autogen_updates: QBox<SlotOfBool>,

    //-----------------------------------------------//
    // `Debug` menu slots.
    //-----------------------------------------------//
    pub debug_update_current_schema_from_asskit: QBox<SlotOfBool>,
    pub debug_import_schema_patch: QBox<SlotNoArgs>,

    //-----------------------------------------------//
    // `PackedFileView` slots.
    //-----------------------------------------------//
    pub packed_file_hide: QBox<SlotOfInt>,
    pub packed_file_update: QBox<SlotOfInt>,
    pub packed_file_unpreview: QBox<SlotOfInt>,

    //-----------------------------------------------//
    // `Generic` slots.
    //-----------------------------------------------//
    pub pack_file_backup_autosave: QBox<SlotNoArgs>,

    pub tab_bar_packed_file_context_menu_show: QBox<SlotOfQPoint>,
    pub tab_bar_packed_file_close: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_close_all: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_close_all_left: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_close_all_right: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_prev: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_next: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_import_from_dependencies: QBox<SlotNoArgs>,
    pub tab_bar_packed_file_toggle_tips: QBox<SlotNoArgs>,
}

pub struct AppUITempSlots {}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `AppUISlots`.
impl AppUISlots {

	/// This function creates an entire `AppUISlots` struct. Used to create the logic of the starting UI.
	pub unsafe fn new(
        app_ui: &Rc<AppUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
        dependencies_ui: &Rc<DependenciesUI>,
        references_ui: &Rc<ReferencesUI>,
    ) -> Self {

        //-----------------------------------------------//
        // `PackFile` menu logic.
        //-----------------------------------------------//

        // Slot to build the "Open from" submenus of the PackFile menu.
        let packfile_open_menu = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui => move || {
                info!("Triggering `Open PackFile Menu` By Slot");
                AppUI::build_open_from_submenus(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui);
            }
        ));

        // What happens when we trigger the "New PackFile" action.
        let packfile_new_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui => move |_| {

                // Check first if there has been changes in the PackFile.
                if AppUI::are_you_sure(&app_ui, false) {
                    info!("Triggering `New PackFile` By Slot");
                    AppUI::new_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui);
                }
            }
        ));

        let packfile_open_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            global_search_ui => move |_| {

                // Check first if there has been changes in the PackFile.
                info!("Triggering `Open PackFile` By Slot?");
                if AppUI::are_you_sure(&app_ui, false) {
                    info!("Triggering `Open PackFile` By Slot");

                    // Create the FileDialog to get the PackFile to open and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &app_ui.main_window,
                        &qtr("open_packfiles"),
                    );
                    file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                    file_dialog.set_file_mode(FileMode::ExistingFiles);

                    // Run it and expect a response (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {

                        // Now the fun thing. We have to get all the selected files, and then open them one by one.
                        // For that we use the same logic as for the "Load All CA PackFiles" feature.
                        let mut paths = vec![];
                        for index in 0..file_dialog.selected_files().count_0a() {
                            paths.push(PathBuf::from(file_dialog.selected_files().at(index).to_std_string()));
                        }

                        // Try to open it, and report it case of error.
                        if let Err(error) = AppUI::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &paths, "") {
                            return show_dialog(&app_ui.main_window, error, false);
                        }

                        if setting_bool("diagnostics_trigger_on_open") {
                            DiagnosticsUI::check(&app_ui, &diagnostics_ui);
                        }
                    }
                }
            }
        ));

        // What happens when we trigger the "Save PackFile" action.
        let packfile_save_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Save PackFile` By Slot");
                if let Err(error) = AppUI::save_packfile(&app_ui, &pack_file_contents_ui, false) {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }
        ));

        // What happens when we trigger the "Save PackFile As" action.
        let packfile_save_packfile_as = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Save PackFile As` By Slot");
                if let Err(error) = AppUI::save_packfile(&app_ui, &pack_file_contents_ui, true) {
                    show_dialog(&app_ui.main_window, error, false);
                }
            }
        ));

        // This slot is used for the "Install" action.
        let packfile_install = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Install` By Slot");

                // Save before installing, to ensure we always have the latest data on install.
                if let Err(error) = AppUI::save_packfile(&app_ui, &pack_file_contents_ui, false) {
                    return show_dialog(&app_ui.main_window, error, false);
                }

                // Get the current path of the PackFile.
                let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFilePath);
                let response = CentralCommand::recv(&receiver);
                let pack_path = if let Response::PathBuf(pack_path) = response { pack_path } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) };
                let mut pack_image_path = pack_path.clone();
                pack_image_path.set_extension("png");

                // Ensure it's a file and it's not in data before proceeding.
                if !pack_path.is_file() {
                    return show_dialog(&app_ui.main_window, "Pack to install not found on disk.", false);
                }

                if let Ok(mut game_local_mods_path) = GAME_SELECTED.read().unwrap().local_mods_path(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                    if !game_local_mods_path.is_dir() {
                        return show_dialog(&app_ui.main_window, "Game Path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.", false);
                    }

                    if pack_path.starts_with(&game_local_mods_path) {
                        return show_dialog(&app_ui.main_window, "This Pack is already being edited from the data folder of the game. You cannot install/uninstall it.", false);
                    }

                    if let Some(ref mod_name) = pack_path.file_name() {
                        game_local_mods_path.push(mod_name);

                        // Check if the PackFile is not a CA one before installing.
                        let ca_paths = match GAME_SELECTED.read().unwrap().ca_packs_paths(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                            Ok(paths) => paths,
                            Err(_) => return show_dialog(&app_ui.main_window, "You can't do that to a CA PackFile, you monster!", false),
                        };

                        if ca_paths.contains(&game_local_mods_path) {
                            return show_dialog(&app_ui.main_window, "You can't do that to a CA PackFile, you monster!", false);
                        }

                        if copy(pack_path, &game_local_mods_path).is_err() {
                            return show_dialog(&app_ui.main_window, "Error installing a Pack. Make sure the game/assembly kit is close and try again.", false);
                        }

                        // Try to copy the image too if exists.
                        game_local_mods_path.pop();
                        game_local_mods_path.push(pack_image_path.file_name().unwrap());
                        if pack_image_path.is_file() && copy(pack_image_path, &game_local_mods_path).is_err()  {
                            return show_dialog(&app_ui.main_window, "Error installing the thumbnail of a Pack. Make sure the game/assembly kit is close and try again.", false);
                        }

                        // Report the success, so the user knows it worked.
                        log_to_status_bar(&tr("install_success"));

                        // Enable the uninstall button.
                        app_ui.packfile_uninstall.set_enabled(true);
                    }
                }
            }
        ));

        // This slot is used for the "Uninstall" action.
        let packfile_uninstall = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                info!("Triggering `Uninstall` By Slot");

                // Get the current path of the PackFile.
                let receiver = CENTRAL_COMMAND.send_background(Command::GetPackFilePath);
                let response = CentralCommand::recv(&receiver);
                let pack_path = if let Response::PathBuf(pack_path) = response { pack_path } else { panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response) };

                // Ensure it's a file and it's not in data before proceeding.
                if !pack_path.is_file() {
                    return show_dialog(&app_ui.main_window, "Pack to install not found on disk.", false);
                }

                if let Ok(mut game_local_mods_path) = GAME_SELECTED.read().unwrap().local_mods_path(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                    if !game_local_mods_path.is_dir() {
                        return show_dialog(&app_ui.main_window, "Game Path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.", false);
                    }

                    if pack_path.starts_with(&game_local_mods_path) {
                        return show_dialog(&app_ui.main_window, "This Pack is already being edited from the data folder of the game. You cannot install/uninstall it.", false);
                    }

                    if let Some(ref mod_name) = pack_path.file_name() {
                        game_local_mods_path.push(mod_name);

                        let ca_paths = match GAME_SELECTED.read().unwrap().ca_packs_paths(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                            Ok(paths) => paths,
                            Err(_) => return show_dialog(&app_ui.main_window, "You can't do that to a CA PackFile, you monster!", false),
                        };

                        if ca_paths.contains(&game_local_mods_path) {
                            return show_dialog(&app_ui.main_window, "You can't do that to a CA PackFile, you monster!", false);
                        }

                        if remove_file(&game_local_mods_path).is_err() {
                            return show_dialog(&app_ui.main_window, "Error uninstalling the Pack from the game's folder. Make sure nothing else is using it and try again.", false);
                        }

                        // Report the success, so the user knows it worked.
                        log_to_status_bar(&tr("uninstall_success"));

                        // Disable the uninstall button.
                        app_ui.packfile_uninstall.set_enabled(false);
                    }
                }
            }
        ));

        // What happens when we trigger the "Load All CA PackFiles" action.
        let packfile_load_all_ca_packfiles = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

            // Check first if there has been changes in the PackFile. If we accept, just take all the PackFiles in the data folder
            // and open them all together, skipping mods.
            if AppUI::are_you_sure(&app_ui, false) {
                info!("Triggering `Load all CA PackFiles` By Slot");

                // Reset the autosave timer.
                let timer = setting_int("autosave_interval");
                if timer > 0 {
                    app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
                    app_ui.timer_backup_autosave.start_0a();
                }

                // Tell the Background Thread to create a new PackFile with the data of one or more from the disk.
                app_ui.toggle_main_window(false);

                // Destroy whatever it's in the PackedFile's views and clear the global search UI.
                GlobalSearchUI::clear(&global_search_ui);
                let _ = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, false);

                let receiver = CENTRAL_COMMAND.send_background(Command::LoadAllCAPackFiles);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {

                    // If it's success....
                    Response::ContainerInfo(ui_data) => {

                        // Set this PackFile always to type `Release`.
                        app_ui.change_packfile_type_release.set_checked(true);

                        // Disable all of these.
                        app_ui.change_packfile_type_data_is_encrypted.set_checked(false);
                        app_ui.change_packfile_type_index_includes_timestamp.set_checked(false);
                        app_ui.change_packfile_type_index_is_encrypted.set_checked(false);
                        app_ui.change_packfile_type_header_is_extended.set_checked(false);

                        // Set the compression level correctly, because otherwise we may fuckup some files.
                        app_ui.change_packfile_type_data_is_compressed.set_checked(*ui_data.compress());

                        // Update the TreeView.
                        let mut build_data = BuildData::new();
                        build_data.editable = true;
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);

                        match &*GAME_SELECTED.read().unwrap().game_key_name() {
                            KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3.trigger(),
                            KEY_TROY => app_ui.game_selected_troy.trigger(),
                            KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms.trigger(),
                            KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2.trigger(),
                            KEY_WARHAMMER => app_ui.game_selected_warhammer.trigger(),
                            KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia.trigger(),
                            KEY_ATTILA => app_ui.game_selected_attila.trigger(),
                            KEY_ROME_2 => app_ui.game_selected_rome_2.trigger(),
                            KEY_SHOGUN_2 => app_ui.game_selected_shogun_2.trigger(),
                            KEY_NAPOLEON => app_ui.game_selected_napoleon.trigger(),
                            KEY_EMPIRE => app_ui.game_selected_empire.trigger(),
                            KEY_ARENA => app_ui.game_selected_arena.trigger(),
                            _ => unreachable!(),
                        }

                        UI_STATE.set_operational_mode(&app_ui, None);
                        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);
                    }

                    // If we got an error...
                    Response::Error(error) => {
                        show_dialog(&app_ui.main_window, error, false);
                    }

                    // In ANY other situation, it's a message problem.
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                // Always reenable the Main Window.
                app_ui.toggle_main_window(true);
            }
        }));

        // What happens when we trigger the "Change PackFile Type" action.
        let packfile_change_packfile_type = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                info!("Triggering `Change PackFile Type` By Slot");
                // TODO: Replace this with the libs function.
                // Get the currently selected PackFile's Type.
                let packfile_type = match &*(app_ui.change_packfile_type_group
                    .checked_action().text().to_std_string()) {
                    "&Boot" => PFHFileType::Boot,
                    "&Release" => PFHFileType::Release,
                    "&Patch" => PFHFileType::Patch,
                    "&Mod" => PFHFileType::Mod,
                    "Mo&vie" => PFHFileType::Movie,
                    _ => unreachable!()
                };

                // Send the type to the Background Thread, and update the UI.
                let _ = CENTRAL_COMMAND.send_background(Command::SetPackFileType(packfile_type));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // What happens when we change the value of "Include Last Modified Date" action.
        let packfile_index_includes_timestamp = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui =>  move |_| {
                let state = app_ui.change_packfile_type_index_includes_timestamp.is_checked();
                let _ = CENTRAL_COMMAND.send_background(Command::ChangeIndexIncludesTimestamp(state));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // What happens when we enable/disable compression on the current PackFile.
        let packfile_data_is_compressed = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui =>  move |_| {
                let state = app_ui.change_packfile_type_data_is_compressed.is_checked();
                let _ = CENTRAL_COMMAND.send_background(Command::ChangeDataIsCompressed(state));
                UI_STATE.set_is_modified(true, &app_ui, &pack_file_contents_ui);
            }
        ));

        // What happens when we trigger the "Preferences" action.
        let packfile_preferences = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            global_search_ui => move |_| {
                info!("Triggering `Preferences Dialog` By Slot");

                let game_key = GAME_SELECTED.read().unwrap().game_key_name();
                let mymod_path_old = setting_path(MYMOD_BASE_PATH);
                let game_path_old = setting_path(&game_key);
                let ak_path_old = setting_path(&format!("{}_assembly_kit", game_key));

                match SettingsUI::new(&app_ui) {
                    Ok(saved) => {
                        if saved {
                            let mymod_path_new = setting_path(MYMOD_BASE_PATH);
                            let game_path_new = setting_path(&game_key);
                            let ak_path_new = setting_path(&format!("{}_assembly_kit", game_key));

                            // If we changed the "MyMod's Folder" path, disable the MyMod mode and set it so the MyMod menu will be re-built
                            // next time we open the MyMod menu.
                            if mymod_path_old != mymod_path_new {
                                UI_STATE.set_operational_mode(&app_ui, None);
                                AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &diagnostics_ui, &global_search_ui);
                            }

                            // If we have changed the path of any of the games, and that game is the current `GameSelected`,
                            // re-select the current `GameSelected` to force it to reload the game's files.
                            if game_path_old != game_path_new || ak_path_old != ak_path_new {
                                QAction::trigger(&app_ui.game_selected_group.checked_action());
                            }
                        }
                    }
                    Err(error) => show_dialog(&app_ui.main_window, error, false),
                }
            }
        ));

        // What happens when we trigger the "Quit" action.
        let packfile_quit = SlotOfBool::new(&app_ui.main_window, clone!(
            mut app_ui => move |_| {
                app_ui.main_window.close();
            }
        ));

        //-----------------------------------------------//
        // `MyMod` menu logic.
        //-----------------------------------------------//

        // Slot to build the "Open from" submenus of the MyMod menu.
        let mymod_open_menu = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            global_search_ui => move || {
                info!("Triggering `Open MyMod Menu` By Slot");
                AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &diagnostics_ui, &global_search_ui);
            }
        ));

        // What happens when we trigger the "Open MyMod Folder" action.
        let mymod_open_mymod_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            let path = setting_path("mymods_base_path");
            if path.is_dir() {
                let _ = open::that(&path);
            } else {
                show_dialog(&app_ui.main_window, "MyMod path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.", false);
            }
        }));

        // This slot is used for the "New MyMod" action.
        let mymod_new = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            global_search_ui => move |_| {
                info!("Triggering `New MyMod` By Slot");

                // Trigger the `New MyMod` Dialog, and get the result.
                match MyModUI::new(&app_ui) {
                    Ok(dialog) => {
                        if let Some((mod_name, mod_game, sublime_support, vscode_support, paths_ignore_on_import, git_support)) = dialog {
                            let full_mod_name = format!("{}.pack", mod_name);

                            // Change the Game Selected to match the one we chose for the new "MyMod".
                            // NOTE: Arena should not be on this list.
                            match &*mod_game {
                                KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3.trigger(),
                                KEY_TROY => app_ui.game_selected_troy.trigger(),
                                KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms.trigger(),
                                KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2.trigger(),
                                KEY_WARHAMMER => app_ui.game_selected_warhammer.trigger(),
                                KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia.trigger(),
                                KEY_ATTILA => app_ui.game_selected_attila.trigger(),
                                KEY_ROME_2 => app_ui.game_selected_rome_2.trigger(),
                                KEY_SHOGUN_2 => app_ui.game_selected_shogun_2.trigger(),
                                KEY_NAPOLEON => app_ui.game_selected_napoleon.trigger(),
                                KEY_EMPIRE => app_ui.game_selected_empire.trigger(),
                                _ => unimplemented!()
                            }

                            // Disable the main window.
                            app_ui.toggle_main_window(false);

                            // Initialize the folder structure of the MyMod.
                            let receiver = CENTRAL_COMMAND.send_background(Command::InitializeMyModFolder(mod_name, mod_game, sublime_support, vscode_support, git_support));
                            let response = CENTRAL_COMMAND.recv_try(&receiver);
                            match response {
                                Response::PathBuf(mymod_pack_path) => {

                                    // Destroy whatever it's in the file's views and clear the global search UI.
                                    let _ = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, false);
                                    GlobalSearchUI::clear(&global_search_ui);

                                    // Reset the autosave timer.
                                    let timer = setting_int("autosave_interval");
                                    if timer > 0 {
                                        app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
                                        app_ui.timer_backup_autosave.start_0a();
                                    }

                                    // Prepare the settings depending on what we choose to ignore.
                                    let mut pack_settings = initialize_pack_settings();
                                    pack_settings.settings_text_mut().insert("import_files_to_ignore".to_owned(), paths_ignore_on_import);

                                    let _ = CENTRAL_COMMAND.send_background(Command::NewPackFile);
                                    let _ = CENTRAL_COMMAND.send_background(Command::SetPackSettings(pack_settings));
                                    let receiver = CENTRAL_COMMAND.send_background(Command::SavePackFileAs(mymod_pack_path.clone()));
                                    let response = CENTRAL_COMMAND.recv_try(&receiver);
                                    match response {
                                        Response::ContainerInfo(pack_file_info) => {

                                            let mut build_data = BuildData::new();
                                            build_data.editable = true;
                                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);
                                            let packfile_item = pack_file_contents_ui.packfile_contents_tree_model().item_1a(0);
                                            packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                                            packfile_item.set_text(&QString::from_std_str(full_mod_name));

                                            // Set the UI to the state it should be in.
                                            app_ui.change_packfile_type_mod.set_checked(true);
                                            app_ui.change_packfile_type_data_is_encrypted.set_checked(false);
                                            app_ui.change_packfile_type_index_includes_timestamp.set_checked(false);
                                            app_ui.change_packfile_type_index_is_encrypted.set_checked(false);
                                            app_ui.change_packfile_type_header_is_extended.set_checked(false);
                                            app_ui.change_packfile_type_data_is_compressed.set_checked(false);

                                            AppUI::enable_packfile_actions(&app_ui, &PathBuf::from(pack_file_info.file_path()), true);

                                            UI_STATE.set_operational_mode(&app_ui, Some(&mymod_pack_path));
                                            UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

                                            AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &diagnostics_ui, &global_search_ui);
                                            app_ui.toggle_main_window(true);
                                        }

                                        Response::Error(error) => {
                                            app_ui.toggle_main_window(true);
                                            show_dialog(&app_ui.main_window, error, false);
                                        }

                                        // In ANY other situation, it's a message problem.
                                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                                    }
                                }
                                Response::Error(error) => {
                                    app_ui.toggle_main_window(true);
                                    show_dialog(&app_ui.main_window, error, false);
                                }

                                // In ANY other situation, it's a message problem.
                                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                            }
                        }
                    }
                    Err(error) => show_dialog(app_ui.main_window(), error, false),
                }
            }
        ));

        // This slot is used for the "Delete Selected MyMod" action.
        let mymod_delete_selected = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            global_search_ui => move |_| {

                // Ask before doing it, as this will permanently delete the mod from the Disk.
                if AppUI::are_you_sure(&app_ui, true) {
                    info!("Triggering `Delete MyMod` By Slot");

                    // We want to keep our "MyMod" name for the success message, so we store it here.
                    let old_mod_name: String;

                    // Depending on our current "Mode", we choose what to do.
                    let mod_deleted = match UI_STATE.get_operational_mode() {

                        // If we have a "MyMod" selected, and everything we need it's configured,
                        // copy the PackFile to the data folder of the selected game.
                        OperationalMode::MyMod(ref game_folder_name, ref mod_name) => {
                            old_mod_name = mod_name.to_owned();
                            let mymods_base_path = setting_path(MYMOD_BASE_PATH);
                            if mymods_base_path.is_dir() {

                                // We get the "MyMod"s PackFile path.
                                let mut mymod_path = mymods_base_path;
                                mymod_path.push(game_folder_name);
                                mymod_path.push(mod_name);

                                if !mymod_path.is_file() {
                                    return show_dialog(&app_ui.main_window, "The Pack of the selected MyMod doesn't exists, so it can't be installed or removed.", false);
                                }

                                // Try to delete his PackFile. If it fails, return error.
                                if remove_file(&mymod_path).is_err() {
                                    return show_dialog(&app_ui.main_window, "Error deleting the MyMod's Pack.", false);
                                }

                                // Now we get his assets folder.
                                let mut mymod_assets_path = mymod_path.clone();
                                mymod_assets_path.pop();
                                mymod_assets_path.push(mymod_path.file_stem().unwrap().to_string_lossy().as_ref());

                                // We check that path exists. This is optional, so it should allow the deletion
                                // process to continue with a warning.
                                if !mymod_assets_path.is_dir() {
                                    show_dialog(&app_ui.main_window, "The Mod's Pack has been deleted, but his assets folder is nowhere to be found.", false);
                                }

                                // If the assets folder exists, we try to delete it. Again, this is optional, so it should not stop the deleting process.
                                else if remove_dir_all(&mymod_assets_path).is_err() {
                                    show_dialog(&app_ui.main_window, "Error deleting the MyMod's Asset Folder.", false);
                                }

                                // Update the MyMod list and return true, as we have effectively deleted the MyMod.
                                AppUI::build_open_mymod_submenus(&app_ui, &pack_file_contents_ui, &diagnostics_ui, &global_search_ui);
                                true
                            }
                            else { return show_dialog(&app_ui.main_window, "MyMod path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.", false); }
                        }

                        // If we have no "MyMod" selected, return an error.
                        OperationalMode::Normal => return show_dialog(&app_ui.main_window, "You can't delete the selected MyMod if there is no MyMod selected.", false),
                    };

                    // If we deleted the "MyMod", we allow chaos to form below.
                    if mod_deleted {
                        UI_STATE.set_operational_mode(&app_ui, None);
                        let _ = CENTRAL_COMMAND.send_background(Command::ResetPackFile);
                        AppUI::enable_packfile_actions(&app_ui, &PathBuf::new(), false);
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clear, DataSource::PackFile);
                        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

                        show_dialog(&app_ui.main_window, tre("mymod_delete_success", &[&old_mod_name]), true);
                    }
                }
            }
        ));

        let mymod_import = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `Import MyMod` By Slot");
            AppUI::import_mymod(&app_ui, &pack_file_contents_ui);
        }));

        let mymod_export = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
            info!("Triggering `Export MyMod` By Slot");
            AppUI::export_mymod(&app_ui, &pack_file_contents_ui, Some(vec![ContainerPath::Folder("".to_owned())]));
        }));

        //-----------------------------------------------//
        // `View` menu logic.
        //-----------------------------------------------//

        // Initializer for the view actions.
        let view_open_menu = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            global_search_ui,
            dependencies_ui,
            references_ui => move || {
                app_ui.view_toggle_packfile_contents.set_checked(pack_file_contents_ui.packfile_contents_dock_widget().is_visible());
                app_ui.view_toggle_global_search_panel.set_checked(global_search_ui.dock_widget().is_visible());
                app_ui.view_toggle_diagnostics_panel.set_checked(diagnostics_ui.diagnostics_dock_widget().is_visible());
                app_ui.view_toggle_dependencies_panel.set_checked(dependencies_ui.dependencies_dock_widget().is_visible());
                app_ui.view_toggle_references_panel.set_checked(references_ui.references_dock_widget().is_visible());
        }));

        let view_toggle_packfile_contents = SlotOfBool::new(&app_ui.main_window, clone!(
            pack_file_contents_ui => move |state| {
            if !state { pack_file_contents_ui.packfile_contents_dock_widget().hide(); }
            else { pack_file_contents_ui.packfile_contents_dock_widget().show();}
        }));

        let view_toggle_global_search_panel = SlotOfBool::new(&app_ui.main_window, clone!(
            global_search_ui => move |state| {
            if !state { global_search_ui.dock_widget().hide(); }
            else {
                global_search_ui.dock_widget().show();
                global_search_ui.search_line_edit().set_focus_0a()
            }
        }));

        let view_toggle_diagnostics_panel = SlotOfBool::new(&app_ui.main_window, clone!(
            diagnostics_ui => move |state| {
                if !state { diagnostics_ui.diagnostics_dock_widget().hide(); }
                else { diagnostics_ui.diagnostics_dock_widget().show();}
        }));

        let view_toggle_dependencies_panel = SlotOfBool::new(&app_ui.main_window, clone!(
            dependencies_ui => move |state| {
                if !state { dependencies_ui.dependencies_dock_widget().hide(); }
                else { dependencies_ui.dependencies_dock_widget().show();}
        }));

        let view_toggle_references_panel = SlotOfBool::new(&app_ui.main_window, clone!(
            references_ui => move |state| {
                if !state { references_ui.references_dock_widget().hide(); }
                else { references_ui.references_dock_widget().show();}
        }));

        //-----------------------------------------------//
        // `Game Selected` menu logic.
        //-----------------------------------------------//

        // What happens when we trigger the "Launch Game" action.
        let game_selected_launch_game = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            match GAME_SELECTED.read().unwrap().game_launch_command(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                Ok(command) => { let _ = open::that(command); },
                _ => show_dialog(&app_ui.main_window, "The currently selected game cannot be launched from Steam.", false),
            }
        }));

        // What happens when we trigger the "Open Game's Data Folder" action.
        let game_selected_open_game_data_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            if let Ok(path) = GAME_SELECTED.read().unwrap().data_path(&setting_path(&GAME_SELECTED.read().unwrap().game_key_name())) {
                let _ = open::that(path);
            } else {
                show_dialog(&app_ui.main_window, "Game Path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.", false);
            }
        }));

        // What happens when we trigger the "Open Game's Assembly Kit Folder" action.
        let game_selected_open_game_assembly_kit_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            let path = setting_path(&format!("{}_assembly_kit", GAME_SELECTED.read().unwrap().game_key_name()));
            if path.is_dir() {
                let _ = open::that(&path);
            } else {
                show_dialog(&app_ui.main_window, "Game Path not configured. Go to <i>'PackFile/Preferences'</i> and configure it.", false);
            }
        }));

        // What happens when we trigger the "Open Config Folder" action.
        let game_selected_open_config_folder = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            if let Ok(path) = config_path() {
                let _ = open::that(path);
            } else {
                show_dialog(&app_ui.main_window, "RPFM's config folder couldn't be open (maybe it doesn't exists?).", false);
            }
        }));

        // What happens when we trigger the "Change Game Selected" action.
        //
        // NOTE: NEVER EVER AGAIN SHALL YOU TRIGGER HERE A REBUILD OF THE GAME-SPECIFIC SLOTS!!!!!!!!!!
        let change_game_selected = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            dependencies_ui => move |_| {
                info!("Triggering `Change Game Selected` By Slot");
                AppUI::change_game_selected(&app_ui, &pack_file_contents_ui, &dependencies_ui, true);
            }
        ));

        //-----------------------------------------------------//
        // `Special Stuff` menu logic.
        //-----------------------------------------------------//

        // What happens when we trigger the "Generate Dependencies Cache" action.
        let special_stuff_generate_dependencies_cache = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            dependencies_ui => move |_| {
                if AppUI::are_you_sure_edition(&app_ui, "generate_dependencies_cache_are_you_sure") {
                    info!("Triggering `Generate Dependencies Cache` By Slot");

                    if !setting_path(&format!("{}_assembly_kit", GAME_SELECTED.read().unwrap().game_key_name())).is_dir() {
                        show_dialog(&app_ui.main_window, tr("generate_dependencies_cache_warn"), true);
                    }

                    // If there is no problem, ere we go.
                    app_ui.toggle_main_window(false);

                    let wait_dialog = QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                        q_message_box::Icon::Information,
                        &qtr("rpfm_title"),
                        &qtr("generate_dependencies_cache_in_progress_message"),
                        QFlags::from(0),
                        &app_ui.main_window,
                    );

                    wait_dialog.set_attribute_1a(WidgetAttribute::WADeleteOnClose);
                    wait_dialog.set_modal(true);
                    wait_dialog.set_standard_buttons(QFlags::from(0));
                    wait_dialog.show();

                    let receiver = CENTRAL_COMMAND.send_background(Command::GenerateDependenciesCache);
                    let response = CENTRAL_COMMAND.recv_try(&receiver);

                    match response {
                        Response::DependenciesInfo(response) => {
                            let mut parent_build_data = BuildData::new();
                            parent_build_data.data = Some((ContainerInfo::default(), response.parent_packed_files().to_vec()));

                            let mut game_build_data = BuildData::new();
                            game_build_data.data = Some((ContainerInfo::default(), response.vanilla_packed_files().to_vec()));

                            let mut asskit_build_data = BuildData::new();
                            asskit_build_data.data = Some((ContainerInfo::default(), response.asskit_tables().to_vec()));

                            dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(parent_build_data), DataSource::ParentFiles);
                            dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(game_build_data), DataSource::GameFiles);
                            dependencies_ui.dependencies_tree_view().update_treeview(true, TreeViewOperation::Build(asskit_build_data), DataSource::AssKitFiles);

                            wait_dialog.done(1);
                            show_dialog(&app_ui.main_window, tr("generate_dependency_cache_success"), true)
                        },
                        Response::Error(error) => {
                            wait_dialog.done(1);
                            show_dialog(&app_ui.main_window, error, false);
                        },
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    app_ui.toggle_main_window(true);
                }
            }
        ));

        // What happens when we trigger the "Optimize PackFile" action.
        let special_stuff_optimize_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {

                if AppUI::are_you_sure_edition(&app_ui, "optimize_packfile_are_you_sure") {
                    info!("Triggering `Optimize PackFile` By Slot");

                    // If there is no problem, ere we go.
                    app_ui.toggle_main_window(false);

                    if let Err(error) = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, true) {
                        return show_dialog(&app_ui.main_window, error, false);
                    }

                    GlobalSearchUI::clear(&global_search_ui);

                    let receiver = CENTRAL_COMMAND.send_background(Command::OptimizePackFile);
                    let response = CENTRAL_COMMAND.recv_try(&receiver);
                    match response {
                        Response::HashSetString(response) => {
                            let response = response.iter().map(|x| ContainerPath::File(x.to_owned())).collect::<Vec<ContainerPath>>();

                            pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Delete(response), DataSource::PackFile);
                            show_dialog(&app_ui.main_window, tr("optimize_packfile_success"), true);
                        }
                        Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                        _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                    }

                    // Re-enable the Main Window.
                    app_ui.toggle_main_window(true);
                }
            }
        ));

        // What happens when we trigger the "Patch Siege AI" action.
        let special_stuff_patch_siege_ai = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui => move |_| {
                info!("Triggering `Patch SiegeAI` By Slot");

                // Ask the background loop to patch the PackFile, and wait for a response.
                app_ui.toggle_main_window(false);

                if let Err(error) = AppUI::purge_them_all(&app_ui, &pack_file_contents_ui, true) {
                    return show_dialog(&app_ui.main_window, error, false);
                }

                GlobalSearchUI::clear(&global_search_ui);

                let receiver = CENTRAL_COMMAND.send_background(Command::PatchSiegeAI);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::StringVecContainerPath(message, paths) => {
                        pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Delete(paths), DataSource::PackFile);
                        show_dialog(&app_ui.main_window, message, true);
                    }

                    // If the PackFile is empty or is not patchable, report it. Otherwise, praise the nine divines.
                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response)
                }

                // Re-enable the Main Window.
                app_ui.toggle_main_window(true);
            }
        ));

        // What happens when we trigger the "Rescue PackFile" action.
        let special_stuff_rescue_packfile = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |_| {
                if AppUI::are_you_sure_edition(&app_ui, "are_you_sure_rescue_packfile") {
                    info!("Triggering `Rescue PackFile` By Slot");

                    app_ui.toggle_main_window(false);

                    // First, we need to save all open `PackedFiles` to the backend. If one fails, we want to know what one.
                    if let Err(error) = AppUI::back_to_back_end_all(&app_ui, &pack_file_contents_ui) {
                        return show_dialog(&app_ui.main_window, error, false);
                    }

                    // Create the FileDialog to save the PackFile and configure it.
                    let file_dialog = QFileDialog::from_q_widget_q_string(
                        &app_ui.main_window,
                        &qtr("save_packfile"),
                    );
                    file_dialog.set_accept_mode(qt_widgets::q_file_dialog::AcceptMode::AcceptSave);
                    file_dialog.set_name_filter(&QString::from_std_str("PackFiles (*.pack)"));
                    file_dialog.set_confirm_overwrite(true);
                    file_dialog.set_default_suffix(&QString::from_std_str("pack"));

                    // Run it and act depending on the response we get (1 => Accept, 0 => Cancel).
                    if file_dialog.exec() == 1 {
                        let path = PathBuf::from(file_dialog.selected_files().at(0).to_std_string());
                        let file_name = path.file_name().unwrap().to_string_lossy().as_ref().to_owned();
                        let receiver = CENTRAL_COMMAND.send_background(Command::CleanAndSavePackFileAs(path));
                        let response = CENTRAL_COMMAND.recv_try(&receiver);
                        match response {
                            Response::ContainerInfo(pack_file_info) => {
                                let mut build_data = BuildData::new();
                                build_data.editable = true;
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Build(build_data), DataSource::PackFile);
                                pack_file_contents_ui.packfile_contents_tree_view().update_treeview(true, TreeViewOperation::Clean, DataSource::PackFile);

                                let packfile_item = pack_file_contents_ui.packfile_contents_tree_model().item_1a(0);
                                packfile_item.set_tool_tip(&QString::from_std_str(new_pack_file_tooltip(&pack_file_info)));
                                packfile_item.set_text(&QString::from_std_str(file_name));

                                UI_STATE.set_operational_mode(&app_ui, None);
                                UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);
                            }
                            Response::Error(error) => show_dialog(&app_ui.main_window, error, false),

                            // In ANY other situation, it's a message problem.
                            _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                        }
                    }

                    // Then we re-enable the main Window and return whatever we've received.
                    app_ui.toggle_main_window(true);
                }
            }
        ));

        //-----------------------------------------------//
        // `Tools` menu logic.
        //-----------------------------------------------//

        let tools_faction_painter = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui => move || {
                info!("Triggering `Faction Painter Tool` By Slot");
                /*
                app_ui.toggle_main_window(false);
                if let Err(error) = ToolFactionPainter::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui) {
                    show_dialog(&app_ui.main_window, error, false);
                }*/
                app_ui.toggle_main_window(true);
            }
        ));

        let tools_unit_editor = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            global_search_ui,
            diagnostics_ui,
            dependencies_ui => move || {
                info!("Triggering `Unit Editor Tool` By Slot");
                /*
                app_ui.toggle_main_window(false);
                if let Err(error) = ToolUnitEditor::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui) {
                    show_dialog(&app_ui.main_window, error, false);
                }*/
                app_ui.toggle_main_window(true);
            }
        ));

		//-----------------------------------------------//
        // `About` menu logic.
        //-----------------------------------------------//

        // What happens when we trigger the "About Qt" action.
        let about_about_qt = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                QMessageBox::about_qt_1a(&app_ui.main_window);
            }
        ));

        // What happens when we trigger the "About RPFM" action.
        let about_about_rpfm = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                #[cfg(feature = "only_for_the_brave")]
                let only_for_the_brave = ", Only For The Brave";

                #[cfg(not(feature = "only_for_the_brave"))]
                let only_for_the_brave = "";

                QMessageBox::about(
                    &app_ui.main_window,
                    &qtr("about_about_rpfm"),

                    // NOTE: This one is hardcoded, because I don't want people attributing themselves the program in the translations.
                    &QString::from_std_str(format!(
                        "<table>
                            <tr>
                                <td><h2><b>Rusted PackFile Manager</b></h2></td>
                            </tr>
                            <tr>
                                <td>{} {} Patch{}</td>
                            </tr>
                             <tr>
                                <td>Feature flags enabled: {}</td>
                            </tr>
                        </table>

                        <p><b>Rusted PackFile Manager</b> (a.k.a. RPFM) is a modding tool for modern Total War Games, made by modders, for modders.</p>
                        <p>This program is <b>open-source</b>, under MIT License. You can always get the last version (or collaborate) here:</p>
                        <a href=\"https://github.com/Frodo45127/rpfm\">https://github.com/Frodo45127/rpfm</a>
                        <p>This program is also <b>free</b> (if you paid for this, sorry, but you got scammed), but if you want to help with money, here is <b>RPFM's Patreon</b>:</p>
                        <a href=\"https://www.patreon.com/RPFM\">https://www.patreon.com/RPFM</a>

                        <h3>Credits</h3>
                        <ul style=\"list-style-type: disc\">
                            <li>Created and Programmed by: <b>Frodo45127</b>.</li>
                            <li>Extra programming work by: <b>Vandy</b>.</li>
                            <li>Modern DDS Read support by: <b>Phazer</b>.</li>

                            <li>App Icons until v1.6.2 by: <b>Maruka</b>.</li>
                            <li>App Icons since v2.0.0 by: <b>Jake Armitage</b>.</li>

                            <li>AnimPack research: <b>Marthenil</b> and <b>Frodo45127</b>.</li>

                            <li>Ca_vp8 research: <b>John Sirett</b>.</li>

                            <li>LUA functions until v1.6.2 by: <b>Aexrael Dex</b>.</li>
                            <li>LUA Types for Kailua until v1.6.2: <b>DrunkFlamingo</b>.</li>
                            <li>LUA Autogen by: <b>Vandy</b>.</li>

                            <li>RigidModel research by: <b>Mr.Jox</b>, <b>Der Spaten</b>, <b>Maruka</b>, <b>phazer</b> and <b>Frodo45127</b>.</li>
                            <li>RigidModel module until v1.6.2 by: <b>Frodo45127</b>.</li>
                            <li>RigidModel module since v2.4.99 by: <b>Phazer</b>.</li>

                            <li>TW: Arena research and coding: <b>Trolldemorted</b>.</li>
                            <li>TreeView Icons made by <a href=\"https://www.flaticon.com/authors/smashicons\" title=\"Smashicons\">Smashicons</a> from <a href=\"https://www.flaticon.com/\" title=\"Flaticon\">www.flaticon.com</a>. Licensed under <a href=\"http://creativecommons.org/licenses/by/3.0/\" title=\"Creative Commons BY 3.0\" target=\"_blank\">CC 3.0 BY</a>
                        </ul>

                        <h3>Special thanks</h3>
                        <ul style=\"list-style-type: disc\">
                            <li><b>PFM team</b>, for providing the community with awesome modding tools.</li>
                            <li><b>CA</b>, for being a mod-friendly company.</li>
                            <li><b>CnC discord guys</b>, for asking for features, helping with testing from time to time, etc...</li>
                        </ul>
                        ", &VERSION, &VERSION_SUBTITLE, &only_for_the_brave, get_feature_flags()))
                    );
            }
        ));

        // What happens when we trigger the "Open Manual" action.
        let about_open_manual = SlotOfBool::new(&app_ui.main_window, |_| { QDesktopServices::open_url(&QUrl::new_1a(&QString::from_std_str(DOCS_BASE_URL))); });

        // What happens when we trigger the "Support me on Patreon" action.
        let about_patreon_link = SlotOfBool::new(&app_ui.main_window, |_| { QDesktopServices::open_url(&QUrl::new_1a(&QString::from_std_str(PATREON_URL))); });

        // What happens when we trigger the "Check Update" action.
        let about_check_updates = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                info!("Triggering `Check Updates` By Slot");
                AppUI::check_updates(&app_ui, true);
            }
        ));

        // What happens when we trigger the "Check Schema Update" action.
        let about_check_schema_updates = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                info!("Triggering `Check Schema Updates` By Slot");
                AppUI::check_schema_updates(&app_ui, true);
            }
        ));

        // What happens when we trigger the "Check Schema Update" action.
        let about_check_message_updates = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                info!("Triggering `Check Schema Updates` By Slot");
                AppUI::check_message_updates(&app_ui, true);
            }
        ));

        // What happens when we trigger the "Check Schema Update" action.
        let about_check_lua_autogen_updates = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                info!("Triggering `Check Lua Autogen Updates` By Slot");
                AppUI::check_lua_autogen_updates(&app_ui, true);
            }
        ));

        // What happens when we trigger the "Update from AssKit" action.
        let debug_update_current_schema_from_asskit = SlotOfBool::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
                info!("Triggering `Update Current Schema from AssKit` By Slot");

                // If there is no problem, ere we go.
                app_ui.toggle_main_window(false);

                let receiver = CENTRAL_COMMAND.send_background(Command::UpdateCurrentSchemaFromAssKit);
                let response = CENTRAL_COMMAND.recv_try(&receiver);
                match response {
                    Response::Success => show_dialog(&app_ui.main_window, tr("update_current_schema_from_asskit_success"), true),
                    Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                    _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                }

                app_ui.toggle_main_window(true);
            }
        ));

        // What happens when we trigger the "Update from AssKit" action.
        let debug_import_schema_patch = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                info!("Triggering `Import Schema Patch` By Slot");

                // If there is no problem, ere we go.
                app_ui.toggle_main_window(false);

                let dialog = QDialog::new_1a(&app_ui.main_window);
                dialog.set_window_title(&qtr("import_schema_patch_title"));
                dialog.set_modal(true);

                // Create the main Grid.
                let main_grid = create_grid_layout(dialog.static_upcast());
                let patch_text_edit = QTextEdit::from_q_widget(&dialog);
                let import_button = QPushButton::from_q_string_q_widget(&qtr("import_schema_patch_button"), &dialog);
                main_grid.add_widget_5a(&patch_text_edit, 0, 0, 1, 1);
                main_grid.add_widget_5a(&import_button, 1, 0, 1, 1);
                import_button.released().connect(dialog.slot_accept());

                // Center it on screen.
                dialog.resize_2a(1000, 600);

                if dialog.exec() == 1 {
                    match serde_json::from_str(&patch_text_edit.to_plain_text().to_std_string()) {
                        Ok(patch) => {
                            let receiver = CENTRAL_COMMAND.send_background(Command::ImportSchemaPatch(patch));
                            let response = CENTRAL_COMMAND.recv_try(&receiver);
                            match response {
                                Response::Success => show_dialog(&app_ui.main_window, tr("import_schema_patch_success"), true),
                                Response::Error(error) => show_dialog(&app_ui.main_window, error, false),
                                _ => panic!("{}{:?}", THREADS_COMMUNICATION_ERROR, response),
                            }
                        },
                        Err(error) => show_dialog(&app_ui.main_window, error, false),
                    }
                }

                app_ui.toggle_main_window(true);
            }
        ));

        //-----------------------------------------------//
        // `PackedFileView` logic.
        //-----------------------------------------------//
        let packed_file_hide = SlotOfInt::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move |index| {
                AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &[index]);
            }
        ));

        // TODO: This lags the ui on switching tabs. Move to the backend + timer.
        let packed_file_update = SlotOfInt::new(&app_ui.main_window, clone!(
            app_ui => move |index| {
                if index == -1 { return; }
                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let widget = packed_file_view.get_mut_widget();
                    if app_ui.tab_bar_packed_file.index_of(widget) == index {

                        if let ViewType::Internal(View::Table(table)) = packed_file_view.get_view() {

                            // For tables, we have to update the dependency data, reset the dropdown's data, and recheck the entire table for errors.
                            let table = table.get_ref_table();
                            let table_name = if let Some(name) = table.table_name() { name.to_owned() } else { "".to_owned() };
                            if let Ok(data) = get_reference_data(*table.get_packed_file_type(), &table_name, &table.table_definition()) {
                                table.set_dependency_data(&data);

                                setup_item_delegates(
                                    &table.table_view_ptr(),
                                    &table.table_definition(),
                                    &data,
                                    table.timer_delayed_updates()
                                );
                            }
                        }
                        break;
                    }
                }

                // We also have to check for colliding packedfile names, so we can use their full path instead.
                app_ui.update_views_names();

                // Update the background icon.
                GameSelectedIcons::set_game_selected_icon(&app_ui);
            }
        ));

        let packed_file_unpreview = SlotOfInt::new(&app_ui.main_window, clone!(
            app_ui => move |index| {
                if index == -1 { return; }

                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let widget = packed_file_view.get_mut_widget();
                    if app_ui.tab_bar_packed_file.index_of(widget) == index {
                        if packed_file_view.get_is_preview() {
                            packed_file_view.set_is_preview(false);
                            let path = packed_file_view.get_ref_path();
                            let path_split = path.split('/').collect::<Vec<_>>();

                            let name = path_split.last().unwrap().to_owned();
                            app_ui.tab_bar_packed_file.set_tab_text(index, &QString::from_std_str(name));
                        }
                        break;
                    }
                }
            }
        ));

        // Autosave slot.
        let pack_file_backup_autosave = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                info!("Triggering `Autosave` By Slot");

                if !setting_bool("disable_autosaves") {
                    let _ = CENTRAL_COMMAND.send_background(Command::TriggerBackupAutosave);
                    log_to_status_bar(&tr("autosaving"));

                    // Reset the timer.
                    let timer = setting_int("autosave_interval");
                    if timer > 0 {
                        app_ui.timer_backup_autosave.set_interval(timer * 60 * 1000);
                        app_ui.timer_backup_autosave.start_0a();
                    }
                }
            }
        ));

        // When we want to show the context menu.
        let tab_bar_packed_file_context_menu_show = SlotOfQPoint::new(&app_ui.main_window, clone!(
            app_ui => move |_| {
            app_ui.tab_bar_packed_file_context_menu.exec_1a_mut(&QCursor::pos_0a());
        }));

        let tab_bar_packed_file_close = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &[index]);
        }));

        let tab_bar_packed_file_close_all = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            let indexes = UI_STATE.get_open_packedfiles().iter().filter_map(|packed_file_view| {
                let index_to_check = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if index_to_check != index && index_to_check != -1 {
                    Some(index_to_check)
                } else {
                    None
                }
            }).collect::<Vec<i32>>();

            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &indexes);
        }));

        let tab_bar_packed_file_close_all_left = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            let indexes = UI_STATE.get_open_packedfiles().iter().filter_map(|packed_file_view| {
                let index_to_check = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if index_to_check < index {
                    Some(index_to_check)
                } else {
                    None
                }
            }).collect::<Vec<i32>>();
            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &indexes);
        }));

        let tab_bar_packed_file_close_all_right = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui => move || {
            let index = app_ui.tab_bar_packed_file.current_index();
            let indexes = UI_STATE.get_open_packedfiles().iter().filter_map(|packed_file_view| {
                let index_to_check = app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget());
                if index_to_check > index {
                    Some(index_to_check)
                } else {
                    None
                }
            }).collect::<Vec<i32>>();
            AppUI::packed_file_view_hide(&app_ui, &pack_file_contents_ui, &indexes);
        }));

        let tab_bar_packed_file_prev = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                let index = app_ui.tab_bar_packed_file.current_index();
                if index != -1 {
                    app_ui.tab_bar_packed_file.set_current_index(index - 1);
                }
            }
        ));

        let tab_bar_packed_file_next = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                let index = app_ui.tab_bar_packed_file.current_index();
                if index != -1 {
                    app_ui.tab_bar_packed_file.set_current_index(index + 1);
                }
            }
        ));

        let tab_bar_packed_file_import_from_dependencies = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui,
            pack_file_contents_ui,
            dependencies_ui => move || {
                info!("Triggering `Import from Dependencies` By Slot");

                // Only allow importing if we currently have a PackFile open.
                if pack_file_contents_ui.packfile_contents_tree_model().row_count_0a() > 0 {

                    // What this does:
                    // - Get the data source and path of the open file.
                    // - Import it into our mod.
                    // - Change the data source of the view to PackFile, so we can reuse the view.
                    let index = app_ui.tab_bar_packed_file.current_index();
                    if index != -1 {
                        let mut paths_by_source = BTreeMap::new();
                        let data_source_and_path = if let Some(packed_file_view) = UI_STATE.get_open_packedfiles().iter().find(|packed_file_view| {
                            index == app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget())
                        }) {
                            let path = packed_file_view.get_ref_path();
                            let data_source = packed_file_view.get_data_source();
                            paths_by_source.insert(data_source, vec![ContainerPath::File(path.to_owned())]);
                            Some((data_source, path.to_owned()))
                        } else { None };

                        // The backend already checks for proper data source. No need to double-check here.
                        if let Some((data_source, path)) = data_source_and_path {
                            dependencies_ui.import_dependencies(paths_by_source, &app_ui, &pack_file_contents_ui);

                            // TODO: MAke sure this uses the correct source.
                            let path_to_purge = UI_STATE.get_open_packedfiles().iter().find_map(|packed_file_view| {
                                if *packed_file_view.get_ref_path() == path && packed_file_view.get_data_source() == DataSource::PackFile {
                                    Some(packed_file_view.get_ref_path().to_owned())
                                } else { None }
                            });

                            // If we're overwriting a PackedFile already on our PackFile, remove it.
                            if let Some(path_to_purge) = path_to_purge {
                                let _  = AppUI::purge_that_one_specifically(&app_ui, &pack_file_contents_ui, &path_to_purge, DataSource::PackFile, false);
                            }

                            if let Some(packed_file_view) = UI_STATE.set_open_packedfiles().iter_mut().find(|packed_file_view| {
                                index == app_ui.tab_bar_packed_file.index_of(packed_file_view.get_mut_widget())
                            }) {
                                packed_file_view.set_data_source(DataSource::PackFile);
                                if let Err(error) = packed_file_view.reload(&path, &pack_file_contents_ui) {
                                    show_dialog(&app_ui.main_window, &error, false);
                                }

                            }
                        }
                    }
                    app_ui.update_views_names();
                }
            }
        ));

        let tab_bar_packed_file_toggle_tips = SlotNoArgs::new(&app_ui.main_window, clone!(
            app_ui => move || {
                let index = app_ui.tab_bar_packed_file.current_index();
                if index == -1 { return; }

                for packed_file_view in UI_STATE.get_open_packedfiles().iter() {
                    let widget = packed_file_view.get_mut_widget();
                    if app_ui.tab_bar_packed_file.index_of(widget) == index {

                        // Re-add the widget with the correct row span before making it visible.
                        if !packed_file_view.get_tips_widget().is_visible() {
                            let layout: QPtr<QGridLayout> = packed_file_view.get_mut_widget().layout().static_downcast();
                            layout.add_widget_5a(packed_file_view.get_tips_widget(), 0, 99, layout.row_count(), 1);
                            packed_file_view.get_tips_widget().set_minimum_width(350);
                            packed_file_view.get_tips_widget().set_maximum_width(350);
                        }

                        packed_file_view.get_tips_widget().set_visible(!packed_file_view.get_tips_widget().is_visible());
                        break;
                    }
                }
            }
        ));

        // And here... we return all the slots.
		Self {

            //-----------------------------------------------//
            // `PackFile` menu slots.
            //-----------------------------------------------//
            packfile_open_menu,
            packfile_new_packfile,
            packfile_open_packfile,
            packfile_save_packfile,
            packfile_save_packfile_as,
            packfile_install,
            packfile_uninstall,
            packfile_load_all_ca_packfiles,
            packfile_change_packfile_type,
            packfile_index_includes_timestamp,
            packfile_data_is_compressed,
            packfile_preferences,
            packfile_quit,

            //-----------------------------------------------//
            // `MyMod` menu slots.
            //-----------------------------------------------//
            mymod_open_menu,
            mymod_open_mymod_folder,
            mymod_new,
            mymod_delete_selected,
            mymod_import,
            mymod_export,

            //-----------------------------------------------//
            // `View` menu slots.
            //-----------------------------------------------//
            view_open_menu,
            view_toggle_packfile_contents,
            view_toggle_global_search_panel,
            view_toggle_diagnostics_panel,
            view_toggle_dependencies_panel,
            view_toggle_references_panel,

            //-----------------------------------------------//
            // `Game Selected` menu slots.
            //-----------------------------------------------//
            game_selected_launch_game,
            game_selected_open_game_data_folder,
            game_selected_open_game_assembly_kit_folder,
            game_selected_open_config_folder,
            change_game_selected,

            //-----------------------------------------------//
            // `Special Stuff` menu slots.
            //-----------------------------------------------//
            special_stuff_generate_dependencies_cache,
            special_stuff_optimize_packfile,
            special_stuff_patch_siege_ai,
            special_stuff_rescue_packfile,

            //-----------------------------------------------//
            // `Tools` menu slots.
            //-----------------------------------------------//
            tools_faction_painter,
            tools_unit_editor,

    		//-----------------------------------------------//
	        // `About` menu slots.
	        //-----------------------------------------------//
    		about_about_qt,
            about_about_rpfm,
            about_open_manual,
            about_patreon_link,
            about_check_updates,
            about_check_schema_updates,
            about_check_message_updates,
            about_check_lua_autogen_updates,

            //-----------------------------------------------//
            // `Debug` menu slots.
            //-----------------------------------------------//
            debug_update_current_schema_from_asskit,
            debug_import_schema_patch,

            //-----------------------------------------------//
            // `PackedFileView` slots.
            //-----------------------------------------------//
            packed_file_hide,
            packed_file_update,
            packed_file_unpreview,

            //-----------------------------------------------//
            // `Generic` slots.
            //-----------------------------------------------//
            pack_file_backup_autosave,

            tab_bar_packed_file_context_menu_show,
            tab_bar_packed_file_close,
            tab_bar_packed_file_close_all,
            tab_bar_packed_file_close_all_left,
            tab_bar_packed_file_close_all_right,
            tab_bar_packed_file_prev,
            tab_bar_packed_file_next,
            tab_bar_packed_file_import_from_dependencies,
            tab_bar_packed_file_toggle_tips,
		}
	}
}

impl AppUITempSlots {
    pub unsafe fn build(
        app_ui: &Rc<AppUI>,
        pack_file_contents_ui: &Rc<PackFileContentsUI>,
        global_search_ui: &Rc<GlobalSearchUI>,
        diagnostics_ui: &Rc<DiagnosticsUI>,
    ) {
        AppUI::build_open_from_submenus(app_ui, pack_file_contents_ui, global_search_ui, diagnostics_ui);
        AppUI::build_open_mymod_submenus(app_ui, pack_file_contents_ui, diagnostics_ui, global_search_ui);
    }
}

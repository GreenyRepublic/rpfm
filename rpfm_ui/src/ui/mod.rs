//---------------------------------------------------------------------------//
// Copyright (c) 2017-2023 Ismael Gutiérrez González. All rights reserved.
//
// This file is part of the Rusted PackFile Manager (RPFM) project,
// which can be found here: https://github.com/Frodo45127/rpfm.
//
// This file is licensed under the MIT license, which can be found here:
// https://github.com/Frodo45127/rpfm/blob/master/LICENSE.
//---------------------------------------------------------------------------//

/*!
Module with all the code for managing the UI.

This module contains the code to manage the main UI and store all his slots.
!*/

use qt_widgets::QApplication;

#[cfg(feature = "only_for_the_brave")]
use qt_widgets::QMessageBox;

#[cfg(feature = "only_for_the_brave")]
use qt_widgets::q_message_box::Icon;

#[cfg(feature = "only_for_the_brave")]
use qt_widgets::q_message_box::StandardButton;

use qt_gui::QFont;
use qt_gui::QIcon;

use qt_core::QFlags;
use qt_core::QString;
use qt_core::WindowState;

use anyhow::Result;

use std::env::args;
use std::path::PathBuf;
use std::rc::Rc;
use std::fs::{read_dir, remove_dir_all};
use std::sync::atomic::AtomicPtr;

use rpfm_lib::games::supported_games::*;
use rpfm_lib::integrations::log::*;

#[cfg(feature = "only_for_the_brave")]
use crate::VERSION;
use crate::app_ui;
use crate::app_ui::AppUI;
use crate::app_ui::slots::{AppUITempSlots, AppUISlots};
use crate::ASSETS_PATH;
use crate::dependencies_ui;
use crate::dependencies_ui::DependenciesUI;
use crate::dependencies_ui::slots::DependenciesUISlots;
use crate::diagnostics_ui;
use crate::diagnostics_ui::DiagnosticsUI;
use crate::diagnostics_ui::slots::DiagnosticsUISlots;
use crate::GAME_SELECTED;
use crate::GAME_SELECTED_ICONS;
use crate::global_search_ui;
use crate::global_search_ui::GlobalSearchUI;
use crate::global_search_ui::slots::GlobalSearchSlots;
use crate::references_ui;
use crate::references_ui::ReferencesUI;
use crate::references_ui::slots::ReferencesUISlots;
use crate::SUPPORTED_GAMES;

#[cfg(feature = "only_for_the_brave")]
use crate::locale::qtr;

use crate::packfile_contents_ui::PackFileContentsUI;
use crate::packfile_contents_ui;
use crate::packfile_contents_ui::slots::PackFileContentsSlots;
use crate::RPFM_PATH;
use crate::settings_ui::backend::*;
use crate::UI_STATE;
use crate::utils::*;

//-------------------------------------------------------------------------------//
//                              Enums & Structs
//-------------------------------------------------------------------------------//

/// This struct contains all the pointers we need to access to EVERY widget/action created at the start of the program.
///
/// This means every widget/action that's created on start (menus, the TreeView,...) should be here.
pub struct UI {
    pub app_ui: Rc<AppUI>,
    pub pack_file_contents_ui: Rc<PackFileContentsUI>,
    pub global_search_ui: Rc<GlobalSearchUI>,
    pub diagnostics_ui: Rc<DiagnosticsUI>,
    pub dependencies_ui: Rc<DependenciesUI>,
}

/// This struct is used to hold all the Icons used for the window's titlebar.
pub struct GameSelectedIcons {
    pub warhammer_3: (AtomicPtr<QIcon>, String),
    pub troy: (AtomicPtr<QIcon>, String),
    pub three_kingdoms: (AtomicPtr<QIcon>, String),
    pub warhammer_2: (AtomicPtr<QIcon>, String),
    pub warhammer: (AtomicPtr<QIcon>, String),
    pub thrones_of_britannia: (AtomicPtr<QIcon>, String),
    pub attila: (AtomicPtr<QIcon>, String),
    pub rome_2: (AtomicPtr<QIcon>, String),
    pub shogun_2: (AtomicPtr<QIcon>, String),
    pub napoleon: (AtomicPtr<QIcon>, String),
    pub empire: (AtomicPtr<QIcon>, String),
    pub arena: (AtomicPtr<QIcon>, String),
}

//-------------------------------------------------------------------------------//
//                             Implementations
//-------------------------------------------------------------------------------//

/// Implementation of `UI`.
impl UI {

    /// This function initialize the entire `UI`.
    pub unsafe fn new() -> Result<Self> {
        let t = std::time::SystemTime::now();

        let app_ui = Rc::new(AppUI::new());
        let global_search_ui = Rc::new(GlobalSearchUI::new(app_ui.main_window())?);
        let pack_file_contents_ui = Rc::new(PackFileContentsUI::new(&app_ui)?);
        let diagnostics_ui = Rc::new(DiagnosticsUI::new(app_ui.main_window())?);
        let dependencies_ui = Rc::new(DependenciesUI::new(&app_ui)?);
        let references_ui = Rc::new(ReferencesUI::new(app_ui.main_window())?);

        AppUITempSlots::build(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui);

        let app_slots = AppUISlots::new(&app_ui, &global_search_ui, &pack_file_contents_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let pack_file_contents_slots = PackFileContentsSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let global_search_slots = GlobalSearchSlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let diagnostics_slots = DiagnosticsUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let dependencies_slots = DependenciesUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);
        let references_slots = ReferencesUISlots::new(&app_ui, &pack_file_contents_ui, &global_search_ui, &diagnostics_ui, &dependencies_ui, &references_ui);

        app_ui::connections::set_connections(&app_ui, &app_slots);
        app_ui::tips::set_tips(&app_ui);

        global_search_ui::connections::set_connections(&global_search_ui, &global_search_slots);
        global_search_ui::tips::set_tips(&global_search_ui);

        packfile_contents_ui::connections::set_connections(&pack_file_contents_ui, &pack_file_contents_slots);
        packfile_contents_ui::tips::set_tips(&pack_file_contents_ui);

        dependencies_ui::connections::set_connections(&dependencies_ui, &dependencies_slots);
        dependencies_ui::tips::set_tips(&dependencies_ui);

        diagnostics_ui::connections::set_connections(&diagnostics_ui, &diagnostics_slots);
        references_ui::connections::set_connections(&references_ui, &references_slots);

        // Initialize settings.
        init_settings(&app_ui);

        // Apply last ui state.
        app_ui.main_window().restore_geometry(&setting_byte_array("geometry"));
        app_ui.main_window().restore_state_1a(&setting_byte_array("windowState"));

        // Apply the font.
        let font_name = setting_string("font_name");
        let font_size = setting_int("font_size");
        let font = QFont::from_q_string_int(&QString::from_std_str(font_name), font_size);
        QApplication::set_font_1a(&font);

        // Here we also initialize the UI.
        UI_STATE.set_operational_mode(&app_ui, None);
        UI_STATE.set_is_modified(false, &app_ui, &pack_file_contents_ui);

        // If we want the window to start maximized...
        if setting_bool("start_maximized") {
            app_ui.main_window().set_window_state(QFlags::from(WindowState::WindowMaximized));
        }

        reload_theme();

        // Show the Main Window...
        app_ui.main_window().show();
        log_to_status_bar("Initializing, please wait...");

        // Do not trigger the automatic game changed signal here, as that will trigger an expensive and useless dependency rebuild.
        info!("Setting initial Game Selected…");
        match &*setting_string("default_game") {
            KEY_WARHAMMER_3 => app_ui.game_selected_warhammer_3().set_checked(true),
            KEY_TROY => app_ui.game_selected_troy().set_checked(true),
            KEY_THREE_KINGDOMS => app_ui.game_selected_three_kingdoms().set_checked(true),
            KEY_WARHAMMER_2 => app_ui.game_selected_warhammer_2().set_checked(true),
            KEY_WARHAMMER => app_ui.game_selected_warhammer().set_checked(true),
            KEY_THRONES_OF_BRITANNIA => app_ui.game_selected_thrones_of_britannia().set_checked(true),
            KEY_ATTILA => app_ui.game_selected_attila().set_checked(true),
            KEY_ROME_2 => app_ui.game_selected_rome_2().set_checked(true),
            KEY_SHOGUN_2 => app_ui.game_selected_shogun_2().set_checked(true),
            KEY_NAPOLEON => app_ui.game_selected_napoleon().set_checked(true),
            KEY_EMPIRE => app_ui.game_selected_empire().set_checked(true),
            KEY_ARENA  => app_ui.game_selected_arena().set_checked(true),

            // Turns out some... lets say "not very bright individual" changed the settings file manually and broke this.
            // So just in case, by default we use WH3.
            _ => app_ui.game_selected_warhammer_3().set_checked(true),
        }
dbg!(t.elapsed().unwrap());
        AppUI::change_game_selected(&app_ui, &pack_file_contents_ui, &dependencies_ui, true);
        info!("Initial Game Selected set to {}.", setting_string("default_game"));
dbg!(t.elapsed().unwrap());
        // We get all the Arguments provided when starting RPFM, just in case we passed it a path,
        // in which case, we automatically try to open it.
        let args = args().collect::<Vec<String>>();
        if args.len() > 1 {
            let paths = args[1..].iter().map(PathBuf::from).filter(|path| path.is_file()).collect::<Vec<_>>();

            info!("Directly opening Pack/s {:?}.", paths);
            if let Err(error) = AppUI::open_packfile(&app_ui, &pack_file_contents_ui, &global_search_ui, &paths, "") {
                show_dialog(app_ui.main_window(), error, false);
            } else if setting_bool("diagnostics_trigger_on_open") {
                DiagnosticsUI::check(&app_ui, &diagnostics_ui);
            }
        }
dbg!(t.elapsed().unwrap());

        // If we have it enabled in the prefs, check if there are updates.
        if setting_bool("check_updates_on_start") { AppUI::check_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are schema updates.
        if setting_bool("check_schema_updates_on_start") { AppUI::check_schema_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are message updates.
        if setting_bool("check_message_updates_on_start") { AppUI::check_message_updates(&app_ui, false) };

        // If we have it enabled in the prefs, check if there are lua autogen updates.
        if setting_bool("check_lua_autogen_updates_on_start") { AppUI::check_lua_autogen_updates(&app_ui, false) };

        // Clean up folders from previous updates, if they exist.
        if !cfg!(debug_assertions) {
            if let Ok(folders) = read_dir(&*RPFM_PATH) {
                for folder in folders.flatten() {
                    let folder_path = folder.path();
                    if folder_path.is_dir() && folder_path.file_name().unwrap().to_string_lossy().starts_with("update") {
                        let _ = remove_dir_all(&folder_path);
                    }
                }
                info!("Update folders cleared.");
            }
        }
dbg!(t.elapsed().unwrap());
        // Show the "only for the brave" alert for specially unstable builds.
        #[cfg(feature = "only_for_the_brave")] {
            let first_boot_setting = "firstBoot".to_owned() + VERSION;
            if !setting_bool(&first_boot_setting) {

                let title = qtr("title_only_for_the_brave");
                let message = qtr("message_only_for_the_brave");
                QMessageBox::from_icon2_q_string_q_flags_standard_button_q_widget(
                    Icon::Warning,
                    &title,
                    &message,
                    QFlags::from(StandardButton::Ok),
                    app_ui.main_window(),
                ).exec();

                // Set it so it doesn't popup again for this version.
                set_setting_bool(&first_boot_setting, true);
            }
        }
dbg!(t.elapsed().unwrap());
        info!("Initialization complete.");
        Ok(Self {
            app_ui,
            global_search_ui,
            pack_file_contents_ui,
            diagnostics_ui,
            dependencies_ui
        })
    }
}

/// Implementation of `GameSelectedIcons`.
impl GameSelectedIcons {

    /// This function loads to memory the icons of all the supported games.
    pub unsafe fn new() -> Self {
        Self {
            warhammer_3: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_3).unwrap().icon_big_file_name())),
            troy: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_TROY).unwrap().icon_big_file_name())),
            three_kingdoms: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THREE_KINGDOMS).unwrap().icon_big_file_name())),
            warhammer_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER_2).unwrap().icon_big_file_name())),
            warhammer: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_WARHAMMER).unwrap().icon_big_file_name())),
            thrones_of_britannia: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_THRONES_OF_BRITANNIA).unwrap().icon_big_file_name())),
            attila: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ATTILA).unwrap().icon_big_file_name())),
            rome_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ROME_2).unwrap().icon_big_file_name())),
            shogun_2: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_SHOGUN_2).unwrap().icon_big_file_name())),
            napoleon: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_NAPOLEON).unwrap().icon_big_file_name())),
            empire: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_EMPIRE).unwrap().icon_big_file_name())),
            arena: (atomic_from_cpp_box(QIcon::from_q_string(&QString::from_std_str(format!("{}/icons/{}",ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_file_name())))), format!("{}/icons/{}", ASSETS_PATH.to_string_lossy(), SUPPORTED_GAMES.game(KEY_ARENA).unwrap().icon_big_file_name())),
        }
    }

    /// This function sets the main window icon according to the currently selected game.
    pub unsafe fn set_game_selected_icon(app_ui: &Rc<AppUI>) {
        let (icon, big_icon) = match &*GAME_SELECTED.read().unwrap().game_key_name() {
            KEY_WARHAMMER_3 => &GAME_SELECTED_ICONS.warhammer_3,
            KEY_TROY => &GAME_SELECTED_ICONS.troy,
            KEY_THREE_KINGDOMS => &GAME_SELECTED_ICONS.three_kingdoms,
            KEY_WARHAMMER_2 => &GAME_SELECTED_ICONS.warhammer_2,
            KEY_WARHAMMER => &GAME_SELECTED_ICONS.warhammer,
            KEY_THRONES_OF_BRITANNIA => &GAME_SELECTED_ICONS.thrones_of_britannia,
            KEY_ATTILA => &GAME_SELECTED_ICONS.attila,
            KEY_ROME_2 => &GAME_SELECTED_ICONS.rome_2,
            KEY_SHOGUN_2 => &GAME_SELECTED_ICONS.shogun_2,
            KEY_NAPOLEON => &GAME_SELECTED_ICONS.napoleon,
            KEY_EMPIRE => &GAME_SELECTED_ICONS.empire,
            KEY_ARENA => &GAME_SELECTED_ICONS.arena,
            _ => unimplemented!(),
        };
        app_ui.main_window().set_window_icon(ref_from_atomic(icon));

        // Fix due to windows paths.
        let big_icon = if cfg!(target_os = "windows") {  big_icon.replace('\\', "/") } else { big_icon.to_owned() };

        if !setting_bool("hide_background_icon") {
            if app_ui.tab_bar_packed_file().count() == 0 {

                // WTF of the day: without the border line, this doesn't work on windows. Who knows why...?
                let border =  if cfg!(target_os = "windows") { "border: 0px solid #754EF9;" } else { "" };
                app_ui.tab_bar_packed_file().set_style_sheet(&QString::from_std_str(format!("
                    QTabWidget::pane {{
                        background-image: url('{}');
                        background-repeat: no-repeat;
                        background-position: center;
                        {}
                    }}
                ", big_icon, border)));
            }
            else {

                // This is laggy after a while.
                app_ui.tab_bar_packed_file().set_style_sheet(&QString::from_std_str("QTabWidget::pane {background-image: url();}"));
            }
        }
    }
}

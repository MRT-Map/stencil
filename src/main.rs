#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(
    clippy::type_complexity,
    clippy::too_many_arguments,
    clippy::missing_panics_doc
)]
#![warn(
    clippy::as_ptr_cast_mut,
    clippy::as_underscore,
    clippy::bool_to_int_with_if,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_lossless,
    clippy::cast_possible_wrap,
    clippy::checked_conversions,
    clippy::clear_with_drain,
    clippy::clone_on_ref_ptr,
    clippy::cloned_instead_of_copied,
    clippy::cognitive_complexity,
    clippy::collection_is_never_read,
    clippy::copy_iterator,
    clippy::create_dir,
    clippy::default_trait_access,
    clippy::deref_by_slicing,
    clippy::doc_link_with_quotes,
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::empty_line_after_outer_attr,
    clippy::empty_structs_with_brackets,
    clippy::enum_glob_use,
    clippy::equatable_if_let,
    clippy::exit,
    clippy::expl_impl_clone_on_copy,
    clippy::explicit_deref_methods,
    clippy::explicit_into_iter_loop,
    clippy::explicit_iter_loop,
    clippy::filetype_is_file,
    clippy::filter_map_next,
    clippy::flat_map_option,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::fn_params_excessive_bools,
    clippy::fn_to_numeric_cast_any,
    clippy::from_iter_instead_of_collect,
    clippy::future_not_send,
    clippy::get_unwrap,
    clippy::if_not_else,
    clippy::if_then_some_else_none,
    clippy::implicit_hasher,
    clippy::impl_trait_in_params,
    clippy::imprecise_flops,
    clippy::inconsistent_struct_constructor,
    clippy::index_refutable_slice,
    clippy::inefficient_to_string,
    clippy::invalid_upcast_comparisons,
    clippy::items_after_statements,
    clippy::iter_not_returning_iterator,
    clippy::iter_on_empty_collections,
    clippy::iter_on_single_items,
    clippy::iter_with_drain,
    clippy::large_digit_groups,
    clippy::large_futures,
    clippy::large_stack_arrays,
    clippy::large_types_passed_by_value,
    clippy::linkedlist,
    clippy::lossy_float_literal,
    clippy::manual_assert,
    clippy::manual_clamp,
    clippy::manual_instant_elapsed,
    clippy::manual_let_else,
    clippy::manual_ok_or,
    clippy::manual_string_new,
    clippy::many_single_char_names,
    clippy::map_err_ignore,
    clippy::map_unwrap_or,
    clippy::match_on_vec_items,
    clippy::mismatching_type_param_order,
    clippy::missing_assert_message,
    clippy::missing_const_for_fn,
    clippy::missing_enforced_import_renames,
    clippy::multiple_unsafe_ops_per_block,
    clippy::must_use_candidate,
    clippy::mut_mut,
    clippy::naive_bytecount,
    clippy::needless_bitwise_bool,
    clippy::needless_collect,
    clippy::needless_continue,
    clippy::needless_for_each,
    clippy::needless_pass_by_value,
    clippy::negative_feature_names,
    clippy::non_ascii_literal,
    clippy::non_send_fields_in_send_ty,
    clippy::or_fun_call,
    clippy::range_minus_one,
    clippy::range_plus_one,
    clippy::rc_buffer,
    clippy::redundant_closure_for_method_calls,
    clippy::redundant_else,
    clippy::redundant_feature_names,
    clippy::redundant_pub_crate,
    clippy::ref_option_ref,
    clippy::ref_patterns,
    clippy::rest_pat_in_fully_bound_structs,
    clippy::return_self_not_must_use,
    clippy::same_functions_in_if_condition,
    clippy::semicolon_if_nothing_returned,
    clippy::semicolon_inside_block,
    clippy::separated_literal_suffix,
    /*clippy::significant_drop_in_scrutinee,
    clippy::significant_drop_tightening,*/
    clippy::single_match_else,
    clippy::str_to_string,
    clippy::string_add,
    clippy::string_add_assign,
    clippy::string_slice,
    clippy::struct_excessive_bools,
    clippy::suboptimal_flops,
    clippy::suspicious_operation_groupings,
    clippy::suspicious_xor_used_as_pow,
    clippy::tests_outside_test_module,
    clippy::trailing_empty_array,
    clippy::trait_duplication_in_bounds,
    clippy::transmute_ptr_to_ptr,
    clippy::transmute_undefined_repr,
    clippy::trivial_regex,
    clippy::trivially_copy_pass_by_ref,
    clippy::try_err,
    clippy::type_repetition_in_bounds,
    clippy::unchecked_duration_subtraction,
    clippy::undocumented_unsafe_blocks,
    clippy::unicode_not_nfc,
    clippy::uninlined_format_args,
    clippy::unnecessary_box_returns,
    clippy::unnecessary_join,
    clippy::unnecessary_safety_comment,
    clippy::unnecessary_safety_doc,
    clippy::unnecessary_self_imports,
    clippy::unnecessary_struct_initialization,
    clippy::unneeded_field_pattern,
    clippy::unnested_or_patterns,
    clippy::unreadable_literal,
    clippy::unsafe_derive_deserialize,
    clippy::unused_async,
    clippy::unused_peekable,
    clippy::unused_rounding,
    clippy::unused_self,
    clippy::unwrap_in_result,
    clippy::use_self,
    clippy::useless_let_if_seq,
    clippy::verbose_bit_mask,
    clippy::verbose_file_reads
)]
#![deny(
    clippy::derive_partial_eq_without_eq,
    clippy::match_bool,
    clippy::mem_forget,
    clippy::mutex_atomic,
    clippy::mutex_integer,
    clippy::nonstandard_macro_braces,
    clippy::path_buf_push_overwrite,
    clippy::rc_mutex,
    clippy::wildcard_dependencies
)]

use bevy::{
    asset::AssetPlugin,
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::LogPlugin,
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuSettings},
        RenderPlugin,
    },
};
use bevy_egui::EguiPlugin;
use bevy_mod_picking::DefaultPickingPlugins;
use bevy_mouse_tracking::prelude::MousePosPlugin;
use bevy_prototype_lyon::prelude::ShapePlugin;
use tracing::Level;
use tracing_error::ErrorLayer;
use tracing_subscriber::{
    fmt::writer::MakeWriterExt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use ui::tilemap::RenderingPlugin;

#[cfg(target_os = "linux")]
use crate::window_settings::settings::LinuxWindow;
use crate::{
    component_actions::ComponentActionPlugins,
    component_tools::ComponentToolPlugins,
    hotkeys::HotkeyPlugin,
    info_windows::InfoWindowsPlugin,
    init::InitPlugin,
    load_save::LoadSavePlugin,
    misc::data_dir,
    ui::UiPlugin,
    window_settings::{settings::INIT_WINDOW_SETTINGS, WindowSettingsPlugin},
};

pub mod component_actions;
pub mod component_tools;
pub mod error_handling;
pub mod hotkeys;
pub mod info_windows;
pub mod init;
pub mod load_save;
pub mod misc;
pub mod pla2;
pub mod state;
pub mod tile;
pub mod ui;
pub mod window_settings;

fn init_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer().compact().with_writer(
                std::io::stdout
                    .with_max_level(Level::DEBUG)
                    .and(tracing_appender::rolling::hourly(data_dir("logs"), "log")),
            ),
        )
        .with(if let Ok(l) = EnvFilter::try_from_default_env() {
            l
        } else {
            EnvFilter::try_new(
                "info,\
            wgpu_core::device=warn,\
            bevy_asset::asset_server=error,\
            surf::middleware::logger::native=off,\
            isahc::handler=error,\
            stencil2=debug",
            )
            .unwrap()
        })
        .with(ErrorLayer::default())
        .init();
}

fn main() {
    std::panic::set_hook(Box::new(error_handling::panic));

    init_logger();
    info!("Logger initialised");

    #[cfg(target_os = "linux")]
    match INIT_WINDOW_SETTINGS.display_server_protocol {
        LinuxWindow::Xorg => std::env::set_var("WINIT_UNIX_BACKEND", "x11"),
        LinuxWindow::Wayland => std::env::set_var("WINIT_UNIX_BACKEND", "wayland"),
        LinuxWindow::Auto => (),
    }

    let mut app = App::new();
    app.add_plugins({
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Stencil".into(),
                    mode: INIT_WINDOW_SETTINGS.window_mode,
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest())
            .set(AssetPlugin {
                file_path: data_dir("assets").to_string_lossy().to_string(),
                ..default()
            })
            .set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    backends: Some(INIT_WINDOW_SETTINGS.backends.into()),
                    ..default()
                }),
                ..default()
            })
            .disable::<LogPlugin>()
    })
    .add_plugins(FrameTimeDiagnosticsPlugin);

    app.add_plugins(DefaultPickingPlugins)
        .add_plugins(MousePosPlugin)
        .add_plugins(EguiPlugin)
        .add_plugins(ShapePlugin);

    app.add_plugins(InitPlugin)
        .add_plugins(UiPlugin)
        .add_plugins(RenderingPlugin)
        .add_plugins(ComponentToolPlugins)
        .add_plugins(ComponentActionPlugins)
        .add_plugins(LoadSavePlugin)
        .add_plugins(InfoWindowsPlugin)
        .add_plugins(HotkeyPlugin)
        .add_plugins(WindowSettingsPlugin)
        .run();
}

//! Windows: the Maillet window, settings, about and the OTP code popup.
//!
//! The Maillet window has two forms, chosen in the settings (General → Window); the choice holds for
//! tray click, global shortcut and every other way of opening (kit PLAN K9):
//! - **Popup**: next to the tray icon, fixed size, hides when clicking elsewhere unless pinned.
//! - **Sidebar**: full height at the edge of the screen half with the tray icon (or the edge the
//!   user picked); it stays when clicking elsewhere, but is kept above other windows only when
//!   pinned; the tray icon or the shortcut bring it back to the front.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent};

pub const MAIN: &str = "main";
/// Fixed window size (logical px): enough to read a mail comfortably. It is
/// both default and minimum; features the user turns on may add to it later.
/// The size never changes with the content – no resizing, no flicker.
const WIDTH: f64 = 700.0;
const HEIGHT: f64 = 700.0;
/// Gap between window and tray icon / screen edge.
const MARGIN: i32 = 8;

#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Form {
    #[default]
    Popup,
    Sidebar,
}

impl Form {
    pub fn parse(s: Option<&str>) -> Self {
        if s == Some("sidebar") { Form::Sidebar } else { Form::Popup }
    }
    fn name(self) -> &'static str {
        match self {
            Form::Popup => "popup",
            Form::Sidebar => "sidebar",
        }
    }
}

/// Edge of the sidebar.
#[derive(Clone, Copy, PartialEq, Default, Debug)]
pub enum Side {
    /// The half of the screen with the tray icon, as last seen on a tray click.
    #[default]
    Auto,
    Left,
    Right,
}

impl Side {
    pub fn parse(s: Option<&str>) -> Self {
        match s {
            Some("left") => Side::Left,
            Some("right") => Side::Right,
            _ => Side::Auto,
        }
    }
}

#[derive(Default)]
pub struct MainWindowState {
    form: Mutex<Form>,
    /// The form and side from the settings.
    pref: Mutex<(Form, Side)>,
    /// Whether the tray icon was last seen in the left half of its screen.
    tray_left: Mutex<Option<bool>>,
    last_click: Mutex<Option<(i32, i32)>>,
    /// Last geometry sent to the window (x, y, physical height).
    applied: Mutex<Option<(i32, i32, i32)>>,
    hidden_on_blur: Mutex<Option<Instant>>,
    shown_at: Mutex<Option<Instant>>,
    /// Pinned: stays open (popup) or on top (sidebar) when another window gets the focus.
    pinned: AtomicBool,
    /// The sidebar went behind other windows when it lost the focus, at this moment.
    lowered: Mutex<Option<Instant>>,
}

pub fn create(app: &tauri::App) -> tauri::Result<()> {
    let main = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("app.html".into()))
        .title("Maillet")
        .inner_size(WIDTH, HEIGHT)
        .decorations(false)
        .skip_taskbar(true)
        .visible(false)
        .build()?;
    let handle = app.handle().clone();
    main.on_window_event(move |event| match event {
        // Popup behaviour: clicking elsewhere hides the window. The sidebar stays.
        WindowEvent::Focused(false) => {
            let state = handle.state::<MainWindowState>();
            let just_shown = state.shown_at.lock().unwrap().is_some_and(|t| t.elapsed() < Duration::from_millis(400));
            // Open settings keep the popup open, so their changes show there at once
            let settings_open = settings_open(&handle);
            let pinned = state.pinned.load(Ordering::Relaxed);
            // The sidebar stays; the window that took the focus may now cover it.
            if *state.form.lock().unwrap() == Form::Sidebar && !pinned && !just_shown {
                *state.lowered.lock().unwrap() = Some(Instant::now());
            }
            if *state.form.lock().unwrap() == Form::Popup && !just_shown && !settings_open && !pinned {
                *state.hidden_on_blur.lock().unwrap() = Some(Instant::now());
                if let Some(w) = handle.get_webview_window(MAIN) {
                    let _ = w.hide();
                }
            }
        }
        // A click into the sidebar brings it to the front again.
        WindowEvent::Focused(true) => {
            handle.state::<MainWindowState>().lowered.lock().unwrap().take();
        }
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();
            if let Some(w) = handle.get_webview_window(MAIN) {
                let _ = w.hide();
            }
        }
        _ => {}
    });

    // Settings and About are built when first opened and destroyed when closed (`show`): each
    // window is a web process of its own. The code popup is prepared, it must appear at once.
    WebviewWindowBuilder::new(app, "code", WebviewUrl::App("code.html".into()))
        .title(crate::base::i18n::t("win.code", &[]))
        .inner_size(crate::features::otp::popup::POPUP_WIDTH, 250.0)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;
    Ok(())
}

/// The settings closed: if the main window is still open, it gets the focus back, so the next click
/// elsewhere or Esc hides the popup as usual.
fn refocus_main(app: &AppHandle) {
    let Some(w) = app.get_webview_window(MAIN) else { return };
    if w.is_visible().unwrap_or(false) {
        *app.state::<MainWindowState>().shown_at.lock().unwrap() = Some(Instant::now());
        let _ = w.set_focus();
        activate_x11(&w);
    }
}

/// Set before the settings window is built, cleared when it is gone: the main window may lose the
/// focus to it before it counts as visible, and must stay open then.
static SETTINGS_OPENING: AtomicBool = AtomicBool::new(false);

/// Whether the settings are open (or opening): then the popup does not hide on focus loss.
fn settings_open(app: &AppHandle) -> bool {
    SETTINGS_OPENING.load(Ordering::Relaxed) || app.get_webview_window("settings").is_some_and(|w| w.is_visible().unwrap_or(false))
}

/// Opens Settings or About, building the window if it is not there yet; closing destroys it.
pub fn show(app: &AppHandle, label: &str) {
    if let Some(w) = app.get_webview_window(label) {
        let _ = w.unminimize();
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }
    let built = match label {
        "settings" => {
            SETTINGS_OPENING.store(true, Ordering::Relaxed);
            WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
                .title(crate::base::i18n::t("win.settings", &[]))
                .inner_size(820.0, 720.0)
                .visible(false)
                .build()
        }
        "about" => WebviewWindowBuilder::new(app, "about", WebviewUrl::App("about.html".into()))
            .title(crate::base::i18n::t("win.about", &[]))
            .inner_size(360.0, 340.0)
            .resizable(false)
            .visible(false)
            .build(),
        _ => return,
    };
    // Built hidden: the page shows itself once its first picture is complete (`window_ready`), so the
    // content does not visibly build up. Should the page never report, it shows after a moment anyway.
    if let Ok(w) = &built {
        let w = w.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(READY_TIMEOUT_MS));
            reveal(&w);
        });
    }
    match built {
        Ok(w) if label == "settings" => {
            let handle = app.clone();
            w.on_window_event(move |event| {
                if let WindowEvent::Destroyed = event {
                    SETTINGS_OPENING.store(false, Ordering::Relaxed);
                    refocus_main(&handle);
                }
            });
        }
        Ok(_) => {}
        Err(e) => {
            SETTINGS_OPENING.store(false, Ordering::Relaxed);
            tracing::warn!("Could not open the {label} window: {e}");
        }
    }
}

/// How long a Settings or About window may take to report ready before it is shown anyway.
const READY_TIMEOUT_MS: u64 = 1500;

/// Shows a Settings or About window that was built hidden, once; later calls change nothing.
pub fn reveal(w: &WebviewWindow) {
    if !w.is_visible().unwrap_or(true) {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

/// The form and side from the settings; an open window takes the new form at once.
pub fn set_pref(app: &AppHandle, form: Form, side: Side) {
    let state = app.state::<MainWindowState>();
    *state.pref.lock().unwrap() = (form, side);
    // An open window (the settings keep it open) changes its form in place, without taking the
    // focus from the settings window.
    if let Some(w) = app.get_webview_window(MAIN).filter(|w| w.is_visible().unwrap_or(false)) {
        *state.form.lock().unwrap() = form;
        state.pinned.store(false, Ordering::Relaxed);
        let _ = w.set_always_on_top(form == Form::Popup);
        layout(app, &w);
        let _ = app.emit("main-shown", form.name());
    }
}

/// Tray click (with the icon's position) and global shortcut (without): opens the window in the
/// form from the settings, or hides it if it is open.
pub fn toggle_main(app: &AppHandle, at: Option<(i32, i32)>) {
    let Some(w) = app.get_webview_window(MAIN) else { return };
    let state = app.state::<MainWindowState>();
    // Clicking the icon or the key grab of the shortcut first takes the focus away, which may
    // already have hidden the popup a moment ago – then the window counts as open.
    let just_hidden = state.hidden_on_blur.lock().unwrap().is_some_and(|t| t.elapsed() < Duration::from_millis(400));
    // A sidebar behind other windows comes to the front; one that was in front a moment ago (this
    // very click took its focus) hides.
    let lowered = *state.lowered.lock().unwrap();
    let behind = lowered.is_some_and(|t| t.elapsed() >= Duration::from_millis(400));
    if w.is_visible().unwrap_or(false) && current_is_sidebar(app) && behind {
        show_main(app, at);
    } else if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
    } else if !just_hidden {
        show_main(app, at);
    }
}

/// Global shortcut: like the tray click, except that a pinned sidebar is never hidden by it, only
/// brought to the front with the focus (it is kept above other windows anyway).
pub fn toggle_main_by_key(app: &AppHandle) {
    let Some(w) = app.get_webview_window(MAIN) else { return };
    let pinned = app.state::<MainWindowState>().pinned.load(Ordering::Relaxed);
    if w.is_visible().unwrap_or(false) && current_is_sidebar(app) && pinned {
        let _ = w.set_focus();
        activate_x11(&w);
    } else {
        toggle_main(app, None);
    }
}

fn current_is_sidebar(app: &AppHandle) -> bool {
    *app.state::<MainWindowState>().form.lock().unwrap() == Form::Sidebar
}

/// Opens the window in the form from the settings (tray click, tray menu, notification).
pub fn show_main(app: &AppHandle, at: Option<(i32, i32)>) {
    let form = app.state::<MainWindowState>().pref.lock().unwrap().0;
    show_form(app, form, at);
}

fn show_form(app: &AppHandle, form: Form, at: Option<(i32, i32)>) {
    let Some(w) = app.get_webview_window(MAIN) else { return };
    let state = app.state::<MainWindowState>();
    *state.form.lock().unwrap() = form;
    if let Some(p) = at {
        *state.last_click.lock().unwrap() = Some(p);
    }
    *state.shown_at.lock().unwrap() = Some(Instant::now());
    // Every opening starts unpinned and in front; the page resets its pin button on this event.
    state.pinned.store(false, Ordering::Relaxed);
    *state.lowered.lock().unwrap() = None;
    let _ = app.emit("main-shown", form.name());
    // Switching the form of an open window: hide it first, so it never shows
    // up with the size of the other form.
    if w.is_visible().unwrap_or(false) {
        let _ = w.hide();
    }
    // Size while hidden, then show; GTK ignores moves of hidden windows, so
    // the position is set (again) after showing.
    layout(app, &w);
    let _ = w.show();
    layout(app, &w);
    // The popup must stay visible above other always-on-top windows (a drop-down terminal such as
    // Guake) when summoned; the sidebar is kept above other windows only when pinned.
    let _ = w.set_always_on_top(form == Form::Popup);
    let _ = w.set_focus();
    activate_x11(&w);
}

/// Sets size and position for the current form.
fn layout(app: &AppHandle, w: &WebviewWindow) {
    let state = app.state::<MainWindowState>();
    let form = *state.form.lock().unwrap();
    let anchor = *state.last_click.lock().unwrap();
    // The monitor of the tray icon, else the one under the mouse, else the primary.
    let point = anchor.or_else(|| app.cursor_position().ok().map(|p| (p.x as i32, p.y as i32)));
    let monitor = point
        .and_then(|(x, y)| w.monitor_from_point(x as f64, y as f64).ok().flatten())
        .or_else(|| w.primary_monitor().ok().flatten());
    let Some(monitor) = monitor else { return };
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let (ax, ay, aw, ah) = (area.position.x, area.position.y, area.size.width as i32, area.size.height as i32);
    let width = ((WIDTH * scale).round() as i32).min(aw);

    // Seen from a tray click: in which half of its screen the icon sits.
    if let Some((cx, _)) = anchor {
        *state.tray_left.lock().unwrap() = Some(cx < ax + aw / 2);
    }
    let left = match state.pref.lock().unwrap().1 {
        Side::Left => true,
        Side::Right => false,
        Side::Auto => state.tray_left.lock().unwrap().unwrap_or(false),
    };
    let (height, x, y) = match form {
        Form::Sidebar => (ah, if left { ax } else { ax + aw - width }, ay),
        Form::Popup => {
            // Smaller only if the screen itself is smaller.
            let height = ((HEIGHT * scale).round() as i32).min(ah - 2 * MARGIN);
            let (x, y) = match anchor {
                // Above the icon if the panel is at the bottom, else below it.
                Some((cx, cy)) => (cx - width / 2, if cy > ay + ah / 2 { cy - height - MARGIN } else { cy + MARGIN }),
                None => (ax + aw - width - MARGIN, ay + MARGIN),
            };
            let x = x.clamp(ax + MARGIN, (ax + aw - width - MARGIN).max(ax));
            let y = y.clamp(ay + MARGIN, (ay + ah - height - MARGIN).max(ay));
            (height, x, y)
        }
    };
    // Only send what changed: every resize or move is visible as a flicker.
    // A hidden window is always set in full (it may have been moved meanwhile).
    let visible = w.is_visible().unwrap_or(false);
    let before = state.applied.lock().unwrap().replace((x, y, height));
    let (resize, moved) = match before {
        Some((bx, by, bh)) if visible => (bh != height, (bx, by) != (x, y)),
        _ => (true, true),
    };
    // (Wayland compositors may ignore positions.)
    if resize {
        let _ = w.set_size(LogicalSize::new(width as f64 / scale, height as f64 / scale));
    }
    if !moved {
        return;
    }
    let _ = w.set_position(PhysicalPosition::new(x, y));
    // The window manager keeps a window on screen: a move that arrives while
    // the window still has its old (larger) size gets clamped. Move again once
    // the resize has gone through.
    let w = w.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(80));
        let _ = w.set_position(PhysicalPosition::new(x, y));
    });
}

/// Window managers refuse focus to windows of background programs, and a
/// tray click does not count as user action for our window. Without focus
/// the popup would not close when clicking elsewhere. Pagers and task bars
/// may activate windows (`_NET_ACTIVE_WINDOW`, source 2) – we do the same.
/// On Wayland the GDK window is no X11 window and nothing happens.
#[cfg(target_os = "linux")]
fn activate_x11(w: &WebviewWindow) {
    request_activation(w);
    // On the very first show the window may not be mapped yet and the
    // request is ignored; ask again shortly after unless it worked.
    let w = w.clone();
    std::thread::spawn(move || {
        for delay in [150, 250, 400, 700] {
            std::thread::sleep(Duration::from_millis(delay));
            if w.is_visible().unwrap_or(false) && !w.is_focused().unwrap_or(false) {
                request_activation(&w);
            }
        }
    });
}

#[cfg(target_os = "linux")]
fn request_activation(w: &WebviewWindow) {
    let window = w.clone();
    let _ = w.run_on_main_thread(move || {
        use gtk::prelude::{Cast, WidgetExt};
        let Ok(gtk_window) = window.gtk_window() else { return };
        let Some(gdk_window) = gtk_window.window() else { return };
        let Ok(x11_window) = gdk_window.downcast::<gdkx11::X11Window>() else { return };
        send_active_window(x11_window.xid());
    });
}

#[cfg(not(target_os = "linux"))]
fn activate_x11(_w: &WebviewWindow) {}

#[cfg(target_os = "linux")]
fn send_active_window(xid: u64) {
    use x11_dl::xlib;
    let Ok(x) = xlib::Xlib::open() else { return };
    // SAFETY: plain Xlib calls on a display opened and closed here.
    unsafe {
        let display = (x.XOpenDisplay)(std::ptr::null());
        if display.is_null() {
            return;
        }
        let root = (x.XDefaultRootWindow)(display);
        let atom = (x.XInternAtom)(display, c"_NET_ACTIVE_WINDOW".as_ptr(), xlib::False);
        let mut event: xlib::XEvent = std::mem::zeroed();
        event.client_message = xlib::XClientMessageEvent {
            type_: xlib::ClientMessage,
            serial: 0,
            send_event: xlib::True,
            display,
            window: xid as xlib::Window,
            message_type: atom,
            format: 32,
            data: xlib::ClientMessageData::from([2, xlib::CurrentTime as i64, 0, 0, 0]),
        };
        (x.XSendEvent)(display, root, xlib::False, xlib::SubstructureRedirectMask | xlib::SubstructureNotifyMask, &mut event);
        (x.XFlush)(display);
        (x.XCloseDisplay)(display);
    }
}

/// Pin or unpin: pinned, the popup stays open when another window gets the focus and the sidebar is
/// kept above other windows (as in Tasqlet); unpinned, other windows may cover the sidebar.
#[tauri::command]
pub fn pin_main(app: AppHandle, pinned: bool) {
    let state = app.state::<MainWindowState>();
    state.pinned.store(pinned, Ordering::Relaxed);
    if current_is_sidebar(&app) {
        if pinned {
            *state.lowered.lock().unwrap() = None;
        }
        if let Some(w) = app.get_webview_window(MAIN) {
            let _ = w.set_always_on_top(pinned);
        }
    }
}

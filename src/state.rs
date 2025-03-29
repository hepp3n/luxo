use std::{
    ffi::OsString,
    sync::{atomic::AtomicBool, Arc},
};

use smithay::{
    backend::renderer::element::{default_primary_scanout_output_compare, RenderElementStates},
    desktop::{
        utils::{
            surface_presentation_feedback_flags_from_states, surface_primary_scanout_output,
            update_surface_primary_scanout_output, with_surfaces_surface_tree,
            OutputPresentationFeedback,
        },
        PopupManager, Space,
    },
    input::{
        keyboard::XkbConfig,
        pointer::{CursorImageStatus, PointerHandle},
        Seat, SeatState,
    },
    output::Output,
    reexports::{
        calloop::LoopHandle,
        wayland_server::backend::{ClientData, ClientId, DisconnectReason},
    },
    utils::{Clock, Monotonic},
    wayland::{
        compositor::{CompositorClientState, CompositorState},
        dmabuf::DmabufFeedback,
        keyboard_shortcuts_inhibit::KeyboardShortcutsInhibitState,
        selection::{data_device::DataDeviceState, primary_selection::PrimarySelectionState},
        shell::{wlr_layer::WlrLayerShellState, xdg::XdgShellState},
        shm::ShmState,
        socket::ListeningSocketSource,
        xwayland_keyboard_grab::XWaylandKeyboardGrabState,
        xwayland_shell::XWaylandShellState,
    },
    xwayland::{X11Wm, XWayland, XWaylandEvent},
};
use xkbcommon::xkb::Keysym;

use crate::{shell::element::WindowElement, udev::UdevData};

pub struct Luxo {
    pub running: Arc<AtomicBool>,
    pub udev_data: UdevData,

    pub start_time: std::time::Instant,
    pub socket_name: OsString,

    pub space: Space<WindowElement>,
    pub popups: PopupManager,
    pub handle: LoopHandle<'static, Luxo>,

    // smithay states
    pub seat_state: SeatState<Luxo>,
    pub shm_state: ShmState,
    pub data_device_state: DataDeviceState,
    pub primary_selection_state: PrimarySelectionState,
    pub compositor_state: CompositorState,
    pub layer_shell_state: WlrLayerShellState,
    pub keyboard_shortcuts_inhibit_state: KeyboardShortcutsInhibitState,
    pub xdg_shell_state: XdgShellState,

    pub seat: Seat<Luxo>,
    pub suppressed_keys: Vec<Keysym>,
    pub pointer: PointerHandle<Luxo>,
    pub cursor_status: CursorImageStatus,
    pub clock: Clock<Monotonic>,

    // xwayland
    pub xwayland_shell_state: XWaylandShellState,
    pub xwm: Option<X11Wm>,
    pub xdisplay: Option<u32>,
}

impl Luxo {
    pub fn new(handle: LoopHandle<'static, Luxo>, udev_data: UdevData) -> Self {
        let display_handle = &udev_data.display_handle;
        let start_time = std::time::Instant::now();

        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        let socket_name = listening_socket.socket_name().to_os_string();

        let clock = Clock::new();

        let mut seat_state = SeatState::<Luxo>::new();
        let shm_state = ShmState::new::<Luxo>(display_handle, vec![]);
        let data_device_state = DataDeviceState::new::<Luxo>(display_handle);
        let primary_selection_state = PrimarySelectionState::new::<Luxo>(display_handle);
        let compositor_state = CompositorState::new::<Luxo>(display_handle);
        let layer_shell_state = WlrLayerShellState::new::<Self>(display_handle);
        let xdg_shell_state = XdgShellState::new::<Luxo>(display_handle);

        // init input
        let mut seat = seat_state.new_wl_seat(display_handle, udev_data.seat_name());

        let pointer = seat.add_pointer();

        let cursor_status = CursorImageStatus::default_named();

        seat.add_keyboard(XkbConfig::default(), 200, 25)
            .expect("Failed to initialize the keyboard");

        let keyboard_shortcuts_inhibit_state =
            KeyboardShortcutsInhibitState::new::<Self>(display_handle);

        // A space represents a two-dimensional plane. Windows and Outputs can be mapped onto it.
        //
        // Windows get a position and stacking order through mapping.
        // Outputs become views of a part of the Space and can be rendered via Space::render_output.
        let space = Space::default();
        let popups = PopupManager::default();

        let xwayland_shell_state = XWaylandShellState::new::<Self>(display_handle);

        XWaylandKeyboardGrabState::new::<Self>(display_handle);

        Self {
            running: Arc::new(AtomicBool::new(true)),
            start_time,
            udev_data,

            space,
            popups,

            handle,

            socket_name,

            // smithay states
            seat_state,
            shm_state,
            data_device_state,
            primary_selection_state,
            compositor_state,
            layer_shell_state,
            keyboard_shortcuts_inhibit_state,
            xdg_shell_state,

            seat,
            suppressed_keys: Vec::new(),
            pointer,
            cursor_status,
            clock,

            // xwayland
            xwayland_shell_state,
            xwm: None,
            xdisplay: None,
        }
    }

    pub fn start_xwayland(&self) -> anyhow::Result<()> {
        use std::process::Stdio;

        use smithay::wayland::compositor::CompositorHandler;

        let (xwayland, client) = XWayland::spawn(
            &self.udev_data.display_handle,
            None,
            std::iter::empty::<(String, String)>(),
            true,
            Stdio::null(),
            Stdio::null(),
            |_| (),
        )
        .expect("failed to start XWayland");

        let ret = self
            .handle
            .insert_source(xwayland, move |event, _, data| match event {
                XWaylandEvent::Ready {
                    x11_socket,
                    display_number,
                } => {
                    let xwayland_scale = std::env::var("LUXO_XWAYLAND_SCALE")
                        .ok()
                        .and_then(|s| s.parse::<u32>().ok())
                        .unwrap_or(1);
                    data.client_compositor_state(&client)
                        .set_client_scale(xwayland_scale);
                    let mut _wm = X11Wm::start_wm(data.handle.clone(), x11_socket, client.clone())
                        .expect("Failed to attach X11 Window Manager");

                    // let cursor = Cursor::load();
                    // let image = cursor.get_image(1, Duration::ZERO);
                    // wm.set_cursor(
                    //     &image.pixels_rgba,
                    //     Size::from((image.width as u16, image.height as u16)),
                    //     Point::from((image.xhot as u16, image.yhot as u16)),
                    // )
                    // .expect("Failed to set xwayland default cursor");
                    // data.xwm = Some(wm);
                    data.xdisplay = Some(display_number);
                }
                XWaylandEvent::Error => {
                    tracing::warn!("XWayland crashed on startup");
                }
            });
        if let Err(e) = ret {
            tracing::error!(
                "Failed to insert the XWaylandSource into the event loop: {}",
                e
            );
        }

        Ok(())
    }
}

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

pub fn update_primary_scanout_output(
    space: &Space<WindowElement>,
    output: &Output,
    cursor_status: &CursorImageStatus,
    render_element_states: &RenderElementStates,
) {
    space.elements().for_each(|window| {
        window.with_surfaces(|surface, states| {
            update_surface_primary_scanout_output(
                surface,
                output,
                states,
                render_element_states,
                default_primary_scanout_output_compare,
            );
        });
    });
    let map = smithay::desktop::layer_map_for_output(output);
    for layer_surface in map.layers() {
        layer_surface.with_surfaces(|surface, states| {
            update_surface_primary_scanout_output(
                surface,
                output,
                states,
                render_element_states,
                default_primary_scanout_output_compare,
            );
        });
    }

    if let CursorImageStatus::Surface(ref surface) = cursor_status {
        with_surfaces_surface_tree(surface, |surface, states| {
            update_surface_primary_scanout_output(
                surface,
                output,
                states,
                render_element_states,
                default_primary_scanout_output_compare,
            );
        });
    }
}

#[derive(Debug, Clone)]
pub struct SurfaceDmabufFeedback {
    pub render_feedback: DmabufFeedback,
    pub scanout_feedback: DmabufFeedback,
}

pub fn take_presentation_feedback(
    output: &Output,
    space: &Space<WindowElement>,
    render_element_states: &RenderElementStates,
) -> OutputPresentationFeedback {
    let mut output_presentation_feedback = OutputPresentationFeedback::new(output);

    space.elements().for_each(|window| {
        if space.outputs_for_element(window).contains(output) {
            window.take_presentation_feedback(
                &mut output_presentation_feedback,
                surface_primary_scanout_output,
                |surface, _| {
                    surface_presentation_feedback_flags_from_states(surface, render_element_states)
                },
            );
        }
    });
    let map = smithay::desktop::layer_map_for_output(output);
    for layer_surface in map.layers() {
        layer_surface.take_presentation_feedback(
            &mut output_presentation_feedback,
            surface_primary_scanout_output,
            |surface, _| {
                surface_presentation_feedback_flags_from_states(surface, render_element_states)
            },
        );
    }

    output_presentation_feedback
}

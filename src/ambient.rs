use std::{
    collections::VecDeque,
    fs::File,
    os::fd::{AsFd, BorrowedFd, OwnedFd},
};

use memmap::MmapMut;

use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer::WlBuffer,
        wl_output::WlOutput,
        wl_registry::{Event, WlRegistry},
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols::wp::linux_dmabuf::zv1::client::{
    zwp_linux_buffer_params_v1::{self, ZwpLinuxBufferParamsV1},
    zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

use crate::modes::Mode;

pub struct Card(std::fs::File);

impl AsFd for Card {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl Card {
    pub fn open(path: &str) -> Self {
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        options.write(true);
        Card(options.open(path).unwrap())
    }
}

struct FrameMeta {
    file: OwnedFd,
    height: u32,
    width: u32,
    stride: u32,
    format: gbm::Format,
}

pub enum Algorithm {
    Samples,
}

struct State {
    screencopy_manager: ZwlrScreencopyManagerV1,
    dma: ZwpLinuxDmabufV1,
    wl_output: WlOutput,
    // surfaces: VecDeque<(FrameInfo, ZwlrScreencopyFrameV1, WlBuffer, WlShmPool)>,
    surfaces: VecDeque<(
        FrameMeta,
        ZwlrScreencopyFrameV1,
        WlBuffer,
        ZwpLinuxBufferParamsV1,
    )>,
    latest_frame: Option<Vec<u8>>,
    algorithm: Algorithm,
    gbm: gbm::Device<Card>,
    leds: Vec<(u16, u16, u16, u16)>,
}

impl State {
    fn from_connection(
        conn: &Connection,
        algorithm: Algorithm,
        leds: Vec<(u16, u16, u16, u16)>,
    ) -> (Self, EventQueue<Self>) {
        let (globals, queue) = registry_queue_init(conn).unwrap();
        let qh = queue.handle();
        let screencopy_manager = globals
            .bind(&qh, 3..=ZwlrScreencopyManagerV1::interface().version, ())
            .unwrap();
        let dma = globals
            .bind(&qh, 1..=ZwpLinuxDmabufV1::interface().version, ())
            .unwrap();
        let wl_output = globals
            .bind(&qh, 1..=WlOutput::interface().version, ())
            .unwrap();
        let surfaces = VecDeque::new();
        let gpu = Card::open("/dev/dri/renderD128");
        let gbm = gbm::Device::new(gpu).unwrap();
        (
            Self {
                screencopy_manager,
                dma,
                wl_output,
                surfaces,
                latest_frame: None,
                algorithm,
                gbm,
                leds,
            },
            queue,
        )
    }

    fn get_pixel_samples(&self, frame: FrameMeta) -> Vec<u8> {
        let mmap = unsafe { MmapMut::map_mut(&File::from(frame.file)).unwrap() };
        let mut pixels = Vec::with_capacity(self.leds.len() * 3);
        for (x, y, _, _) in &self.leds {
            let idx = ((*y as u32 * frame.width + *x as u32) * 4) as usize;
            pixels.push(mmap[idx + 2]);
            pixels.push(mmap[idx + 1]);
            pixels.push(mmap[idx]);
        }
        pixels
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for State {
    fn event(
        state: &mut Self,
        frame: &ZwlrScreencopyFrameV1,
        event: <ZwlrScreencopyFrameV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            zwlr_screencopy_frame_v1::Event::Failed => {
                println!("failed")
            }
            zwlr_screencopy_frame_v1::Event::LinuxDmabuf {
                format,
                width,
                height,
            } => {
                let bo = state
                    .gbm
                    .create_buffer_object::<()>(
                        width,
                        height,
                        gbm::Format::Argb8888,
                        gbm::BufferObjectFlags::LINEAR,
                    )
                    .unwrap();
                let owned_fd = bo.fd().unwrap();
                let fd = owned_fd.as_fd();
                let params = state.dma.create_params(qh, ());
                params.add(fd, 0, 0, bo.stride(), 0, 0);
                let buf = params.create_immed(
                    width as i32,
                    height as i32,
                    format,
                    zwp_linux_buffer_params_v1::Flags::empty(),
                    qh,
                    (),
                );
                frame.copy(&buf);
                let frameinfo = FrameMeta {
                    file: owned_fd,
                    height,
                    width,
                    stride: bo.stride(),
                    format: gbm::Format::Argb8888,
                };
                state
                    .surfaces
                    .push_back((frameinfo, frame.clone(), buf, params));
            }
            zwlr_screencopy_frame_v1::Event::Ready { .. } => {
                let (frameinfo, frame, buffer, params) = state.surfaces.pop_front().unwrap();
                frame.destroy();
                buffer.destroy();
                params.destroy();
                let pixels = match &state.algorithm {
                    Algorithm::Samples => state.get_pixel_samples(frameinfo),
                };
                state.latest_frame = Some(pixels);
            }
            _ => (),
        }
    }
}

impl Dispatch<WlBuffer, ()> for State {
    fn event(
        _: &mut Self,
        _: &WlBuffer,
        _: <WlBuffer as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrScreencopyManagerV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwlrScreencopyManagerV1,
        _: <ZwlrScreencopyManagerV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpLinuxDmabufV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpLinuxDmabufV1,
        _: <ZwpLinuxDmabufV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpLinuxBufferParamsV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &ZwpLinuxBufferParamsV1,
        _: <ZwpLinuxBufferParamsV1 as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlOutput, ()> for State {
    fn event(
        _: &mut Self,
        _: &WlOutput,
        _: <WlOutput as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut State,
        _: &WlRegistry,
        _: Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<State>,
    ) {
        /* react to dynamic global events here */
    }
}

pub struct Ambient {
    state: State,
    queue: EventQueue<State>,
    conn: Connection
}

impl Ambient {
    pub fn new(algorithm: Algorithm, leds: Vec<(u16, u16, u16, u16)>) -> Self {
        let conn = Connection::connect_to_env().unwrap();
        let (state, queue) = State::from_connection(&conn, algorithm, leds);
        Self { state, queue, conn }
    }
}


impl Mode for Ambient {
    fn render(&mut self) -> Vec<u8> {
        self.queue.blocking_dispatch(&mut self.state).unwrap();
        let qh = self.queue.handle();

        self.state
            .screencopy_manager
            .capture_output(1, &self.state.wl_output, &qh, ());

        let frame = &self.state.latest_frame;       
        match frame {
            Some(pixels) => pixels.clone(), // how do we avoid cloning here?
            None => vec![0, 0, 0]
        }
    }
}

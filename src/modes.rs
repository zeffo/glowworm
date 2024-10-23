use std::{
    ffi::CStr,
    fs::File,
    time::{SystemTime, UNIX_EPOCH},
};

use nix::{
    fcntl,
    sys::{memfd, mman, stat},
    unistd,
};

use colorgrad::{Color, CustomGradient};
use serde::{Deserialize, Serialize};
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use wayland_client::{
    globals::{registry_queue_init, GlobalListContents},
    protocol::{
        wl_buffer::WlBuffer, wl_output::WlOutput, wl_registry, wl_shm::WlShm,
        wl_shm_pool::WlShmPool,
    },
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
};
use wayland_protocols::wp::linux_dmabuf::zv1::client::{
    zwp_linux_buffer_params_v1::ZwpLinuxBufferParamsV1, zwp_linux_dmabuf_v1::ZwpLinuxDmabufV1,
};
use wayland_protocols_wlr::screencopy::v1::client::{
    zwlr_screencopy_frame_v1::{self, ZwlrScreencopyFrameV1},
    zwlr_screencopy_manager_v1::ZwlrScreencopyManagerV1,
};

pub trait GlowMode {
    fn get_colors(&mut self) -> Vec<u8>;
}

pub struct StaticGradient {
    colors: Vec<u8>,
}

impl StaticGradient {
    #[allow(dead_code)]
    pub fn from_colors(gradient_colors: Vec<Color>, leds: u16) -> Self {
        let gradient = CustomGradient::new()
            .colors(&gradient_colors)
            .build()
            .unwrap()
            .colors(leds as usize);
        let colors = gradient
            .iter()
            .flat_map(|c| c.to_rgba8()[..3].to_owned())
            .collect();
        Self { colors }
    }
}

impl GlowMode for StaticGradient {
    fn get_colors(&mut self) -> Vec<u8> {
        self.colors.to_vec()
    }
}

pub struct DynamicGradient {
    colors: Vec<u8>, // can we use a reference here?
    cursor: usize,
}
impl DynamicGradient {
    pub fn from_colors(gradient_colors: Vec<Color>, leds: u16) -> Self {
        let gradient = CustomGradient::new()
            .colors(&gradient_colors)
            .build()
            .unwrap()
            .colors(leds as usize);
        let colors = gradient
            .iter()
            .flat_map(|c| c.to_rgba8()[..3].to_owned())
            .collect();
        Self { colors, cursor: 0 }
    }
}

impl GlowMode for DynamicGradient {
    // TODO: try to return a reference here instead of allocating a new vec on every call
    fn get_colors(&mut self) -> Vec<u8> {
        let ret = [
            &self.colors[self.cursor..self.colors.len()],
            &self.colors[..self.cursor],
        ]
        .concat();
        self.cursor = (self.cursor + 3) % self.colors.len();
        ret
    }
}

#[derive(Serialize, Deserialize)]
pub struct LEDConfig {
    leds: Vec<(u16, u16, u16, u16)>, // vec of capture areas for each LED
}

fn create_shm_fd() -> std::io::Result<OwnedFd> {
    // Only try memfd on linux and freebsd.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    loop {
        // Create a file that closes on successful execution and seal it's operations.
        match memfd::memfd_create(
            CStr::from_bytes_with_nul(b"glowworm\0").unwrap(),
            memfd::MemFdCreateFlag::MFD_CLOEXEC | memfd::MemFdCreateFlag::MFD_ALLOW_SEALING,
        ) {
            Ok(fd) => {
                // This is only an optimization, so ignore errors.
                // F_SEAL_SRHINK = File cannot be reduced in size.
                // F_SEAL_SEAL = Prevent further calls to fcntl().
                let _ = fcntl::fcntl(
                    fd.as_raw_fd(),
                    fcntl::F_ADD_SEALS(
                        fcntl::SealFlag::F_SEAL_SHRINK | fcntl::SealFlag::F_SEAL_SEAL,
                    ),
                );
                return Ok(fd);
            }
            Err(nix::errno::Errno::EINTR) => continue,
            Err(nix::errno::Errno::ENOSYS) => break,
            Err(errno) => return Err(std::io::Error::from(errno)),
        }
    }

    // Fallback to using shm_open.
    let sys_time = SystemTime::now();
    let mut mem_file_handle = format!(
        "/glowworm-{}",
        sys_time.duration_since(UNIX_EPOCH).unwrap().subsec_nanos()
    );
    loop {
        match mman::shm_open(
            // O_CREAT = Create file if does not exist.
            // O_EXCL = Error if create and file exists.
            // O_RDWR = Open for reading and writing.
            // O_CLOEXEC = Close on successful execution.
            // S_IRUSR = Set user read permission bit .
            // S_IWUSR = Set user write permission bit.
            mem_file_handle.as_str(),
            fcntl::OFlag::O_CREAT
                | fcntl::OFlag::O_EXCL
                | fcntl::OFlag::O_RDWR
                | fcntl::OFlag::O_CLOEXEC,
            stat::Mode::S_IRUSR | stat::Mode::S_IWUSR,
        ) {
            Ok(fd) => match mman::shm_unlink(mem_file_handle.as_str()) {
                Ok(_) => return Ok(fd),
                Err(errno) => match unistd::close(fd.as_raw_fd()) {
                    Ok(_) => return Err(std::io::Error::from(errno)),
                    Err(errno) => return Err(std::io::Error::from(errno)),
                },
            },
            Err(nix::errno::Errno::EEXIST) => {
                // If a file with that handle exists then change the handle
                mem_file_handle = format!(
                    "/glowworm-{}",
                    sys_time.duration_since(UNIX_EPOCH).unwrap().subsec_nanos()
                );
                continue;
            }
            Err(nix::errno::Errno::EINTR) => continue,
            Err(errno) => return Err(std::io::Error::from(errno)),
        }
    }
}

pub struct AmbientState {
    screencopy_manager: ZwlrScreencopyManagerV1,
    latest_frame: Option<ZwlrScreencopyFrameV1>,
    dma: ZwpLinuxDmabufV1,
    shm: WlShm,
    wl_output: WlOutput,
}

impl AmbientState {
    fn from_connection(conn: &Connection) -> (Self, EventQueue<Self>) {
        let (globals, queue) = registry_queue_init(conn).unwrap();
        let qh = queue.handle();
        let latest_frame = None;
        let screencopy_manager = globals
            .bind(&qh, 3..=ZwlrScreencopyManagerV1::interface().version, ())
            .unwrap();
        let dma = globals
            .bind(&qh, 1..=ZwpLinuxDmabufV1::interface().version, ())
            .unwrap();
        let wl_output = globals
            .bind(&qh, 1..=WlOutput::interface().version, ())
            .unwrap();
        let shm = globals
            .bind(&qh, 1..=WlShm::interface().version, ())
            .unwrap();
        (
            Self {
                screencopy_manager,
                latest_frame,
                dma,
                shm,
                wl_output,
            },
            queue,
        )
    }
}

impl Dispatch<ZwlrScreencopyFrameV1, ()> for AmbientState {
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
                // let fourcc = DrmFourcc::try_from(format).unwrap();
                // let params = state.dma.create_params(qh, ());
                // params.add(fd, plane_idx, offset, stride, modifier_hi, modifier_lo);
                // let buffer = params.create_immed(
                //     width as i32,
                //     height as i32,
                //     format,
                //     Flags::empty(),
                //     qh,
                //     (),
                // );
                // frame.copy(&buffer);
                // println!("{:#?}, {:#?}, {:#?}", fourcc, width, height);
            }
            zwlr_screencopy_frame_v1::Event::Buffer {
                format,
                width,
                height,
                stride,
            } => match format {
                WEnum::Value(v) => {
                    let size = (height * stride * 2) as u64;
                    let file = File::from(create_shm_fd().unwrap());
                    file.set_len(size).unwrap();
                    let fd = file.as_fd();
                    let pool = state.shm.create_pool(fd, size as i32, qh, ());
                    let buffer = pool.create_buffer(
                        0,
                        width as i32,
                        height as i32,
                        stride as i32,
                        v,
                        qh,
                        (),
                    );
                    frame.copy(&buffer);
                    pool.destroy();
                    buffer.destroy();
                    frame.destroy();
                }
                WEnum::Unknown(e) => {
                    println!("{:#?}", e);
                }
            },
            zwlr_screencopy_frame_v1::Event::Ready {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
            } => {
                println!("{:#?}, {:#?}, {:#?}", tv_sec_hi, tv_sec_lo, tv_nsec);
            }
            _ => (),
        }
    }
}
impl Dispatch<WlShm, ()> for AmbientState {
    fn event(
        _: &mut Self,
        _: &WlShm,
        _: <WlShm as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlShmPool, ()> for AmbientState {
    fn event(
        _: &mut Self,
        _: &WlShmPool,
        _: <WlShmPool as Proxy>::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<WlBuffer, ()> for AmbientState {
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

impl Dispatch<ZwlrScreencopyManagerV1, ()> for AmbientState {
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

impl Dispatch<ZwpLinuxDmabufV1, ()> for AmbientState {
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

impl Dispatch<ZwpLinuxBufferParamsV1, ()> for AmbientState {
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

impl Dispatch<WlOutput, ()> for AmbientState {
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

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for AmbientState {
    fn event(
        _: &mut AmbientState,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<AmbientState>,
    ) {
        /* react to dynamic global events here */
    }
}

pub struct Ambient {
    state: AmbientState,
    queue: EventQueue<AmbientState>,
    conn: Connection,
}

impl Ambient {
    pub fn new() -> Self {
        let conn = Connection::connect_to_env().unwrap();
        let (state, queue) = AmbientState::from_connection(&conn);
        Ambient { state, queue, conn }
    }
}

impl GlowMode for Ambient {
    fn get_colors(&mut self) -> Vec<u8> {
        self.queue.blocking_dispatch(&mut self.state).unwrap();
        let qh = self.queue.handle();

        let _ = self
            .state
            .screencopy_manager
            .capture_output(1, &self.state.wl_output, &qh, ());
        vec![0, 0, 0]
    }
}

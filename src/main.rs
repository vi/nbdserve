extern crate bufstream;
extern crate nbd;
#[macro_use]
extern crate structopt;

use std::path::PathBuf;
use structopt::StructOpt;

use nbd::server::{handshake, transmission, Export};
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result, Seek, SeekFrom};
use std::net::{TcpListener, TcpStream};

#[derive(Debug, StructOpt)]
struct Opt {
    /// Address to listen the port on
    #[structopt(short = "a", long = "addr", default_value = "127.0.0.1")]
    host: String,

    /// TCP port to listen
    #[structopt(short = "p", long = "port", default_value = "10809")]
    port: u16,

    /// Read-only mode
    #[structopt(short = "r", long = "read-only")]
    readonly: bool,

    // Size in bytes. May be omitted for regular files
    #[structopt(short = "s", long = "size")]
    size: Option<u64>,

    /// Hint clients that elevator algorithm should be used
    #[structopt(long = "rotational")]
    rotational: bool,

    /// Convert TRIM operations to FALLOC_FL_PUNCH_HOLE or something (not implemented)
    #[structopt(long = "trim")]
    trim: bool,

    /// Support RESIZE NBD extension (not implemented)
    #[structopt(long = "resize")]
    resize: bool,

    /// Quiet mode, suppress non-error output
    #[structopt(short = "q", long = "quiet")]
    quiet: bool,

    /// File or device to be served
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn strerror(s: &'static str) -> Result<()> {
    let stderr: Box<dyn std::error::Error + Send + Sync> = s.into();
    Err(Error::new(ErrorKind::InvalidData, stderr))
}

fn handle_client(opt: &Opt, stream: TcpStream) -> Result<()> {
    let mut s = bufstream::BufStream::new(stream);
    let data = handshake(&mut s, |name| {
        // Multiple files could be served under different export names.
        // For now, simply use the one file.
        println!("requested export: {name}");

        let mut oo = OpenOptions::new();
        oo.read(true);
        if !opt.readonly {
            oo.write(true).create(true);
        }
        let mut f = oo.open(opt.file.clone())?;

        let size = opt.size.unwrap_or_else(|| get_size(&mut f));
        if size == 0 {
            eprintln!(
                "warning: Use --size option to set size of the device. Serving zero-sized nbd now."
            );
        }

        Ok(Export {
            size,
            readonly: opt.readonly,
            rotational: opt.rotational,
            data: f,
            resizeable: false,
            send_trim: false,
            send_flush: false,
        })
    })?;
    transmission(&mut s, data)?;
    Ok(())
}

#[cfg(windows)]
fn size_for_windows_device(f: &mut File) -> u64 {
    use std::os::windows::io::AsRawHandle;
    use std::ptr::null_mut;
    use winapi::ctypes::c_void;
    use winapi::shared::minwindef::DWORD;
    use winapi::um::winioctl::{GET_LENGTH_INFORMATION, IOCTL_DISK_GET_LENGTH_INFO};
    let mut p_length: GET_LENGTH_INFORMATION = Default::default();
    let fh = f.as_raw_handle();
    let dwpl_size: DWORD = std::mem::size_of::<GET_LENGTH_INFORMATION>() as DWORD;
    let mut dwpl_bytes_return: DWORD = 0;
    let ret = unsafe {
        winapi::um::ioapiset::DeviceIoControl(
            fh as *mut c_void,
            IOCTL_DISK_GET_LENGTH_INFO,
            null_mut(),
            0,
            &mut p_length as *mut GET_LENGTH_INFORMATION as *mut c_void,
            dwpl_size,
            &mut dwpl_bytes_return as *mut DWORD,
            null_mut(),
        )
    };
    if ret == 0 {
        0
    } else {
        let size = unsafe { *p_length.Length.QuadPart() };
        size as u64
    }
}

#[cfg(not(windows))]
fn size_for_windows_device(f: &mut File) -> u64 {
    0
}

fn get_size(f: &mut File) -> u64 {
    if let Ok(x) = f.seek(SeekFrom::End(0)) {
        if x != 0 {
            return x;
        }
    }

    if let Ok(metadata) = f.metadata() {
        return metadata.len();
    }

    size_for_windows_device(f)
}

fn main() -> Result<()> {
    let opt = Opt::from_args();

    if opt.trim || opt.resize {
        strerror("This option is not supported yet")?;
    }

    let hostport = format!("{}:{}", opt.host, opt.port);
    let listener = TcpListener::bind(&hostport)?;

    if !opt.quiet {
        println!("Serving NBD on {}", hostport);
    }

    while let Ok((stream, addr)) = listener.accept() {
        if !opt.quiet {
            println!("A connection from {}", addr);
        }
        match handle_client(&opt, stream) {
            Ok(_) => {
                if !opt.quiet {
                    println!("finished");
                };
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }
    strerror("socket accept error")?;
    Ok(())
}

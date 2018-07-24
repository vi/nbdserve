extern crate nbd;
#[macro_use]
extern crate structopt;
extern crate bufstream;

use std::path::PathBuf;
use structopt::StructOpt;

use std::io::{Result,Error,ErrorKind,Seek,SeekFrom};
use std::net::{TcpListener, TcpStream};
use std::fs::{OpenOptions,File};

#[derive(Debug, StructOpt)]
struct Opt {
    /// Address to listen the port on
    #[structopt(short = "a", long = "addr", default_value = "127.0.0.1")]
    host: String,

    /// TCP port to listen
    #[structopt(short = "p", long = "port", default_value = "10809")]
    port: u16,
    
    /// Read-only mode
    #[structopt(short="r", long="read-only")]
    readonly: bool,
    
    // Size in bytes. May be omitted for regular files
    #[structopt(short="s", long="size")]
    size: Option<u64>,
    
    /// Hint clients that elevator algorithm should be used
    #[structopt(long="rotational")]
    rotational: bool,
    
    /// Convert TRIM operations to FALLOC_FL_PUNCH_HOLE or something (not implemented)
    #[structopt(long="trim")]
    trim: bool,
    
    /// Support RESIZE NBD extension (not implemented)
    #[structopt(long="resize")]
    resize: bool,
    
    /// Quiet mode, suppress non-error output
    #[structopt(short="q", long="quiet")]
    quiet: bool,
    
    /// File or device to be served
    #[structopt(parse(from_os_str))]
    file: PathBuf,
}

fn strerror(s: &'static str) -> Result<()> {
    let stderr: Box<::std::error::Error + Send + Sync> = s.into();
    Err(Error::new(ErrorKind::InvalidData, stderr))
}

fn handle_client(file: &mut File, size: u64, readonly: bool, stream: TcpStream) -> Result<()> {
    use nbd::server::{Export,handshake,transmission};
    let mut s = bufstream::BufStream::new(stream);
    let e = Export {
        size,
        readonly,
        ..Default::default()
    };
    handshake(&mut s, &e)?;
    transmission(&mut s, file)?;
    Ok(())
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    
    if opt.trim || opt.resize {
        strerror("This option is not supported yet")?;
    }
    
    let mut oo = OpenOptions::new();
    oo.read(true);
    
    if !opt.readonly {
        oo.write(true).create(true);
    }
    
    let mut f = oo.open(opt.file)?;
    
    let size = if let Some(s) = opt.size {
        s
    } else {
        let x = f.seek(SeekFrom::End(0))?;
        if x == 0 {
            eprintln!("warning: Use --size option to set size of the device. Serving zero-sized nbd now.");
        }
        x
    };
    
    let hostport = format!("{}:{}", opt.host, opt.port);
    let listener = TcpListener::bind(&hostport)?;
    
    if !opt.quiet {
        println!("Serving NBD on {}", hostport);
    }
    
    while let Ok((stream, addr)) = listener.accept() {
        if !opt.quiet {
            println!("A connection from {}", addr);
        }
        match handle_client(&mut f, size, opt.readonly, stream) {
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

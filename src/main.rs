use clap::Parser;
use image::{GenericImageView, Pixel};
use rand::random;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;
use tokio::io;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpSocket;
use tokio::time::{sleep, Instant};

const DEFAULT_HOST: &str = "151.217.15.90:1337";

#[derive(Parser)]
struct Args {
    /// Image file to broadcast
    #[arg(short, long)]
    image: String,
    /// host:port to connect to
    #[arg(short, long)]
    addr: Option<SocketAddr>,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let (path, addr) = {
        let args = Args::parse();
        let image = args.image;
        let addr: SocketAddr = args.addr.unwrap_or_else(|| DEFAULT_HOST.parse().unwrap());
        (image, addr)
    };

    let path = PathBuf::from(path);
    let img = image::open(path).expect("asshole");
    let (w, h) = (img.width(), img.height());
    let coords = {
        let mut coords: Vec<_> = ((0..w).flat_map(|x| (0..h).map(move |y| (x, y)))).collect();
        let num_px = w as usize * h as usize;
        for _ in 0..(w * h) {
            let a = random::<usize>() % num_px;
            let b = random::<usize>() % num_px;
            coords.swap(a, b);
        }
        coords
    };
    let cmd = {
        let mut cmd = String::new();
        for (x, y) in coords {
            let color = img.get_pixel(x, y);
            let (r, g, b) = (
                color.channels()[0],
                color.channels()[1],
                color.channels()[2],
            );
            let (x, y) = (x + 700, y + 100);
            let line = format!("PX {x} {y} {r:02x}{g:02x}{b:02x}\n");
            cmd.push_str(&line);
        }
        cmd
    };

    let sock = TcpSocket::new_v4()?;
    let mut stream = sock.connect(addr).await?;

    loop {
        let target_delta = Duration::from_micros(1_000_000 / 240);
        let before = Instant::now();
        stream.write_all(cmd.as_bytes()).await?;
        let delta = Instant::now() - before;
        if delta < target_delta {
            sleep(target_delta - delta).await;
        }
    }

    // Ok(())
}

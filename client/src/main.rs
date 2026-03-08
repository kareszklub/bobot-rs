use crate::packets::{RecvPacket, SendPacket};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

mod packets;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server_address = "192.168.50.59:1234";

    let stream = TcpStream::connect(server_address).await?;
    let (reader, mut writer) = stream.into_split();

    println!("Connected! Sending Welcome...");

    let read_task = tokio::spawn(async move {
        let mut buf_reader = BufReader::new(reader);
        let mut buf = Vec::new();

        loop {
            buf.clear();
            match buf_reader.read_until(0, &mut buf).await {
                Ok(0) => {
                    println!("Server closed the connection.");
                    break;
                }
                Ok(_) => {
                    if let Ok(pkt) = postcard::from_bytes_cobs::<SendPacket>(&mut buf) {
                        println!("Pico sent: {:?}", pkt);
                    } else {
                        eprintln!("Failed to parse incoming packet.");
                    }
                }
                Err(e) => {
                    eprintln!("Read error: {}", e);
                    break;
                }
            }
        }
    });

    let write_task = tokio::spawn(async move {
        let welcome = postcard::to_allocvec_cobs(&RecvPacket::WelcomeServer).unwrap();
        writer.write_all(&welcome).await.unwrap();

        println!("Welcome sent! Press [ENTER] in this console to toggle the Pico's LED.");

        let stdin = tokio::io::stdin();
        let mut reader = BufReader::new(stdin).lines();

        while let Ok(Some(_line)) = reader.next_line().await {
            let toggle = postcard::to_allocvec_cobs(&RecvPacket::CommandToggleLed(true)).unwrap();

            if let Err(e) = writer.write_all(&toggle).await {
                eprintln!("Failed to send toggle command: {}", e);
                break;
            }
            println!(">> Sent LED Toggle command to Pico!");
        }
    });

    let _ = tokio::join!(read_task, write_task);
    Ok(())
}

use std::{
    io::{self, Write, Read},
    net::{
        TcpListener,
        TcpStream,
        SocketAddr,
        Ipv4Addr
    }
};

use simple_file_server_bounty::http::request;

fn create_socket() -> SocketAddr {
    SocketAddr::new(std::net::IpAddr::V4(Ipv4Addr::LOCALHOST), 5500)
}

fn handle_client(stream: &mut TcpStream) -> io::Result<()> {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer)?;

    let buf_str = String::from_utf8_lossy(&buffer);
    let request = request::HttpRequest::new(&buf_str)?;
    let response = request.response()?;

    println!("Requested Path: {:?}", &response.current_path);
    // println!("{}", unsafe {String::from_utf8_unchecked(response.response_body.clone())});

    let mut body = response.response_body.clone();

    stream.write(&mut body)?;
    stream.flush()?;
    Ok(())
}

fn serve(socket: SocketAddr) -> io::Result<()> {
    println!("\nServer started at http://{}:{}", socket.ip(), socket.port());
    let listener = TcpListener::bind(socket)?;
    let mut counter = 0;
    for stream in listener.incoming() {
        match std::thread::spawn(|| handle_client(&mut stream?)).join() {
            Ok(_) => {
                counter += 1;
                println!("Connected stream... {}", counter);
            },
            Err(_) => continue,
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let socket = create_socket();
    serve(socket)?;
    Ok(())
}

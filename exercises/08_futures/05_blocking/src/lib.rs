// TODO: the `echo` server uses non-async primitives.
//  When running the tests, you should observe that it hangs, due to a
//  deadlock between the caller and the server.
//  Use `spawn_blocking` inside `echo` to resolve the issue.
use std::io::{Read, Write};
use tokio::net::{TcpListener, TcpStream};

pub async fn echo(listener: TcpListener) -> Result<(), anyhow::Error> {
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("{}", addr);
        // client_handle(socket);
        tokio::task::spawn_blocking(move || client_handle(socket));
    }
}

fn client_handle(socket: TcpStream) -> Result<(), anyhow::Error> {
    let mut socket = socket.into_std()?;
    socket.set_nonblocking(false)?;
    let mut buffer = Vec::new();
    socket.read_to_end(&mut buffer)?;
    socket.write_all(&buffer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::SocketAddr;
    use std::panic;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::task::JoinSet;

    async fn bind_random() -> (TcpListener, SocketAddr) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        (listener, addr)
    }

    #[tokio::test]
    async fn test_echo() {
        let (listener, addr) = bind_random().await;
        tokio::spawn(echo(listener));

        let large_data = vec![b'A'; 10 * 1024 * 1024]; // 10MB
        let requests = vec![large_data, vec![1], vec![1]];
        let mut join_set = JoinSet::new();

        for request in requests {
            join_set.spawn(async move {
                let mut socket = tokio::net::TcpStream::connect(addr).await.unwrap();
                let (mut reader, mut writer) = socket.split();

                // Send the request
                writer.write_all(request.as_slice()).await.unwrap();
                // Close the write side of the socket
                writer.shutdown().await.unwrap();

                // Read the response
                let mut buf = Vec::with_capacity(request.len());
                reader.read_to_end(&mut buf).await.unwrap();
                assert_eq!(&buf, request.as_slice());
            });
        }

        while let Some(outcome) = join_set.join_next().await {
            if let Err(e) = outcome {
                if let Ok(reason) = e.try_into_panic() {
                    panic::resume_unwind(reason);
                }
            }
        }
    }
}

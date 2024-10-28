use std::io;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

/// Separate thread for reading only, spawned by [`spawn_read_loop`].
async fn read_loop<F>(callback: F)
where
    F: Fn(Vec<u8>),
    F: Send + 'static,
{
    loop {
        if let Ok(input) = read().await {
            callback(input);
        }
    }
}

/// Asynchronously, using `tokio`, reads a message from stdin. Will first read the length, and then the message itself. Returns the message, if there is one, in a buffer.
pub async fn read() -> io::Result<Vec<u8>> {
    let mut stdin = tokio::io::stdin();
    let mut length = [0; 4];
    stdin.read_exact(&mut length).await?;
    let mut buffer = vec![0; u32::from_ne_bytes(length) as usize];
    stdin.read_exact(&mut buffer).await?;
    Ok(buffer)
}

/// Asynchronously, using `tokio`, writes a message to stdout. Writes the length of a buffer, and then the buffer.
pub async fn write(message: &[u8]) -> io::Result<()> {
    let mut stdout = tokio::io::stdout();
    let length = message.len() as u32;
    stdout.write_all(&length.to_ne_bytes()).await?;
    stdout.write_all(message).await?;
    stdout.flush().await?;
    Ok(())
}

/// Spawns a read loop that calls a given callback whenever a message is received.
/// Use of this function in particular to handle feedback from the browser is not required, but recommended.
/// This function does not block! The read loop is spawned in the background and calls your callback only when a message is received.
/// Generally, writing as needed will happen in your main thread after this function is called.
pub fn spawn_read_loop<F>(read_callback: F)
where
    F: Fn(Vec<u8>),
    F: Send + 'static,
{
    tokio::spawn(read_loop(read_callback));
}

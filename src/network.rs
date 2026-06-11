use std::io;

#[allow(unused)]
pub struct NetworkInterface {
    handle: rtnetlink::Handle,
}

impl NetworkInterface {
    #[allow(unused)]
    pub fn new() -> Result<Self, io::Error> {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(connection);

        Ok(Self { handle })
    }
}

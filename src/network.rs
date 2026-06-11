use error::CraterError;

#[allow(unused)]
pub struct NetworkInterface {
    handle: rtnetlink::Handle,
}

impl NetworkInterface {
    #[allow(unused)]
    pub fn new() -> Result<Self, CraterError> {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(connection);

        Ok(Self { handle })
    }
}

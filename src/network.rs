use error::CraterError;
use futures::TryStreamExt;
use rtnetlink::packet_route::link::{LinkFlags, LinkMessage};

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

    #[allow(unused)]
    pub async fn set_loopback_up(&self) -> Result<(), CraterError> {
        let mut links = self
            .handle
            .link()
            .get()
            .match_name("lo".to_string())
            .execute();

        if let Some(link) = links.try_next().await? {
            let mut link_msg = LinkMessage::default();
            link_msg.header.index = link.header.index;
            link_msg.header.flags = LinkFlags::Up;
            link_msg.header.change_mask = LinkFlags::Up;

            self.handle.link().set(link_msg).execute().await?;
        }

        Ok(())
    }
}

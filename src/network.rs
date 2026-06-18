use error::CraterError;
use futures::TryStreamExt;
use rtnetlink::{
    packet_route::link::{LinkAttribute, LinkFlags, LinkMessage},
    LinkBridge, LinkVeth, RouteMessageBuilder,
};
use std::{
    marker::PhantomData,
    net::{IpAddr, Ipv4Addr} 
};

pub struct Bridge;

impl Bridge {
    const BRIDGE_NAME: &str = "crater-br";

    pub async fn create() -> Result<(), CraterError> {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(connection);

        let req = LinkBridge::new(Self::name()).build();

        match handle.link().add(req).execute().await {
            Ok(()) => Ok(()), 
            Err(err) => {
                if let rtnetlink::Error::NetlinkError(e) = &err && let Some(code) = e.code && code.get() == -17 {
                    Ok(())
                } else {
                    Err(err.into())
                }
            },
        }
    }

    pub fn name() -> &'static str {
        Self::BRIDGE_NAME
    }
}

pub struct NetworkInterface<S: InterfaceSide> {
    handle: rtnetlink::Handle,
    _interface_side: PhantomData<S>,
}

impl<S: InterfaceSide> NetworkInterface<S> {
    pub fn new() -> Result<Self, CraterError> {
        let (connection, handle, _) = rtnetlink::new_connection()?;
        tokio::spawn(connection);

        Ok(Self {
            handle,
            _interface_side: PhantomData,
        })
    }

    pub async fn assign_address(
        &self,
        veth_end_name: &str,
        addr: IpAddr,
        addr_prefix: u8,
    ) -> Result<(), CraterError> {
        let if_idx = self
            .get_interface_index(veth_end_name)
            .await?
            .expect("Interface not found something is wrong in program logic");

        self.handle
            .address()
            .add(if_idx, addr, addr_prefix)
            .execute()
            .await?;

        Ok(())
    }

    pub async fn set_link_up(&self, name: &str) -> Result<(), CraterError> {
        let mut links = self
            .handle
            .link()
            .get()
            .match_name(name.to_string())
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

    async fn get_interface_index(&self, name: &str) -> Result<Option<u32>, CraterError> {
        let mut links = self
            .handle
            .link()
            .get()
            .match_name(name.to_string())
            .execute();

        let if_idx = links.try_next().await?.map(|l| l.header.index);

        Ok(if_idx)
    }
}

impl NetworkInterface<ContainerSideInterface> {
    pub async fn set_loopback_up(&self) -> Result<(), CraterError> {
        self.set_link_up("lo").await
    }

    pub async fn add_default_route(&self, addr: Ipv4Addr) -> Result<(), CraterError> {
        let route_msg = RouteMessageBuilder::<Ipv4Addr>::new().gateway(addr).build();

        self.handle.route().add(route_msg).execute().await?;

        Ok(())
    }
}

impl NetworkInterface<HostSideInterface> {
    pub async fn create_veth_pair(
        &self,
        host_end: &str,
        container_end: &str,
    ) -> Result<(), CraterError> {
        let link_req = LinkVeth::new(host_end, container_end).build();

        self.handle.link().add(link_req).execute().await?;

        Ok(())
    }

    pub async fn link_to_bridge(&self, host_end: &str) -> Result<(), CraterError> {
        let bridge_idx = self
            .get_interface_index(Bridge::name())
            .await?
            .expect("Bridge does not exist");
        let host_side_veth_idx = self
            .get_interface_index(host_end)
            .await?
            .expect("Host side veth with given name doesn't exist");

        let mut msg = LinkMessage::default();
        msg.header.index = host_side_veth_idx;
        msg.attributes.push(LinkAttribute::Controller(bridge_idx));

        self.handle.link().set(msg).execute().await?;

        Ok(())
    }

    pub async fn move_veth_pair_end(&self, name: &str, pid: u32) -> Result<(), CraterError> {
        let mut link_msg = LinkMessage::default();

        let if_idx = self
            .get_interface_index(name)
            .await?
            .expect("Interface not found something is wrong in program logic");

        link_msg.header.index = if_idx;
        link_msg.attributes.push(LinkAttribute::NetNsPid(pid));

        self.handle.link().set(link_msg).execute().await?;

        Ok(())
    }
}

pub trait InterfaceSide {}

pub struct HostSideInterface;
impl InterfaceSide for HostSideInterface {}

pub struct ContainerSideInterface;
impl InterfaceSide for ContainerSideInterface {}

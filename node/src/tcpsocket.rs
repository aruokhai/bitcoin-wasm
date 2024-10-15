use std::sync::Arc;

use wasi::{io::poll, sockets::tcp::{ErrorCode, InputStream, IpSocketAddress, Network, OutputStream, TcpSocket}};

pub struct WasiTcpSocket {
    inner: Arc<TcpSocket>,
    network_ref: Network,

}

impl  WasiTcpSocket  {
    
    pub fn new( inner: TcpSocket, network_ref: Network) ->  Self {
        WasiTcpSocket{inner: Arc::new(inner),  network_ref}
    }

    pub fn blocking_connect(
        &self,
        remote_address: IpSocketAddress,
    ) -> Result<(InputStream, OutputStream), ErrorCode> {
            let sub: poll::Pollable = self.inner.subscribe();
            self.inner.start_connect(&self.network_ref, remote_address)?;
            loop {
                match self.inner.finish_connect() {
                    Err(ErrorCode::WouldBlock) => sub.block(),
                    result => return result,
                }
            }
    }
   

}
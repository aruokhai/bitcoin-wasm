use std::sync::Arc;

use wasi::{io::poll, sockets::tcp::{ErrorCode, InputStream, IpSocketAddress, Network, OutputStream, TcpSocket}};

pub struct WasiTcpSocket {
    inner: Arc<TcpSocket>,
    socket_address: IpSocketAddress,
    network_ref: Network,


}

impl  WasiTcpSocket  {
    
    fn new( inner: TcpSocket, socket_address: IpSocketAddress , network_ref: Network) ->  Self {
        return WasiTcpSocket{inner: Arc::new(inner), socket_address,  network_ref: network_ref}
    }

    pub fn blocking_bind(
        &self,
    ) -> Result<(), ErrorCode> {
        let sub = self.inner.subscribe();

        self.inner.start_bind(&self.network_ref, self.socket_address)?;

        loop {
            match self.inner.finish_bind() {
                Err(ErrorCode::WouldBlock) => sub.block(),
                result => return result,
            }
        }
    }

    pub fn blocking_listen(&self) -> Result<(), ErrorCode> {
        let sub: poll::Pollable = self.inner.subscribe();

        self.inner.start_listen()?;

        loop {
            match self.inner.finish_listen() {
                Err(ErrorCode::WouldBlock) => sub.block(),
                result => return result,
            }
        }
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

    // pub async fn accept(&self) -> Result<(TcpSocket, InputStream, OutputStream), wasi::sockets::network::ErrorCode> {
    //     let timeout = monotonic_clock::subscribe_duration(TIMEOUT_NS);
    //     let sub = self.inner.subscribe();
    //     return AsyncWasiTcpListener{ inner: self.inner.clone(), sub_pollable: sub, time_pollable: timeout,async_type: AsyncWasiTcpListenerType::Waiting  }.await;
    // }
   

}
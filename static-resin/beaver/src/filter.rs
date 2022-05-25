use std::net::{IpAddr};
use std::any::Any;

pub enum Context {
    File(FileContext),
    ClientNetwork(RemoteConnectContext),
    ServerNetwork(ListenConnectionsContext),
    KVContext(std::collections::HashMap<String, String>),
    CustomContext(Box<dyn Any + 'static>)
}

pub trait CustomContext {
    fn as_any(&self) -> &dyn Any;
}

// Possible extension: infer from file object? 
pub struct FileContext {
    pub file_name: String,
    pub path: String,
}

pub struct RemoteConnectContext {
    pub remote_ip_address: IpAddr,
    pub port: u16,
}

// TODO: Flesh out use case for this; do we need this? 
pub struct ListenConnectionsContext {
    _ip_address: IpAddr,
}



#[macro_export]
macro_rules! kv_ctx {
    ($($k:literal => $v:expr),* $(,)?) => {{
        let mut m = std::collections::HashMap::new();
        $(m.insert($k.to_string(), $v.to_string()));*
        ;
        $crate::filter::Context::KVContext(m)
    }};
}


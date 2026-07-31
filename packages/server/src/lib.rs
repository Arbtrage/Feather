pub mod config;
pub mod domain;
pub mod grpc;
pub mod http;
pub mod observability;
pub mod services;
pub mod storage;

pub mod proto {
    pub mod feather {
        pub mod v1 {
            tonic::include_proto!("feather.v1");
        }
    }
}

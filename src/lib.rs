extern crate futures;
extern crate grpcio;
extern crate rust_xds_grpc;

#[macro_use]
extern crate log;

use grpcio::{DuplexSink, RequestStream, RpcContext, RpcStatus, RpcStatusCode, UnarySink};

use rust_xds_grpc::discovery::{
    DiscoveryRequest, DiscoveryResponse, IncrementalDiscoveryRequest, IncrementalDiscoveryResponse,
};

use rust_xds_grpc::ads_grpc::AggregatedDiscoveryService;
use rust_xds_grpc::cds_grpc::ClusterDiscoveryService;
use rust_xds_grpc::eds_grpc::EndpointDiscoveryService;
use rust_xds_grpc::lds_grpc::ListenerDiscoveryService;
use rust_xds_grpc::rds_grpc::RouteDiscoveryService;
use rust_xds_grpc::sds_grpc::SecretDiscoveryService;

use futures::Future;

#[derive(Clone)]
struct DiscoveryServer;

impl DiscoveryServer {
    fn stream(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        ctx.spawn(
            sink.fail(RpcStatus::new(RpcStatusCode::Unimplemented, None))
                .map_err(|err| error!("stream: {:?}", err)),
        )
    }

    fn fetch(
        &mut self,
        ctx: RpcContext,
        req: DiscoveryRequest,
        sink: UnarySink<DiscoveryResponse>,
    ) {
        ctx.spawn(
            sink.fail(RpcStatus::new(RpcStatusCode::Unimplemented, None))
                .map_err(|err| error!("fetch: {:?}", err)),
        )
    }
}

impl ClusterDiscoveryService for DiscoveryServer {
    fn stream_clusters(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        self.stream(ctx, stream, sink);
    }

    fn incremental_clusters(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<IncrementalDiscoveryRequest>,
        sink: DuplexSink<IncrementalDiscoveryResponse>,
    ) {
        ctx.spawn(
            sink.fail(RpcStatus::new(RpcStatusCode::Unimplemented, None))
                .map_err(|err| error!("incremental_clusters: {:?}", err)),
        )
    }

    fn fetch_clusters(
        &mut self,
        ctx: RpcContext,
        req: DiscoveryRequest,
        sink: UnarySink<DiscoveryResponse>,
    ) {
        self.fetch(ctx, req, sink);
    }
}

impl RouteDiscoveryService for DiscoveryServer {
    fn stream_routes(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        self.stream(ctx, stream, sink);
    }

    fn incremental_routes(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<IncrementalDiscoveryRequest>,
        sink: DuplexSink<IncrementalDiscoveryResponse>,
    ) {
        ctx.spawn(
            sink.fail(RpcStatus::new(RpcStatusCode::Unimplemented, None))
                .map_err(|err| error!("incremental_routes: {:?}", err)),
        )
    }

    fn fetch_routes(
        &mut self,
        ctx: RpcContext,
        req: DiscoveryRequest,
        sink: UnarySink<DiscoveryResponse>,
    ) {
        self.fetch(ctx, req, sink);
    }
}

impl EndpointDiscoveryService for DiscoveryServer {
    fn stream_endpoints(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        self.stream(ctx, stream, sink);
    }

    fn fetch_endpoints(
        &mut self,
        ctx: RpcContext,
        req: DiscoveryRequest,
        sink: UnarySink<DiscoveryResponse>,
    ) {
        self.fetch(ctx, req, sink);
    }
}

impl ListenerDiscoveryService for DiscoveryServer {
    fn stream_listeners(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        self.stream(ctx, stream, sink);
    }

    fn fetch_listeners(
        &mut self,
        ctx: RpcContext,
        req: DiscoveryRequest,
        sink: UnarySink<DiscoveryResponse>,
    ) {
        self.fetch(ctx, req, sink);
    }
}

impl AggregatedDiscoveryService for DiscoveryServer {
    fn stream_aggregated_resources(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        self.stream(ctx, stream, sink);
    }

    fn incremental_aggregated_resources(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<IncrementalDiscoveryRequest>,
        sink: DuplexSink<IncrementalDiscoveryResponse>,
    ) {
        ctx.spawn(
            sink.fail(RpcStatus::new(RpcStatusCode::Unimplemented, None))
                .map_err(|err| error!("incremental_aggregated_resources: {:?}", err)),
        )
    }
}

impl SecretDiscoveryService for DiscoveryServer {
    fn stream_secrets(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        self.stream(ctx, stream, sink);
    }

    fn fetch_secrets(
        &mut self,
        ctx: RpcContext,
        req: DiscoveryRequest,
        sink: UnarySink<DiscoveryResponse>,
    ) {
        self.fetch(ctx, req, sink);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use grpcio::{Environment, ServerBuilder};
    use rust_xds_grpc::cds_grpc::create_cluster_discovery_service;
    use std::sync::Arc;

    use futures::sync::oneshot;
    use futures::Future;
    use std::io::Read;
    use std::{io, thread};

    #[test]
    fn it_works() {
        let env = Arc::new(Environment::new(1));
        let service = create_cluster_discovery_service(DiscoveryServer);
        let mut server = ServerBuilder::new(env)
            .register_service(service)
            .bind("127.0.0.1", 9090)
            .build()
            .unwrap();

        server.start();

        // Block until newline on stdin.
        let (tx, rx) = oneshot::channel();
        thread::spawn(move || {
            let _ = io::stdin().read(&mut [0]).unwrap();
            tx.send(())
        });
        let _ = rx.wait();
        let _ = server.shutdown().wait();
    }
}

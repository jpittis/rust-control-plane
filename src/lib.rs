mod cache;

extern crate futures;
extern crate grpcio;
extern crate protobuf;
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

use futures::sync::mpsc;
use futures::Future;

trait Cache {
    fn fetch(&mut self, req: DiscoveryRequest) -> DiscoveryResponse;
    fn watch(&mut self, req: DiscoveryRequest) -> mpsc::Receiver<DiscoveryResponse>;
}

#[derive(Clone)]
struct DiscoveryServer<C: Cache> {
    cache: C,
}

impl<C: Cache> DiscoveryServer<C> {
    fn new(cache: C) -> DiscoveryServer<C> {
        DiscoveryServer { cache }
    }

    fn stream(
        &mut self,
        ctx: RpcContext,
        stream: RequestStream<DiscoveryRequest>,
        sink: DuplexSink<DiscoveryResponse>,
    ) {
        // TODO: Probably don't want to use a channel here.
        // let sender, receiver = mpsc::channel(0);
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
        let resp = self.cache.fetch(req);
        ctx.spawn(sink.success(resp).map_err(|err| error!("fetch: {:?}", err)))
    }
}

impl<C: Cache> ClusterDiscoveryService for DiscoveryServer<C> {
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

impl<C: Cache> RouteDiscoveryService for DiscoveryServer<C> {
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

impl<C: Cache> EndpointDiscoveryService for DiscoveryServer<C> {
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

impl<C: Cache> ListenerDiscoveryService for DiscoveryServer<C> {
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

impl<C: Cache> AggregatedDiscoveryService for DiscoveryServer<C> {
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

impl<C: Cache> SecretDiscoveryService for DiscoveryServer<C> {
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

    use grpcio::{ChannelBuilder, EnvBuilder, Environment, ServerBuilder};
    use rust_xds_grpc::cds_grpc::{
        create_cluster_discovery_service, ClusterDiscoveryServiceClient,
    };
    use std::sync::Arc;

    use futures::sync::oneshot;
    use futures::Future;
    use std::io::Read;
    use std::{io, thread};

    #[derive(Clone)]
    struct DummyCache {
        resp: DiscoveryResponse,
    }

    impl DummyCache {
        fn new(resp: DiscoveryResponse) -> DummyCache {
            DummyCache { resp }
        }
    }

    impl Cache for DummyCache {
        fn fetch(&mut self, req: DiscoveryRequest) -> DiscoveryResponse {
            self.resp.clone()
        }
    }

    #[test]
    fn it_works() {
        let mut resp = DiscoveryResponse::new();
        resp.set_version_info("foobar".to_string());
        let cache = DummyCache::new(resp.clone());
        let s_env = Arc::new(Environment::new(1));
        let discovery_server = DiscoveryServer::new(cache);
        let service = create_cluster_discovery_service(discovery_server);
        let mut server = ServerBuilder::new(s_env)
            .register_service(service)
            .bind("127.0.0.1", 9090)
            .build()
            .unwrap();

        server.start();

        let c_env = Arc::new(EnvBuilder::new().build());
        let ch = ChannelBuilder::new(c_env).connect("127.0.0.1:9090");
        let client = ClusterDiscoveryServiceClient::new(ch);

        let req = DiscoveryRequest::new();
        assert_eq!(resp, client.fetch_clusters(&req).unwrap());

        let _ = server.shutdown().wait();
    }
}

use cached::Cached;

use futures::sync::oneshot;

use protobuf::Message;

use rust_xds_grpc::base::Node;
use rust_xds_grpc::discovery::DiscoveryRequest;

use rust_xds_grpc::cds::Cluster;
use rust_xds_grpc::eds::ClusterLoadAssignment;
use rust_xds_grpc::lds::Listener;
use rust_xds_grpc::rds::RouteConfiguration;
use rust_xds_grpc::sds::SdsDummy;

// Alias the protobuf messages to their common names.
type Endpoint = ClusterLoadAssignment;
type Secret = SdsDummy;
type Route = RouteConfiguration;

struct Resource<M: Message> {
    name: String,
    message: M,
}

struct Resources<M: Message> {
    items: Vec<Resource<M>>,
}

struct Snapshot {
    version: String,
    clusters: Resources<Cluster>,
    endpoints: Resources<Endpoint>,
    listeners: Resources<Listener>,
    routes: Resources<Route>,
    secrets: Resources<Secret>,
}

trait NodeHash {
    fn hash(&self, node: &Node) -> String;
}

struct SnapshotCache<C: Cached<String, Snapshot>, H: NodeHash> {
    cached: C,
    node_hash: H,
}

#[derive(Debug)]
enum SnapshotCacheError {
    SkipFetch,
    MissingKey,
}

impl<C: Cached<String, Snapshot>, H: NodeHash> SnapshotCache<C, H> {
    fn fetch(&mut self, req: &DiscoveryRequest) -> Result<&Snapshot, SnapshotCacheError> {
        let key = self.node_hash.hash(req.get_node());
        match self.cached.cache_get(&key) {
            Some(snapshot) => {
                if req.get_version_info() == snapshot.version {
                    return Err(SnapshotCacheError::SkipFetch);
                }
                Ok(snapshot)
            }
            None => Err(SnapshotCacheError::MissingKey),
        }
    }
}

trait Cache {
    fn fetch_endpoints(
        &mut self,
        req: &DiscoveryRequest,
    ) -> Result<Response<Endpoint>, SnapshotCacheError>;
    fn watch_endpoints(
        &mut self,
        req: &DiscoveryRequest,
    ) -> Result<oneshot::Receiver<Response<Endpoint>>, SnapshotCacheError>;
}

struct Response<'a, M: Message> {
    version: String,
    resources: &'a Resources<M>,
}

impl<C: Cached<String, Snapshot>, H: NodeHash> Cache for SnapshotCache<C, H> {
    fn fetch_endpoints(
        &mut self,
        req: &DiscoveryRequest,
    ) -> Result<Response<Endpoint>, SnapshotCacheError> {
        self.fetch(req).map(|snapshot| build_response(req, &snapshot.endpoints))
    }

    fn watch_endpoints(
        &mut self,
        req: &DiscoveryRequest,
    ) -> Result<oneshot::Receiver<Response<Endpoint>>, SnapshotCacheError> {
        let key = self.node_hash.hash(req.get_node());

        // TODO: Pull out the status for this node.
        // TODO: Create a new status if one does not exist.
        // TODO: Mark the current time as the last_watch_time on the status.

        let (p, c) = oneshot::channel::<Response<Endpoint>>();

        match self.fetch_endpoints(req) {
            Ok(resp) => p.send(resp).map_err(|err| unreachable!()),
            Err(_) => {
                // TODO: Grab the next watch ID.
                // TODO: Store the producer under the watch ID.
                unimplemented!()
            }
        };

        Ok(c)
    }
}

fn build_response<'a, M: Message>(
    req: &DiscoveryRequest,
    resources: &'a Resources<M>,
) -> Response<'a, M> {
    Response {
        version: req.version_info.clone(),
        resources: resources,
    }
}

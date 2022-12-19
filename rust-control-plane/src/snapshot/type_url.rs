macro_rules! prefix {
    ($type:literal) => {
        concat!("type.googleapis.com/", $type)
    };
}

pub const ENDPOINT: &str = prefix!("envoy.config.endpoint.v3.ClusterLoadAssignment");
pub const CLUSTER: &str = prefix!("envoy.config.cluster.v3.Cluster");
pub const ROUTE: &str = prefix!("envoy.config.route.v3.RouteConfiguration");
pub const VIRTUAL_HOST: &str = prefix!("envoy.config.route.v3.VirtualHost");
pub const LISTENER: &str = prefix!("envoy.config.listener.v3.Listener");
pub const SECRET: &str = prefix!("envoy.extensions.transport_sockets.tls.v3.Secret");
pub const RUNTIME: &str = prefix!("envoy.service.runtime.v3.Runtime");
pub const SCOPED_ROUTE: &str = prefix!("envoy.config.route.v3.ScopedRouteConfiguration");
pub const EXTENSION_CONFIG: &str = prefix!("envoy.config.core.v3.TypedExtensionConfig");

pub const ANY_TYPE: &str = "";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_concatinates_valid_type() {
        assert_eq!(
            CLUSTER,
            "type.googleapis.com/envoy.config.cluster.v3.Cluster"
        )
    }
}

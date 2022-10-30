macro_rules! prefix {
    ($type:literal) => {
        concat!("type.googleapis.com/", $type)
    };
}

pub const ENDPOINT: &'static str = prefix!("envoy.config.endpoint.v3.ClusterLoadAssignment");
pub const CLUSTER: &'static str = prefix!("envoy.config.cluster.v3.Cluster");
pub const ROUTE: &'static str = prefix!("envoy.config.route.v3.RouteConfiguration");
pub const SCOPEDROUTE: &'static str = prefix!("envoy.config.route.v3.ScopedRouteConfiguration");
pub const VIRTUALHOST: &'static str = prefix!("envoy.config.route.v3.VirtualHost");
pub const LISTENER: &'static str = prefix!("envoy.config.listener.v3.Listener");
pub const SECRET: &'static str = prefix!("envoy.extensions.transport_sockets.tls.v3.Secret");
pub const EXTENSIONCONFIG: &'static str = prefix!("envoy.config.core.v3.TypedExtensionConfig");
pub const RUNTIME: &'static str = prefix!("envoy.service.runtime.v3.Runtime");

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

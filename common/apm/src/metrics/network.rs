use lazy_static::lazy_static;

use crate::metrics::{
    auto_flush_from, exponential_buckets, linear_buckets, make_auto_flush_static_metric,
    register_counter_vec, register_histogram_vec, register_int_counter, register_int_gauge,
    register_int_gauge_vec, CounterVec, HistogramVec, IntCounter, IntGauge, IntGaugeVec,
};

make_auto_flush_static_metric! {
    pub label_enum MessageDirection {
        sent,
        received,
    }

    pub label_enum ProtocolKind {
        rpc,
    }

    pub label_enum RPCResult {
        success,
        timeout,
    }

    pub label_enum MessageTaret {
      single,
      multi,
      all
    }

    pub struct MessageCounterVec: LocalIntCounter {
        "direction" => MessageDirection,
    }

    pub struct RPCResultCounterVec: LocalCounter {
        "result" => RPCResult,
    }

    pub struct ProtocolTimeHistogramVec: LocalHistogram {
        "type" => ProtocolKind,
    }
}

lazy_static! {
    pub static ref NETWORK_MESSAGE_COUNT_VEC: CounterVec = register_counter_vec!(
        "axon_network_message_total",
        "Total number of network message",
        &["direction", "target", "type", "module", "action"]
    )
    .expect("network message total");
    pub static ref NETWORK_MESSAGE_SIZE_COUNT_VEC: CounterVec = register_counter_vec!(
        "axon_network_message_size",
        "Accumulated compressed network message size",
        &["direction", "url"]
    )
    .expect("network message size");
    pub static ref NETWORK_RPC_RESULT_COUNT_VEC: CounterVec = register_counter_vec!(
        "axon_network_rpc_result_total",
        "Total number of network rpc result",
        &["result"]
    )
    .expect("network rpc result total");
    pub static ref NETWORK_PROTOCOL_TIME_HISTOGRAM_VEC: HistogramVec = register_histogram_vec!(
        "axon_network_protocol_time_cost_seconds",
        "Network protocol time cost",
        &["type"],
        exponential_buckets(0.01, 2.0, 20).expect("network protocol time expontial")
    )
    .expect("network protocol time cost");
    pub static ref NETWORK_PING_HISTOGRAM_VEC: HistogramVec = register_histogram_vec!(
        "axon_network_ping_in_ms",
        "Network peer ping time",
        &["ip"],
        linear_buckets(100.0, 200.0, 5).expect("network ping time linear buckets")
    )
    .expect("network ping time");
}

lazy_static! {
    pub static ref NETWORK_RPC_RESULT_COUNT_VEC_STATIC: RPCResultCounterVec =
        auto_flush_from!(NETWORK_RPC_RESULT_COUNT_VEC, RPCResultCounterVec);
    pub static ref NETWORK_PROTOCOL_TIME_HISTOGRAM_VEC_STATIC: ProtocolTimeHistogramVec = auto_flush_from!(
        NETWORK_PROTOCOL_TIME_HISTOGRAM_VEC,
        ProtocolTimeHistogramVec
    );
}

lazy_static! {
    pub static ref NETWORK_TOTAL_PENDING_DATA_SIZE: IntGauge = register_int_gauge!(
        "axon_network_total_pending_data_size",
        "Total pending data size"
    )
    .expect("network total pending data size");
    pub static ref NETWORK_IP_PENDING_DATA_SIZE_VEC: IntGaugeVec = register_int_gauge_vec!(
        "axon_network_ip_pending_data_size",
        "IP pending data size",
        &["ip"]
    )
    .expect("network ip pending data size");
    pub static ref NETWORK_RECEIVED_MESSAGE_IN_PROCESSING_GUAGE: IntGauge = register_int_gauge!(
        "axon_network_received_message_in_processing_guage",
        "Total number of network received message current in processing"
    )
    .expect("network received message in processing");
    pub static ref NETWORK_RECEIVED_IP_MESSAGE_IN_PROCESSING_GUAGE_VEC: IntGaugeVec =
        register_int_gauge_vec!(
            "axon_network_received_ip_message_in_processing_guage",
            "Number of network received messasge from ip current in processing",
            &["ip"]
        )
        .expect("network received ip message in processing");
    pub static ref NETWORK_CONNECTED_PEERS: IntGauge =
        register_int_gauge!("axon_network_connected_peers", "Total connected peer count")
            .expect("network total connecteds");
    pub static ref NETWORK_IP_DISCONNECTED_COUNT_VEC: CounterVec = register_counter_vec!(
        "axon_network_ip_disconnected_count",
        "Total number of ip disconnected count",
        &["ip"]
    )
    .expect("network disconnect ip count");
    pub static ref NETWORK_OUTBOUND_CONNECTING_PEERS: IntGauge = register_int_gauge!(
        "axon_network_outbound_connecting_peers",
        "Total number of network outbound connecting peers"
    )
    .expect("network outbound connecting peer count");
    pub static ref NETWORK_UNIDENTIFIED_CONNECTIONS: IntGauge = register_int_gauge!(
        "axon_network_unidentified_connections",
        "Total number of network unidentified connections"
    )
    .expect("network unidentified connections");
    pub static ref NETWORK_SAVED_PEER_COUNT: IntCounter = register_int_counter!(
        "axon_network_saved_peer_count",
        "Total number of saved peer count"
    )
    .expect("network saved peer count");
    pub static ref NETWORK_TAGGED_CONSENSUS_PEERS: IntGauge = register_int_gauge!(
        "axon_network_tagged_consensus_peers",
        "Total number of consensus peers"
    )
    .expect("network tagged consensus peers");
    pub static ref NETWORK_CONNECTED_CONSENSUS_PEERS: IntGauge = register_int_gauge!(
        "axon_network_connected_consensus_peers",
        "Total number of connected consensus peers"
    )
    .expect("network connected consenss peers");
}

fn on_network_message(direction: &str, target: &str, url: &str, inc: f64) {
    let spliced: Vec<&str> = url.split('/').collect();
    if spliced.len() < 4 {
        return;
    }

    let network_type = spliced[1];
    let module = spliced[2];
    let action = spliced[3];

    NETWORK_MESSAGE_COUNT_VEC
        .with_label_values(&[direction, target, network_type, module, action])
        .inc_by(inc);
}

pub fn on_network_message_sent_all_target(url: &str) {
    on_network_message("sent", "all", url, 1.0)
}

pub fn on_network_message_sent_multi_target(url: &str, target_count: f64) {
    on_network_message("sent", "single", url, target_count);
}

pub fn on_network_message_sent(url: &str) {
    on_network_message("sent", "single", url, 1.0);
}

pub fn on_network_message_received(url: &str) {
    on_network_message("received", "single", url, 1.0);
}

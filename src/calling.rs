use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    time::Duration,
};

use crate::api::{AccessToken, RegionGtms};
use anyhow::{Context, Result};
use base64::Engine;
use reqwest::Client;
use tokio::net::UdpSocket;
use uuid::Uuid;

// TODO: Derive from config for other tenants.
pub const SKYPE_CLIENT_HEADER: &str = "SkypeSpaces/1415/teams-cli/TsCallingVersion=2025.49.01.15";
pub const TEAMS_PARTITION: &str = "amer03";
pub const TEAMS_REGION: &str = "amer";
pub const TEAMS_RING: &str = "general";
pub const VIDEO_SSRC_RANGE_SIZE: u32 = 100;
pub const DEFAULT_STUN_SERVER: &str = "stun.l.google.com:19302";

/// STUN magic cookie (RFC 5389).
const MAGIC_COOKIE: u32 = 0x2112A442;

/// STUN message types.
const BINDING_REQUEST: u16 = 0x0001;
const BINDING_RESPONSE: u16 = 0x0101;
const BINDING_ERROR_RESPONSE: u16 = 0x0111;

/// STUN attribute types.
const ATTR_MAPPED_ADDRESS: u16 = 0x0001;
const ATTR_USERNAME: u16 = 0x0006;
const ATTR_MESSAGE_INTEGRITY: u16 = 0x0008;
const ATTR_XOR_MAPPED_ADDRESS: u16 = 0x0020;
const ATTR_FINGERPRINT: u16 = 0x8028;
const ATTR_PRIORITY: u16 = 0x0024;
const ATTR_USE_CANDIDATE: u16 = 0x0025;
const ATTR_ICE_CONTROLLED: u16 = 0x8029;
const ATTR_ICE_CONTROLLING: u16 = 0x802A;

/// STUN header size (type + length + magic + transaction ID).
const STUN_HEADER_SIZE: usize = 20;

/// FINGERPRINT XOR constant per RFC 5389.
const FINGERPRINT_XOR: u32 = 0x5354554e;

/// ICE connectivity check timeout per attempt.
const CHECK_TIMEOUT: Duration = Duration::from_millis(500);

/// Maximum retry count for connectivity checks.
const CHECK_MAX_RETRIES: u32 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct ConversationCallParams {
    pub trouter_surl: String,
    pub caller_mri: String,
    pub caller_display_name: String,
    pub endpoint_id: String,
    pub participant_id: String,
    pub thread_id: String,
    pub chain_id: String,
    pub message_id: String,
    pub caller_oid: String,
    pub tenant_id: String,
}

#[derive(Debug)]
pub struct ConversationCreated {
    pub conversation_controller: String,
    pub add_participant_url: Option<String>,
}

#[derive(Debug)]
pub struct ConversationJoined {
    pub cc_active_url: Option<String>,
}

/// ICE transport protocol.
#[derive(Debug, Clone, PartialEq)]
pub enum Transport {
    Udp,
    TcpActive,
    TcpPassive,
}

#[derive(Debug, Clone)]
pub enum CandidateType {
    Host,
    ServerReflexive,
    Relay,
}

#[derive(Debug, Clone)]
pub struct IceCandidate {
    pub foundation: String,
    pub component: u8,
    pub transport: Transport,
    pub priority: u32,
    pub address: String,
    pub port: u16,
    pub candidate_type: CandidateType,
    pub raddr: Option<String>,
    pub rport: Option<u16>,
}

#[derive(Debug, Clone)]
pub struct AvSdpResult {
    pub sdp: String,
    pub audio_crypto_line: String,
    pub video_crypto_line: String,
    pub audio_ufrag: String,
    pub audio_pwd: String,
    pub video_ufrag: String,
    pub video_pwd: String,
}

#[derive(Debug, Clone)]
pub struct AvSdpParams<'a> {
    pub local_ip: &'a str,
    pub audio_port: u16,
    pub video_port: u16,
    pub audio_ufrag: &'a str,
    pub audio_pwd: &'a str,
    pub video_ufrag: &'a str,
    pub video_pwd: &'a str,
    pub audio_candidates: &'a [IceCandidate],
    pub video_candidates: &'a [IceCandidate],
    /// Base SSRC for the video x-ssrc-range attribute.
    pub video_ssrc_base: u32,
    /// SSRC for audio x-ssrc-range attribute.
    pub audio_ssrc: u32,
}

/// Generate a random 4-character ICE ufrag.
pub fn generate_ice_ufrag() -> String {
    use std::fmt::Write;
    let bytes: [u8; 3] = rand::random();
    let mut s = String::with_capacity(4);
    for b in &bytes {
        write!(s, "{:02x}", b).unwrap();
    }
    s.truncate(4);
    s
}

pub fn generate_ssrc() -> u32 {
    rand::random()
}

pub fn generate_ice_pwd() -> String {
    let bytes: [u8; 12] = rand::random();
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn derive_epconv_url(region_gtms: &RegionGtms) -> Option<String> {
    if let Some(url) = &region_gtms.conversation_service_url {
        return Some(url.clone());
    }

    // Fallback: derive from potentialCallRequestUrl
    let potential_url = region_gtms.potential_call_request_url.as_deref()?;

    if let Some(idx) = potential_url.find("/api/v2/") {
        let base = &potential_url[..idx];
        Some(format!("{}/api/v2/epconv", base))
    } else {
        Some(potential_url.replace("/cc/v1/potentialcall", "/epconv"))
    }
}

fn generate_srtp_key() -> String {
    let bytes: [u8; 30] = rand::random();
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn to_sdp_line(candidate: &IceCandidate) -> String {
    let transport_str = match candidate.transport {
        Transport::Udp => "UDP",
        Transport::TcpActive => "TCP-ACT",
        Transport::TcpPassive => "TCP-PASS",
    };
    let type_str = match candidate.candidate_type {
        CandidateType::Host => "host",
        CandidateType::ServerReflexive => "srflx",
        CandidateType::Relay => "relay",
    };
    let mut line = format!(
        "candidate:{} {} {} {} {} {} typ {}",
        candidate.foundation,
        candidate.component,
        transport_str,
        candidate.priority,
        candidate.address,
        candidate.port,
        type_str
    );
    if let (Some(ra), Some(rp)) = (&candidate.raddr, candidate.rport) {
        line.push_str(&format!(" raddr {} rport {}", ra, rp));
    }
    line
}

pub fn generate_av_sdp_offer(p: &AvSdpParams) -> AvSdpResult {
    let audio_crypto_key = generate_srtp_key();
    let audio_crypto_line = format!(
        "a=crypto:2 AES_CM_128_HMAC_SHA1_80 inline:{}|2^31",
        audio_crypto_key
    );
    let video_crypto_key = generate_srtp_key();
    let video_crypto_line = format!(
        "a=crypto:2 AES_CM_128_HMAC_SHA1_80 inline:{}|2^31",
        video_crypto_key
    );
    let mut sdp = String::new();

    // Session level
    sdp.push_str("v=0\r\n");
    sdp.push_str(&format!("o=- 0 0 IN IP4 {}\r\n", p.local_ip));
    sdp.push_str("s=session\r\n");
    sdp.push_str(&format!("c=IN IP4 {}\r\n", p.local_ip));
    sdp.push_str("b=CT:99980\r\n");
    sdp.push_str("t=0 0\r\n");
    sdp.push_str("a=x-mediabw:main-video send=12000;recv=12000\r\n");

    // Audio m-line
    sdp.push_str(&format!("m=audio {} RTP/SAVP 0\r\n", p.audio_port));
    sdp.push_str(&format!(
        "a=x-ssrc-range:{}-{}\r\n",
        p.audio_ssrc, p.audio_ssrc
    ));
    sdp.push_str("a=rtcp-fb:* x-message app send:dsh recv:dsh\r\n");
    sdp.push_str("a=rtcp-rsize\r\n");
    sdp.push_str("a=mid:0\r\n");
    sdp.push_str("a=rtpmap:0 PCMU/8000\r\n");
    sdp.push_str("a=ptime:20\r\n");
    sdp.push_str("a=sendrecv\r\n");
    sdp.push_str("a=rtcp-mux\r\n");
    sdp.push_str("a=label:main-audio\r\n");
    sdp.push_str("a=x-source:main-audio\r\n");
    sdp.push_str(&format!("a=ice-ufrag:{}\r\n", p.audio_ufrag));
    sdp.push_str(&format!("a=ice-pwd:{}\r\n", p.audio_pwd));

    if p.audio_candidates.is_empty() {
        sdp.push_str(&format!(
            "a=candidate:1 1 UDP 2130706431 {} {} typ host\r\n",
            p.local_ip, p.audio_port
        ));
    } else {
        for candidate in p.audio_candidates {
            sdp.push_str(&format!("a={}\r\n", to_sdp_line(candidate)));
        }
    }

    sdp.push_str(&audio_crypto_line);
    sdp.push_str("\r\n");

    // Video m-line — X-H264UC (Teams proprietary H.264 SVC variant)
    sdp.push_str(&format!(
        "m=video {} RTP/SAVP 122 121 123\r\n",
        p.video_port
    ));
    sdp.push_str("a=mid:1\r\n");
    sdp.push_str("a=rtpmap:122 X-H264UC/90000\r\n");
    sdp.push_str("a=fmtp:122 packetization-mode=1;mst-mode=NI-TC\r\n");
    sdp.push_str("a=rtpmap:121 x-rtvc1/90000\r\n");
    sdp.push_str("a=rtpmap:123 x-ulpfecuc/90000\r\n");
    sdp.push_str("a=rtcp-fb:* x-message app send:src,x-pli recv:src,x-pli\r\n");
    sdp.push_str("a=rtcp-rsize\r\n");
    sdp.push_str(&format!(
        "a=x-ssrc-range:{}-{}\r\n",
        p.video_ssrc_base,
        p.video_ssrc_base.saturating_add(VIDEO_SSRC_RANGE_SIZE - 1)
    ));
    sdp.push_str("a=x-caps:121 263:320:240:15.0:250000:1;4359:176:144:15.0:100000:1\r\n");
    sdp.push_str("a=sendrecv\r\n");
    sdp.push_str("a=rtcp-mux\r\n");
    sdp.push_str("a=label:main-video\r\n");
    sdp.push_str("a=x-source:main-video\r\n");
    sdp.push_str(&format!("a=ice-ufrag:{}\r\n", p.video_ufrag));
    sdp.push_str(&format!("a=ice-pwd:{}\r\n", p.video_pwd));

    if p.video_candidates.is_empty() {
        sdp.push_str(&format!(
            "a=candidate:1 1 UDP 2130706431 {} {} typ host\r\n",
            p.local_ip, p.video_port
        ));
    } else {
        for candidate in p.video_candidates {
            sdp.push_str(&format!("a={}\r\n", to_sdp_line(candidate)));
        }
    }

    sdp.push_str(&video_crypto_line);
    sdp.push_str("\r\n");

    AvSdpResult {
        sdp,
        audio_crypto_line,
        video_crypto_line,
        audio_ufrag: p.audio_ufrag.to_string(),
        audio_pwd: p.audio_pwd.to_string(),
        video_ufrag: p.video_ufrag.to_string(),
        video_pwd: p.video_pwd.to_string(),
    }
}

pub(crate) fn trouter_callback(trouter_surl: &str, endpoint_id: &str, path: &str) -> String {
    let hash = format!("{:08x}", {
        // Simple hash from endpoint_id + path to produce unique per-path values
        let mut h: u32 = 0x811c9dc5; // FNV-1a init
        for b in endpoint_id.bytes().chain(path.bytes()) {
            h ^= b as u32;
            h = h.wrapping_mul(0x01000193);
        }
        h
    });
    format!(
        "{}callAgent/{}/{}/{}",
        trouter_surl, endpoint_id, hash, path
    )
}

// Scope: https://ic3.teams.office.com/.default
pub async fn create_1to1_call(
    token: &AccessToken,
    epconv_url: &str,
    params: &ConversationCallParams,
    sdp_offer: &str,
) -> Result<(ConversationCreated, ConversationJoined)> {
    let tc = |path: &str| trouter_callback(&params.trouter_surl, &params.endpoint_id, path);
    let cause_id = &params.message_id[..8.min(params.message_id.len())];

    let payload = serde_json::json!({
        "conversationRequest": {
            "conversationType": null,
            "subject": "",
            "suppressDialout": false,  // false = ring the callee
            "roster": {
                "type": "Delta",
                "rosterUpdate": tc("conversation/rosterUpdate/")
            },
            "properties": {
                "allowConversationWithoutHost": true,
                "enableGroupCallEventMessages": true,
                "enableGroupCallUpgradeMessage": false,
                "enableGroupCallMeetupGeneration": false
            },
            "links": {
                "conversationEnd": tc("conversation/conversationEnd/"),
                "conversationUpdate": tc("conversation/conversationUpdate/"),
                "localParticipantUpdate": tc("conversation/localParticipantUpdate/"),
                "addParticipantSuccess": tc("conversation/addParticipantSuccess/"),
                "addParticipantFailure": tc("conversation/addParticipantFailure/"),
                "addModalitySuccess": tc("conversation/addModalitySuccess/"),
                "addModalityFailure": tc("conversation/addModalityFailure/"),
                "confirmUnmute": tc("conversation/confirmUnmute/"),
                "receiveMessage": tc("conversation/receiveMessage/")
            }
        },
        "groupContext": null,
        "groupChat": {
            "threadId": params.thread_id,
            "messageId": null
        },
        "participants": {
            "from": {
                "id": params.caller_mri,
                "displayName": params.caller_display_name,
                "endpointId": params.endpoint_id,
                "participantId": params.participant_id,
                "languageId": "en-US"
            },
            "to": []
        },
        "capabilities": null,
        "endpointCapabilities": 73463,
        "clientEndpointCapabilities": 9336554,
        "endpointMetadata": { "holographicCapabilities": 3 },
        "meetingInfo": null,
        "endpointState": {
            "endpointStateSequenceNumber": 0,
            "endpointProperties": {
                "additionalEndpointProperties": {
                    "infoShownInReportMode": "FullInformation"
                }
            }
        },
        "callInvitation": {
            "callModalities": ["Audio", "Video", "ScreenViewer"],
            "replaces": null,
            "transferor": null,
            "links": {
                "progress": tc("call/progress/"),
                "mediaAnswer": tc("call/mediaAnswer/"),
                "acceptance": tc("call/acceptance/"),
                "redirection": tc("call/redirection/"),
                "end": tc("call/end/")
            },
            "clientContentForMediaController": {
                "controlVideoStreaming": tc("call/controlVideoStreaming/"),
                "csrcInfo": tc("call/csrcInfo/"),
                "dominantSpeakerInfo": tc("call/dominantSpeakerInfo/")
            },
            "pstnContent": {
                "emergencyCallCountry": "",
                "platformName": "teams-cli",
                "publicApiCall": false
            },
            "mediaContent": {
                "contentType": "application/sdp",
                "blob": sdp_offer
            }
        },
        "debugContent": {
            "ecsEtag": "\"0\"",
            "causeId": cause_id
        }
    });

    let access_token = format!("Bearer {}", token.value);

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let resp = client
        .post(epconv_url)
        .header("Authorization", access_token)
        .header("Content-Type", "application/json")
        .header("x-microsoft-skype-chain-id", &params.chain_id)
        .header("x-microsoft-skype-message-id", &params.message_id)
        .header("x-microsoft-skype-client", SKYPE_CLIENT_HEADER)
        .header("Referer", "https://teams.microsoft.com/")
        .header("ms-teams-partition", TEAMS_PARTITION)
        .header("ms-teams-region", TEAMS_REGION)
        .header("ms-teams-ring", TEAMS_RING)
        .header("x-ms-migration", "True")
        .json(&payload)
        .send()
        .await
        .context("1:1 call POST to epconv failed")?;

    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        anyhow::bail!("1:1 call epconv failed ({}): {}", status, body);
    }

    // Parse conversationController from response
    let resp_json: serde_json::Value =
        serde_json::from_str(&body).context("1:1 call response is not valid JSON")?;

    let conv_controller = resp_json
        .pointer("/conversationController")
        .or_else(|| resp_json.pointer("/conversationResponse/conversationController"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            headers
                .get("location")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .context("No conversationController in 1:1 call response")?;

    let add_participant_url = resp_json
        .pointer("/links/addParticipant")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let cc_active_url = resp_json
        .pointer("/links/active")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let created = ConversationCreated {
        conversation_controller: conv_controller,
        add_participant_url,
    };
    let joined = ConversationJoined { cc_active_url };

    Ok((created, joined))
}

pub fn generate_transaction_id() -> [u8; 12] {
    let id1 = Uuid::new_v4();
    let id2 = Uuid::new_v4();
    let b1 = id1.as_bytes();
    let b2 = id2.as_bytes();
    let mut txn = [0u8; 12];
    txn[..8].copy_from_slice(&b1[..8]);
    txn[8..12].copy_from_slice(&b2[..4]);
    txn
}

pub fn build_stun_binding_request(transaction_id: &[u8; 12]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(STUN_HEADER_SIZE);
    buf.extend_from_slice(&BINDING_REQUEST.to_be_bytes());
    buf.extend_from_slice(&0u16.to_be_bytes()); // length = 0
    buf.extend_from_slice(&MAGIC_COOKIE.to_be_bytes());
    buf.extend_from_slice(transaction_id);
    buf
}

pub fn is_stun_response(data: &[u8]) -> bool {
    if data.len() < STUN_HEADER_SIZE {
        return false;
    }
    let msg_type = u16::from_be_bytes([data[0], data[1]]);
    let magic = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    msg_type == BINDING_RESPONSE && magic == MAGIC_COOKIE
}

pub fn get_transaction_id(data: &[u8]) -> Option<[u8; 12]> {
    if data.len() < STUN_HEADER_SIZE {
        return None;
    }
    let mut txn = [0u8; 12];
    txn.copy_from_slice(&data[8..20]);
    Some(txn)
}

fn decode_xor_mapped_address(value: &[u8], transaction_id: &[u8]) -> Option<SocketAddr> {
    if value.len() < 4 {
        return None;
    }
    let family = value[1];
    let xport = u16::from_be_bytes([value[2], value[3]]);
    let port = xport ^ (MAGIC_COOKIE >> 16) as u16;

    match family {
        0x01 if value.len() >= 8 => {
            // IPv4
            let cookie = MAGIC_COOKIE.to_be_bytes();
            let ip = Ipv4Addr::new(
                value[4] ^ cookie[0],
                value[5] ^ cookie[1],
                value[6] ^ cookie[2],
                value[7] ^ cookie[3],
            );
            Some(SocketAddr::new(IpAddr::V4(ip), port))
        }
        0x02 if value.len() >= 20 => {
            // IPv6
            let mut xor_key = [0u8; 16];
            xor_key[..4].copy_from_slice(&MAGIC_COOKIE.to_be_bytes());
            if transaction_id.len() >= 12 {
                xor_key[4..16].copy_from_slice(&transaction_id[..12]);
            }
            let mut octets = [0u8; 16];
            for i in 0..16 {
                octets[i] = value[4 + i] ^ xor_key[i];
            }
            let ip = std::net::Ipv6Addr::from(octets);
            Some(SocketAddr::new(IpAddr::V6(ip), port))
        }
        _ => None,
    }
}

fn decode_mapped_address(value: &[u8]) -> Option<SocketAddr> {
    if value.len() < 4 {
        return None;
    }
    let family = value[1];
    let port = u16::from_be_bytes([value[2], value[3]]);

    match family {
        0x01 if value.len() >= 8 => {
            let ip = Ipv4Addr::new(value[4], value[5], value[6], value[7]);
            Some(SocketAddr::new(IpAddr::V4(ip), port))
        }
        _ => None,
    }
}

pub fn parse_binding_response(data: &[u8]) -> Option<SocketAddr> {
    if data.len() < STUN_HEADER_SIZE {
        return None;
    }
    let msg_type = u16::from_be_bytes([data[0], data[1]]);
    if msg_type != BINDING_RESPONSE {
        return None;
    }
    let magic = u32::from_be_bytes([data[4], data[5], data[6], data[7]]);
    if magic != MAGIC_COOKIE {
        return None;
    }

    let txn_id = &data[8..20];
    let msg_len = u16::from_be_bytes([data[2], data[3]]) as usize;
    let attrs_end = std::cmp::min(STUN_HEADER_SIZE + msg_len, data.len());

    let mut pos = STUN_HEADER_SIZE;
    while pos + 4 <= attrs_end {
        let attr_type = u16::from_be_bytes([data[pos], data[pos + 1]]);
        let attr_len = u16::from_be_bytes([data[pos + 2], data[pos + 3]]) as usize;
        let attr_start = pos + 4;
        let attr_end = attr_start + attr_len;

        if attr_end > attrs_end {
            break;
        }

        if attr_type == ATTR_XOR_MAPPED_ADDRESS {
            return decode_xor_mapped_address(&data[attr_start..attr_end], txn_id);
        }

        // Also handle MAPPED-ADDRESS as fallback
        if attr_type == ATTR_MAPPED_ADDRESS {
            return decode_mapped_address(&data[attr_start..attr_end]);
        }

        // Advance to next attribute (padded to 4-byte boundary)
        pos = attr_start + ((attr_len + 3) & !3);
    }

    None
}

fn compute_priority(ctype: CandidateType, local_preference: u16, component: u8) -> u32 {
    let type_preference: u32 = match ctype {
        CandidateType::Host => 126,
        CandidateType::ServerReflexive => 100,
        CandidateType::Relay => 0,
    };
    (type_preference << 24) | ((local_preference as u32) << 8) | (256 - component as u32)
}

pub async fn gather_srflx_candidate(socket: &UdpSocket, stun_server: &str) -> Option<IceCandidate> {
    // Resolve STUN server address
    let server_addr: SocketAddr = match tokio::net::lookup_host(stun_server).await {
        Ok(mut addrs) => addrs.next()?,
        Err(e) => {
            eprintln!("Failed to resolve STUN server {}: {}", stun_server, e);
            return None;
        }
    };

    let txn_id = generate_transaction_id();
    let request = build_stun_binding_request(&txn_id);

    // Send request and wait for response
    for attempt in 0..2 {
        if let Err(e) = socket.send_to(&request, server_addr).await {
            eprintln!("STUN send to {} failed: {}", server_addr, e);
            return None;
        }

        let mut buf = [0u8; 256];
        match tokio::time::timeout(Duration::from_secs(2), socket.recv_from(&mut buf)).await {
            Ok(Ok((len, _from))) => {
                let data = &buf[..len];
                if is_stun_response(data) {
                    // Verify transaction ID matches
                    if let Some(resp_txn) = get_transaction_id(data) {
                        if resp_txn == txn_id {
                            if let Some(mapped_addr) = parse_binding_response(data) {
                                let local_addr = socket.local_addr().ok()?;
                                return Some(IceCandidate {
                                    foundation: "2".into(),
                                    component: 1,
                                    transport: Transport::Udp,
                                    priority: compute_priority(
                                        CandidateType::ServerReflexive,
                                        1,
                                        1,
                                    ),
                                    address: mapped_addr.ip().to_string(),
                                    port: mapped_addr.port(),
                                    candidate_type: CandidateType::ServerReflexive,
                                    raddr: Some(local_addr.ip().to_string()),
                                    rport: Some(local_addr.port()),
                                });
                            }
                        }
                    }
                }
            }
            Ok(Err(e)) => {
                eprintln!("STUN recv error (attempt {}): {}", attempt, e);
            }
            Err(_) => {
                eprintln!("STUN timeout (attempt {})", attempt);
            }
        }
    }

    None
}

pub async fn invite_user(
    token: &AccessToken,
    conversation_controller: &str,
    params: &ConversationCallParams,
    callee_mri: &str,
    include_video: bool,
) -> Result<()> {
    let tc = |path: &str| trouter_callback(&params.trouter_surl, &params.endpoint_id, path);

    // Derive /add URL from conversation controller
    let add_url = if let Some(idx) = conversation_controller.find('?') {
        let (path, query) = conversation_controller.split_at(idx);
        format!("{}/add{}", path.trim_end_matches('/'), query)
    } else {
        format!("{}/add", conversation_controller.trim_end_matches('/'))
    };

    let callee_participant_id = uuid::Uuid::new_v4().to_string();
    let call_modalities: Vec<&str> = if include_video {
        vec!["Audio", "Video"]
    } else {
        vec!["Audio"]
    };

    let payload = serde_json::json!({
        "disableUnmute": false,
        "participants": {
            "from": {
                "id": params.caller_mri,
                "displayName": params.caller_display_name,
                "endpointId": params.endpoint_id,
                "participantId": params.participant_id,
                "languageId": "en-US"
            },
            "to": [{
                "id": callee_mri,
                "participantId": callee_participant_id
            }]
        },
        // Include call invitation data to trigger ringing on callee's device
        "participantInvitationData": {
            "callModalities": call_modalities,
            "callDirection": "Outgoing"
        },
        "callInvitation": {
            "callModalities": call_modalities,
            "replaces": null,
            "transferor": null
        },
        "replacementDetails": null,
        "groupContext": null,
        "groupChat": {
            "threadId": params.thread_id,
            "messageId": null
        },
        "links": {
            "addParticipantSuccess": tc("conversation/addParticipantSuccess/"),
            "addParticipantFailure": tc("conversation/addParticipantFailure/")
        }
    });

    // Fresh message-id for deduplication
    let invite_msg_id = uuid::Uuid::new_v4().to_string();

    let access_token = format!("Bearer {}", token.value);

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let resp = client
        .post(&add_url)
        .header("Authorization", access_token)
        .header("Content-Type", "application/json")
        .header("x-microsoft-skype-chain-id", &params.chain_id)
        .header("x-microsoft-skype-message-id", &invite_msg_id)
        .header("x-microsoft-skype-client", SKYPE_CLIENT_HEADER)
        .header("Referer", "https://teams.microsoft.com/")
        .header("ms-teams-partition", TEAMS_PARTITION)
        .header("ms-teams-region", TEAMS_REGION)
        .header("ms-teams-ring", TEAMS_RING)
        .header("x-ms-migration", "True")
        .json(&payload)
        .send()
        .await
        .context("Failed to POST user invite")?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if !status.is_success() {
        anyhow::bail!("User invite failed ({}): {}", status, body);
    }

    Ok(())
}

pub fn extract_callee_oid_from_thread(thread_id: &str, caller_oid: &str) -> Option<String> {
    let inner = thread_id
        .strip_prefix("19:")
        .and_then(|s| s.strip_suffix("@unq.gbl.spaces"))?;

    let parts: Vec<&str> = inner.split('_').collect();
    if parts.len() != 2 {
        return None;
    }

    if parts[0] == caller_oid {
        Some(parts[1].to_string())
    } else if parts[1] == caller_oid {
        Some(parts[0].to_string())
    } else {
        println!(
            "Thread ID {} doesn't contain caller OID {}",
            thread_id, caller_oid
        );
        Some(parts[1].to_string())
    }
}

fn cc_call_links(tc: &dyn Fn(&str) -> String) -> serde_json::Value {
    serde_json::json!({
        "links": {
            "mediaAcknowledgement": tc("call/mediaAcknowledgement/"),
            "rejection": tc("call/rejection/"),
            "acknowledgement": tc("call/acknowledgement/"),
            "mediaRenegotiation": tc("call/mediaRenegotiation/"),
            "replacement": tc("call/replacement/"),
            "progress": tc("call/progress/"),
            "mediaAnswer": tc("call/mediaAnswer/"),
            "newMediaOffer": tc("call/newMediaOffer/"),
            "redirection": tc("call/redirection/"),
            "balanceUpdate": tc("call/balanceUpdate/"),
            "acceptance": tc("call/acceptance/"),
            "controlVideoStreaming": tc("call/controlVideoStreaming/"),
            "dominantSpeakerInfo": tc("call/dominantSpeakerInfo/"),
            "csrcInfo": tc("call/csrcInfo/"),
            "end": tc("call/end/"),
            "retargetCompletion": tc("call/retargetCompletion/"),
            "transfer": tc("call/transfer/"),
            "transferAcceptance": tc("call/transferAcceptance/"),
            "transferCompletion": tc("call/transferCompletion/"),
            "holdCompletion": tc("call/holdCompletion/"),
            "resumeCompletion": tc("call/resumeCompletion/"),
            "call": tc("call/updateMediaDescriptions"),
            "monitorCompletion": tc("call/monitorCompletion/")
        },
        "clientContentForMediaController": {
            "controlVideoStreaming": tc("call/controlVideoStreaming/"),
            "csrcInfo": tc("call/csrcInfo/")
        }
    })
}

pub async fn acknowledge_call_acceptance(
    token: &AccessToken,
    acknowledgement_url: &str,
    params: &ConversationCallParams,
) -> Result<()> {
    let tc = |path: &str| trouter_callback(&params.trouter_surl, &params.endpoint_id, path);

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let access_token = format!("Bearer {}", token.value);

    let resp = client
        .post(acknowledgement_url)
        .header("Authorization", access_token)
        .header("Content-Type", "application/json")
        .header("x-microsoft-skype-chain-id", &params.chain_id)
        .header("x-microsoft-skype-message-id", &params.message_id)
        .header("x-microsoft-skype-client", SKYPE_CLIENT_HEADER)
        .header("Referer", "https://teams.microsoft.com/")
        .header("ms-teams-partition", TEAMS_PARTITION)
        .header("ms-teams-region", TEAMS_REGION)
        .header("ms-teams-ring", TEAMS_RING)
        .json(&serde_json::json!({
            "callAcceptanceAcknowledgement": cc_call_links(&tc)
        }))
        .send()
        .await
        .context("Failed to POST call acceptance acknowledgement")?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status.is_success() {
        Ok(())
    } else {
        anyhow::bail!(
            "Call acceptance acknowledgement failed ({}): {}",
            status,
            body
        );
    }
}

pub async fn register_cc_callbacks(
    token: &AccessToken,
    call_leg_url: &str,
    params: &ConversationCallParams,
) -> Result<()> {
    let tc = |path: &str| trouter_callback(&params.trouter_surl, &params.endpoint_id, path);

    let payload = serde_json::json!({
        "callAcceptanceAcknowledgement": cc_call_links(&tc),
        "callParticipantUpdate": cc_call_links(&tc)
    });

    let client = Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .build()?;

    let access_token = format!("Bearer {}", token.value);

    let resp = client
        .post(call_leg_url)
        .header("Authorization", access_token)
        .header("Content-Type", "application/json")
        .header("x-microsoft-skype-chain-id", &params.chain_id)
        .header("x-microsoft-skype-message-id", &params.message_id)
        .header("x-microsoft-skype-client", SKYPE_CLIENT_HEADER)
        .header("Referer", "https://teams.microsoft.com/")
        .header("ms-teams-partition", TEAMS_PARTITION)
        .header("ms-teams-region", TEAMS_REGION)
        .header("ms-teams-ring", TEAMS_RING)
        .json(&payload)
        .send()
        .await
        .context("Failed to POST CC callback registration")?;

    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();

    if status.is_success() {
        Ok(())
    } else {
        anyhow::bail!("CC callback registration failed ({}): {}", status, body);
    }
}

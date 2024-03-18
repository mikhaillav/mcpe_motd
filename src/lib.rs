//! # MCPE MOTD
//!  A library to fetch some information from MCPE (MCBE actually) over raknet.

use std::net::UdpSocket;

/// Enumerates the possible errors you can get.
#[derive(Debug)]
pub enum MotdErrorCode {
    /// UdpSocket couldn't bind on 0.0.0.0:0 (random port that system will give us).
    CantBind = 1,
    /// UdpSocket couldn't send raknet packet to the target server.
    CantSendTo = 2,
    /// Minecraft require at least 4 fields: edition, motd, protocol_version, version_name.
    ServerIdStringTooSmall = 3,
    /// Minecraft won't work with that field if it isn't a valid number.
    CantParseProtocolVersion = 4,
    /// Minecraft won't work with that field if it isn't a valid number.
    CantParsePlayerCount = 5,
    /// Minecraft won't work with that field if it isn't a valid number.
    CantParsePlayerMaxCount = 6,
    /// Minecraft won't work with that field if it isn't a valid number.
    CantParseGameModeNum = 7,
    /// Minecraft won't work with that field if it isn't a valid number.
    CantParsePort4 = 8,
    /// Minecraft won't work with that field if it isn't a valid number.
    CantParsePort6 = 9,
}

/// Custom error type.
#[derive(Debug)]
pub struct MotdError {
    /// Error code.
    pub code: MotdErrorCode,
    /// More detailed info about an error.
    pub message: String,
}

/// Parsed [server id string](https://wiki.vg/Raknet_Protocol#Unconnected_Pong).
/// **Be careful, if server id string is invalid (e.g. has fewer fields), lib will (at least try to) add default ones.**
/// However, there is *UnconnectedPong* struct with *server_id_string_parsed_ok* field.
#[derive(Debug)]
pub struct ServerIdStringParsed {
    /// Server minecraft edition (MCPE or MCEE).
    pub edition: String,
    /// Text that is displayed in the server tab.
    pub motd: String,
    /// Minecraft protocol version (e.g. 615).
    pub protocol_version: i16,
    /// Minecraft version name (e.g. 1.20.30).
    pub version_name: String,
    /// How many players is playing on the server.
    pub player_count: i32,
    /// How many players can be playing on the server at the same time.
    pub max_player_count: i32,
    /// Some unique id.
    pub server_unique_id: String,
    /// Map name (display in esc menu at the right top).
    pub level_name: String,
    /// Default gamemode.
    pub gamemode: String,
    /// Default gamemode but number.
    pub gamemode_numeric: u8,
    /// Port used for IPv4 communication.
    pub port_v4: u16,
    /// Port used for IPv6 communication.
    pub port_v6: u16,
}

/// Parsed [RakNet unconnected pong packet](https://wiki.vg/Raknet_Protocol#Unconnected_Pong).
/// Has more information than *ServerIdStringParsed*.
/// Unlike *ServerIdStringParsed*, using *UnconnectedPong* you can check if server id string was parsed correctly (without adding default ones).
#[derive(Debug)]
pub struct UnconnectedPong {
    /// Packet id (0x1c).
    pub id: u8,
    /// Time since server start in ms.
    pub time_since_start: i64,
    /// Server guid.
    pub server_guid: i64,
    /// Whoops magic...
    pub magic: [u8; 16],
    /// Length of server id string.
    pub server_id_string_len: i16,
    /// Raw server id string.
    pub server_id_string_raw: String,
    /// Whether server id string was parsed correctly.
    pub server_id_string_parsed_ok: bool,
    /// Parsed server id string.
    pub server_id_string_parsed: ServerIdStringParsed,
}

/// Returns parsed [RakNet unconnected pong packet](https://wiki.vg/Raknet_Protocol#Unconnected_Pong) or error explaining why packet wasn't parsed.
///
/// # Arguments
///
/// * `addr` - address of the target server.
///
/// # Panics
///
/// Function can return an error if:
///  - couldn't send packet to the target server
///  - couldn't parse response (e.g. invalid unconnected pong packet)
///
/// However, it will try to replace invalid data with default ones (e.g. empty port field in server id string will be replaced with 19132) until minecraft can process that data.
/// Obviously, minecraft won't work with empty server id string or *String* instead of *version_protocol field*.
///
/// # Example
///
/// ```
/// use::mcpe_motd::fetch_unconected_pong;
///
/// let pong = match fetch_unconected_pong("127.0.0.1:19132") {
///     Ok(pong) => pong,
///     Err(e) => panic!(e)
/// };
///
/// println!("Server id string was correctly parsed (true / false): {}.", pong.server_id_string_parsed_ok);
/// println!("Raw server id string: {}.", pong.server_id_string_raw);
/// println!("Server guid: {}.", pong.server_guid);
/// ```
pub fn fetch_unconected_pong(addr: &str) -> Result<UnconnectedPong, MotdError> {
    let socket = match UdpSocket::bind("0.0.0.0:0") {
        Ok(sock) => sock,
        Err(_) => { return Err(MotdError { code: MotdErrorCode::CantSendTo, message: String::from("Couldn't bind to 0.0.0.0:0") }); }
    };

    let buf: &[u8] = &[/*ID*/ 0x01, /*Time*/ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, /*MAGIC*/ 0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78, /*Client GUID*/ 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    match socket.send_to(buf, addr) {
        Ok(_) => (),
        Err(_) => { return Err(MotdError { code: MotdErrorCode::CantSendTo, message: String::from("Couldn't send to ... (here should be ip)") }); }
    }

    let mut response: [u8; 1024] = [0; 1024];
    let (size, _src) = socket.recv_from(&mut response).expect("ddd");
    let response = &mut response[..size];

    // Packet id (0x1c) - 1 byte
    let id = response[0];

    // Time since start in ms - 8 bytes
    let time_since_start: i64 = (response[8] as i64) |
        (response[7] as i64) << 8 |
        (response[6] as i64) << 16 |
        (response[5] as i64) << 24 |
        (response[4] as i64) << 32 |
        (response[3] as i64) << 40 |
        (response[2] as i64) << 48 |
        (response[1] as i64) << 56;

    // Server GUID - 8 bytes
    let server_guid: i64 = (response[16] as i64) |
        (response[15] as i64) << 8 |
        (response[14] as i64) << 16 |
        (response[13] as i64) << 24 |
        (response[12] as i64) << 32 |
        (response[11] as i64) << 40 |
        (response[10] as i64) << 48 |
        (response[9] as i64) << 56;

    // Magic - 16 bytes
    const MAGIC: [u8; 16] = [0x00, 0xff, 0xff, 0x00, 0xfe, 0xfe, 0xfe, 0xfe, 0xfd, 0xfd, 0xfd, 0xfd, 0x12, 0x34, 0x56, 0x78];

    // Server id string length - 2 bytes
    let server_id_string_len = (response[34] as i16) |
        (response[33] as i16) << 8;

    // Server id string - <server_id_string_len> bytes
    let server_id_string = String::from_utf8_lossy(&response[35..35 + server_id_string_len as usize]).to_string();

    let split_server_id_string: &Vec<String> = &server_id_string.split(";")
        .filter(|s| *s != "")
        .map(|s| s.to_string())
        .collect();

    let split_server_id_string_size = split_server_id_string.len();

    if split_server_id_string_size < 4 {
        return Err(MotdError { code: MotdErrorCode::ServerIdStringTooSmall, message: String::from("Server id string has less than 4 required fields") });
    }

    let mut server_id_string_parsed_ok = true;

    let server_id_string_parsed = ServerIdStringParsed {
        edition: split_server_id_string[0].to_string(),

        motd: split_server_id_string[1].to_string(),

        protocol_version: match split_server_id_string[2].parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(MotdError { code: MotdErrorCode::CantParseProtocolVersion, message: String::from("Couldn't parse protocol_version field from server id string") });
            }
        },

        version_name: split_server_id_string[3].to_string(),

        player_count: if split_server_id_string_size >= 5 {
            match split_server_id_string[4].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(MotdError { code: MotdErrorCode::CantParsePlayerCount, message: String::from("Couldn't parse player_count field from server id string") });
                }
            }
        } else {
            server_id_string_parsed_ok = false;
            -1
        },

        max_player_count: if split_server_id_string_size >= 6 {
            match split_server_id_string[5].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(MotdError { code: MotdErrorCode::CantParsePlayerMaxCount, message: String::from("Couldn't parse max_player_count field from server id string") });
                }
            }
        } else {
            server_id_string_parsed_ok = false;
            -1
        },

        server_unique_id: if split_server_id_string_size >= 7 { split_server_id_string[6].to_string() } else { "".to_string() },

        level_name: if split_server_id_string_size >= 8 { split_server_id_string[7].to_string() } else { "".to_string() },

        gamemode: if split_server_id_string_size >= 9 { split_server_id_string[8].to_string() } else { "Survival".to_string() },

        gamemode_numeric: if split_server_id_string_size >= 10 {
            match split_server_id_string[9].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(MotdError { code: MotdErrorCode::CantParseGameModeNum, message: String::from("Couldn't parse gamemode_numeric field from server id string") });
                }
            }
        } else {
            server_id_string_parsed_ok = false;
            0
        },

        port_v4: if split_server_id_string_size >= 11 {
            match split_server_id_string[10].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(MotdError { code: MotdErrorCode::CantParsePort4, message: String::from("Couldn't parse port_v4 field from server id string") });
                }
            }
        } else {
            server_id_string_parsed_ok = false;
            19132
        },

        port_v6: if split_server_id_string_size >= 12 {
            match split_server_id_string[11].parse() {
                Ok(v) => v,
                Err(_) => {
                    return Err(MotdError { code: MotdErrorCode::CantParsePort6, message: String::from("Couldn't parse port_v6 field from server id string") });
                }
            }
        } else {
            server_id_string_parsed_ok = false;
            19132
        },
    };

    Ok(UnconnectedPong {
        id,
        time_since_start,
        server_guid,
        magic: MAGIC,
        server_id_string_len,
        server_id_string_raw: server_id_string,
        server_id_string_parsed_ok,
        server_id_string_parsed,
    })
}

/// Returns parsed [server id string](https://wiki.vg/Raknet_Protocol#Unconnected_Pong) or error explaining why it wasn't parsed.
/// # Arguments
///
/// * `addr` - address of the target server.
///
/// # Panics
///
/// Function can return an error if:
///  - couldn't send packet to the target server
///  - couldn't parse response (e.g. invalid unconnected pong packet)
///
/// However, it will try to replace invalid data with default ones (e.g. empty port field in server id string will be replaced with 19132) until minecraft can process that data.
/// Obviously, minecraft won't work with empty server id string or *String* instead of *version_protocol field*.
///
/// # Example
///
/// ```
/// use::mcpe_motd::fetch_server_id_string;
///
/// let server_id_string = match fetch_server_id_string("127.0.0.1:19132") {
///     Ok(str) => str,
///     Err(e) => panic!(e)
/// };
///
/// // Will print -1 / -1 if server id string is invalid (as well as vanilla minecraft will).
/// println!("There are {} / {} players on the server.",
/// server_id_string.player_count,
/// server_id_string.max_player_count);
/// ```
pub fn fetch_server_id_string(addr: &str) -> Result<ServerIdStringParsed, MotdError> {
    let unconected_pong = match fetch_unconected_pong(addr) {
        Ok(v) => v,
        Err(e) => { return Err(e); }
    };

    Ok(unconected_pong.server_id_string_parsed)
}
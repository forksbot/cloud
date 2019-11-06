use quick_error::quick_error;

quick_error! {
    #[derive(Debug)]
    pub enum Error {
        SetLoggerError(err: log::SetLoggerError) {
            from()
            description("logger error")
            display("Logger error: {}", err)
            cause(err)
        }
        Io(err: std::io::Error) {
            from()
            description("io error")
            display("I/O error: {}", err)
            cause(err)
        }
        JWT(err: biscuit::errors::Error) {
            from()
            description("jwt error")
            display("Authentication error: {}", err)
            cause(err)
        }
        JWTValidate(err: biscuit::errors::ValidationError) {
            from()
            description("jwt error")
            display("Authentication error: {}", err)
            cause(err)
        }
        Reqwest(err: reqwest::Error) {
            from()
            description("fetch error")
            display("Fetch error: {}", err)
            cause(err)
        }
        Ring(err: ring::error::Unspecified) {
            from()
            description("Encryption/Decryption")
            display("Ring: {}", err)
        }
        ImmediateDisconnect {
            description("peer disconnected immediately")
        }
        NoClient {
            description("No client with this ID")
        }
        ClientIdExists {
            description("Client with that ID already exists")
        }
        InvalidMqttPacket {
            description("Invalid Mqtt Packet")
        }
        InvalidClientId {
            description("Invalid Client ID")
        }
        DisconnectRequest {
            description("Received Disconnect Request")
        }
        NotInQueue {
            description("Couldn't find requested message in the queue")
        }
        DisconnectPacket {
            description("Received disconnect packet from client")
        }
        Other
    }
}

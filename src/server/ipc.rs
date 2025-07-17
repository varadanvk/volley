use zmq::{Context, Socket};

pub struct IPCChannel {
    pub context: Context,
    pub socket: Socket,
}

impl IPCChannel {
    // Create server (REP socket that binds)
    pub fn new_server(endpoint: &str) -> Result<Self, zmq::Error> {
        let context = Context::new();
        let socket = context.socket(zmq::REP)?;
        socket.bind(endpoint)?;
        Ok(Self { context, socket })
    }

    // Create client (REQ socket that connects)
    pub fn new_client(endpoint: &str) -> Result<Self, zmq::Error> {
        let context = Context::new();
        let socket = context.socket(zmq::REQ)?;
        socket.connect(endpoint)?;
        Ok(Self { context, socket })
    }

    // New: Create PULL socket (for receiving actions on server)
    pub fn new_pull(endpoint: &str) -> Result<Self, zmq::Error> {
        let context = Context::new();
        let socket = context.socket(zmq::PULL)?;
        socket.bind(endpoint)?;
        Ok(Self { context, socket })
    }

    // New: Create PUB socket (for broadcasting state on server)
    pub fn new_pub(endpoint: &str) -> Result<Self, zmq::Error> {
        let context = Context::new();
        let socket = context.socket(zmq::PUB)?;
        socket.bind(endpoint)?;
        Ok(Self { context, socket })
    }

    // New: Create PUSH socket (for sending actions from client)
    pub fn new_push(endpoint: &str) -> Result<Self, zmq::Error> {
        let context = Context::new();
        let socket = context.socket(zmq::PUSH)?;
        socket.connect(endpoint)?;
        Ok(Self { context, socket })
    }

    // New: Create SUB socket (for receiving state on client)
    pub fn new_sub(endpoint: &str) -> Result<Self, zmq::Error> {
        let context = Context::new();
        let socket = context.socket(zmq::SUB)?;
        socket.connect(endpoint)?;
        socket.set_subscribe(b"")?; // Subscribe to all messages
        Ok(Self { context, socket })
    }

    // Raw send bytes
    pub fn send_bytes(&self, data: &[u8]) -> Result<(), zmq::Error> {
        self.socket.send(data, 0)
    }

    // Raw receive bytes
    pub fn recv_bytes(&self) -> Result<Vec<u8>, zmq::Error> {
        self.socket.recv_bytes(0)
    }

    // Non-blocking receive (for PULL)
    pub fn recv_bytes_nonblocking(&self) -> Result<Vec<u8>, zmq::Error> {
        self.socket.recv_bytes(zmq::DONTWAIT)
    }

    // Blocking receive with timeout
    pub fn recv_bytes_timeout(&self, timeout_ms: i32) -> Result<Vec<u8>, zmq::Error> {
        self.socket.set_rcvtimeo(timeout_ms)?;
        self.socket.recv_bytes(0)
    }
}

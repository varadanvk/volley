use crate::server::models::{Action, Command};
use zmq::{Context, Socket};

pub struct IPCChannel {
    pub context: Context,
    pub socket: Socket,
}

impl IPCChannel {
    pub fn new(endpoint: &str) -> Self {
        let context = Context::new();
        let socket = context.socket(zmq::REQ).unwrap();
        socket.connect(endpoint).unwrap();
        Self { context, socket }
    }

    pub fn send_command(&self, command: Command) -> Result<(), zmq::Error> {
        let command_bytes = command.to_bytes();
        self.socket.send(command_bytes, 0).unwrap();
        Ok(())
    }

    pub fn send_action(&self, action: Action) -> Result<(), zmq::Error> {
        let action_bytes = action.to_bytes();
        self.socket.send(action_bytes, 0).unwrap();
        Ok(())
    }
}

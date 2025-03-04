use std::net::TcpStream;
use std::sync::{Arc, Mutex};

/// Type alias for the shared clients collection
pub type Clients = Arc<Mutex<Vec<TcpStream>>>;

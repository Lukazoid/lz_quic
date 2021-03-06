use protocol::ConnectionId;
use std::collections::{HashMap, HashSet};
use std::net::SocketAddr;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
struct AddressTuple {
    pub local_address: SocketAddr,
    pub remote_address: SocketAddr,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AddressConnectionIds {
    Single(ConnectionId),
    Multiple(HashSet<ConnectionId>),
}

/// The type responsible for mapping to/from remote addresses and connection identifiers.
#[derive(Debug, Clone, Default)]
pub struct ConnectionMap {
    /// Each connection ID MUST be used on only one local address
    connection_addresses: HashMap<ConnectionId, AddressTuple>,

    /// Each address tuple can have multiple connections
    address_connections: HashMap<AddressTuple, HashSet<ConnectionId>>,
}

impl ConnectionMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_capacity(count: usize) -> Self {
        Self {
            connection_addresses: HashMap::with_capacity(count),
            address_connections: HashMap::with_capacity(count),
        }
    }

    pub fn insert(
        &mut self,
        connection_id: ConnectionId,
        local_address: SocketAddr,
        remote_address: SocketAddr,
    ) -> bool {
        let address_tuple = AddressTuple {
            local_address,
            remote_address,
        };

        if self.connection_addresses
            .entry(connection_id)
            .or_insert(address_tuple) != &address_tuple
        {
            return false;
        }

        let connection_ids = self.address_connections
            .entry(address_tuple)
            .or_insert_with(Default::default);

        connection_ids.insert(connection_id)
    }

    pub fn remove_connection(&mut self, connection_id: ConnectionId) {
        if let Some(address_tuple) = self.connection_addresses.remove(&connection_id) {
            if let Some(connection_ids) = self.address_connections.get_mut(&address_tuple) {
                assert!(
                    connection_ids.remove(&connection_id),
                    "there should have been a connection id for the removed address tuple"
                );
            }
        }
    }

    pub fn remove_address(&mut self, local_address: SocketAddr, remote_address: SocketAddr) {
        let address_tuple = AddressTuple {
            local_address,
            remote_address,
        };

        if let Some(connection_ids) = self.address_connections.remove(&address_tuple) {
            for connection_id in connection_ids {
                assert_eq!(
                    self.connection_addresses.remove(&connection_id),
                    Some(address_tuple),
                    "the removed connection should have been pointing at the correct address tuple"
                );
            }
        }
    }

    pub fn contains_connection_id(&self, connection_id: ConnectionId) -> bool {
        self.connection_addresses.contains_key(&connection_id)
    }

    pub fn get_connection_id(
        &self,
        local_address: SocketAddr,
        remote_address: SocketAddr,
    ) -> Option<AddressConnectionIds> {
        let address_tuple = AddressTuple {
            local_address,
            remote_address,
        };

        if let Some(address_connections) = self.address_connections.get(&address_tuple) {
            match address_connections.len() {
                0 => None,
                1 => Some(AddressConnectionIds::Single(*address_connections
                    .iter()
                    .next()
                    .unwrap())),
                _ => Some(AddressConnectionIds::Multiple(address_connections.clone())),
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AddressConnectionIds, ConnectionMap};
    use protocol::ConnectionId;

    #[test]
    fn get_connection_id_returns_none_when_no_connection_added() {
        let connection_map = ConnectionMap::new();

        assert_matches!(
            connection_map.get_connection_id(
                "10.0.0.1:65413".parse().unwrap(),
                "10.0.0.2:443".parse().unwrap()
            ),
            None
        );
    }

    #[test]
    fn get_connection_id_returns_correct_id() {
        let mut connection_map = ConnectionMap::new();

        let connection_id = ConnectionId::generate().unwrap();

        let local_address = "10.0.0.1:65412".parse().unwrap();
        let remote_address = "10.0.0.2:443".parse().unwrap();
        assert!(connection_map.insert(connection_id, local_address, remote_address));

        assert_eq!(
            connection_map.get_connection_id(local_address, remote_address),
            Some(AddressConnectionIds::Single(connection_id))
        );
    }

    #[test]
    fn insert_fails_if_connection_id_already_exists() {
        let mut connection_map = ConnectionMap::new();

        let connection_id = ConnectionId::generate().unwrap();
        assert!(connection_map.insert(
            connection_id,
            "10.0.0.1:65412".parse().unwrap(),
            "10.0.0.2:443".parse().unwrap()
        ));

        assert_eq!(
            connection_map.insert(
                connection_id,
                "10.0.0.1:65413".parse().unwrap(),
                "10.0.0.2:443".parse().unwrap()
            ),
            false
        );
    }

    #[test]
    fn insert_associates_multiple_connection_ids_if_addresses_already_exists() {
        let mut connection_map = ConnectionMap::new();

        let first_connection_id = ConnectionId::generate().unwrap();
        let second_connection_id = ConnectionId::generate().unwrap();

        let local_address = "10.0.0.1:65412".parse().unwrap();
        let remote_address = "10.0.0.2:443".parse().unwrap();

        assert!(connection_map.insert(first_connection_id, local_address, remote_address));

        assert!(connection_map.insert(second_connection_id, local_address, remote_address));

        assert_eq!(
            connection_map.get_connection_id(local_address, remote_address),
            Some(AddressConnectionIds::Multiple(hashset![
                first_connection_id,
                second_connection_id
            ]))
        );
    }

    #[test]
    fn contains_connection_id_returns_false_for_new_connection_id() {
        let connection_map = ConnectionMap::new();

        assert_eq!(
            connection_map.contains_connection_id(ConnectionId::generate().unwrap()),
            false
        );
    }

    #[test]
    fn contains_connection_id_returns_true_for_existing_connection_id() {
        let mut connection_map = ConnectionMap::new();

        let connection_id = ConnectionId::generate().unwrap();
        assert!(connection_map.insert(
            connection_id,
            "10.0.0.1:65412".parse().unwrap(),
            "10.0.0.2:443".parse().unwrap()
        ));

        assert!(connection_map.contains_connection_id(connection_id));
    }
}

use bimap::BiMap;
use protocol::ConnectionId;
use std::net::SocketAddr;

/// The type responsible for mapping to/from remote addresses and connection identifiers.
#[derive(Debug, Clone, Default)]
pub struct ConnectionMap {
    map: BiMap<ConnectionId, (SocketAddr, SocketAddr)>,
}

// TODO LH I think this is safe to do. remove once https://github.com/wrieger93/bimap-rs/issues/1 is released
unsafe impl Send for ConnectionMap {}
unsafe impl Sync for ConnectionMap {}

impl ConnectionMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_new_connection(
        &mut self,
        connection_id: ConnectionId,
        source_address: SocketAddr,
        destination_address: SocketAddr,
    ) -> bool {
        self.map
            .insert_no_overwrite(connection_id, (source_address, destination_address))
    }

    pub fn update_connection_addresses(
        &mut self,
        connection_id: ConnectionId,
        source_address: SocketAddr,
        destination_address: SocketAddr,
    ) -> bool {
        if self.map.remove_by_left(&connection_id).is_none() {
            false
        } else {
            self.insert_new_connection(connection_id, source_address, destination_address)
        }
    }

    pub fn update_connection_id(
        &mut self,
        old_connection_id: ConnectionId,
        new_connection_id: ConnectionId,
    ) -> bool {
        // check the new connection id isn't already mapped
        if self.map.get_by_left(&new_connection_id).is_some() {
            return false;
        }

        if let Some((_, socket_addresses)) = self.map.remove_by_left(&old_connection_id) {
            self.insert_new_connection(new_connection_id, socket_addresses.0, socket_addresses.1)
        } else {
            false
        }
    }

    pub fn contains_connection_id(&self, connection_id: ConnectionId) -> bool {
        self.map.contains_left(&connection_id)
    }

    pub fn get_connection_id(
        &self,
        source_address: SocketAddr,
        destination_address: SocketAddr,
    ) -> Option<ConnectionId> {
        self.map
            .get_by_right(&(source_address, destination_address))
            .map(|x| *x)
    }
}

#[cfg(test)]
mod tests {
    use super::ConnectionMap;
    use protocol::ConnectionId;
    use rand;

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

        let connection_id = ConnectionId::generate(&mut rand::thread_rng());

        let source_address = "10.0.0.1:65412".parse().unwrap();
        let destination_address = "10.0.0.2:443".parse().unwrap();
        assert!(connection_map.insert_new_connection(
            connection_id,
            source_address,
            destination_address
        ));

        assert_eq!(
            connection_map.get_connection_id(source_address, destination_address),
            Some(connection_id)
        );
    }

    #[test]
    fn insert_new_connection_fails_if_connection_id_already_exists() {
        let mut connection_map = ConnectionMap::new();

        let connection_id = ConnectionId::generate(&mut rand::thread_rng());
        assert!(connection_map.insert_new_connection(
            connection_id,
            "10.0.0.1:65412".parse().unwrap(),
            "10.0.0.2:443".parse().unwrap()
        ));

        assert_eq!(
            connection_map.insert_new_connection(
                connection_id,
                "10.0.0.1:65413".parse().unwrap(),
                "10.0.0.2:443".parse().unwrap()
            ),
            false
        );
    }

    #[test]
    fn insert_new_connection_fails_if_addresses_already_exists() {
        let mut connection_map = ConnectionMap::new();

        let first_connection_id = ConnectionId::generate(&mut rand::thread_rng());
        let second_connection_id = ConnectionId::generate(&mut rand::thread_rng());

        let source_address = "10.0.0.1:65412".parse().unwrap();
        let destination_address = "10.0.0.2:443".parse().unwrap();

        assert!(connection_map.insert_new_connection(
            first_connection_id,
            source_address,
            destination_address
        ));

        assert_eq!(
            connection_map.insert_new_connection(
                second_connection_id,
                source_address,
                destination_address
            ),
            false
        );
    }

    #[test]
    fn contains_connection_id_returns_false_for_new_connection_id() {
        let connection_map = ConnectionMap::new();

        assert_eq!(
            connection_map.contains_connection_id(ConnectionId::generate(&mut rand::thread_rng())),
            false
        );
    }

    #[test]
    fn contains_connection_id_returns_true_for_existing_connection_id() {
        let mut connection_map = ConnectionMap::new();

        let connection_id = ConnectionId::generate(&mut rand::thread_rng());
        assert!(connection_map.insert_new_connection(
            connection_id,
            "10.0.0.1:65412".parse().unwrap(),
            "10.0.0.2:443".parse().unwrap()
        ));

        assert!(connection_map.contains_connection_id(connection_id));
    }

    #[test]
    fn update_connection_id_fails_if_connection_id_does_not_exist() {
        let mut connection_map = ConnectionMap::new();

        let old_connection_id = ConnectionId::generate(&mut rand::thread_rng());
        let new_connection_id = ConnectionId::generate(&mut rand::thread_rng());

        assert_eq!(
            connection_map.update_connection_id(old_connection_id, new_connection_id),
            false
        );
        assert_eq!(
            connection_map.contains_connection_id(old_connection_id),
            false
        );
        assert_eq!(
            connection_map.contains_connection_id(new_connection_id),
            false
        );
    }

    #[test]
    fn update_connection_id_fails_if_new_connection_id_already_exists() {
        let mut connection_map = ConnectionMap::new();

        let first_connection_id = ConnectionId::generate(&mut rand::thread_rng());
        let second_connection_id = ConnectionId::generate(&mut rand::thread_rng());

        assert!(connection_map.insert_new_connection(
            first_connection_id,
            "10.0.0.1:65412".parse().unwrap(),
            "10.0.0.2:443".parse().unwrap()
        ));
        assert!(connection_map.insert_new_connection(
            second_connection_id,
            "10.0.0.1:65413".parse().unwrap(),
            "10.0.0.2:443".parse().unwrap()
        ));

        assert_eq!(
            connection_map.update_connection_id(first_connection_id, second_connection_id),
            false
        );

        assert!(connection_map.contains_connection_id(first_connection_id));
        assert!(connection_map.contains_connection_id(second_connection_id));
    }
}

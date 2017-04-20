use crypto::certificate::Certificate;
use fnv::FnvHasher;
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone)]
pub struct CertificateSet {
    certificates: Vec<Certificate>,
    hash: u64,
}

impl CertificateSet {
    fn new(mut certificates: Vec<Certificate>) -> Self {
        certificates.sort_by(|x, y| x.bytes().cmp(y.bytes()));

        let mut hasher = FnvHasher::default();

        for certificate in certificates.iter() {
            certificate.hash(&mut hasher);
        }

        Self {
            certificates: certificates,
            hash: hasher.finish(),
        }
    }

    pub fn certificate(&self, index: usize) -> Option<&Certificate> {
        if index < self.certificates.len() {
            Some(&self.certificates[index])
        } else {
            None
        }
    }

    pub fn index_of(&self, certificate: &Certificate) -> Option<usize> {
        self.certificates
            .binary_search_by(|c| c.bytes().cmp(certificate.bytes()))
            .ok()
    }

    pub fn hash(&self) -> u64 {
        self.hash
    }
}

impl From<Vec<Certificate>> for CertificateSet {
    fn from(value: Vec<Certificate>) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypto::certificate::Certificate;

    #[test]
    fn index_of_non_existent_certificate_returns_none() {
        // Arrange
        let certificate_set = CertificateSet::from(vec![]);

        // Act
        let index = certificate_set.index_of(&Certificate::new(vec![6, 3, 1, 3]));

        // Assert
        assert_eq!(index, None);
    }

    #[test]
    fn index_of_existent_certificate_returns_index() {
        // Arrange
        let certificate = Certificate::new(vec![6, 3, 1, 3]);
        let certificate_set = CertificateSet::from(vec![certificate.clone()]);

        // Act
        let index = certificate_set.index_of(&certificate);

        // Assert
        assert_eq!(index, Some(0));
    }

    #[test]
    fn non_existent_certificate_at_index_returns_none() {
        // Arrange
        let certificate_set = CertificateSet::from(vec![]);

        // Act
        let found_certificate = certificate_set.certificate(0);

        // Assert
        assert_eq!(found_certificate, None);
    }

    #[test]
    fn certificate_at_index_returns_certificate() {
        // Arrange
        let certificate = Certificate::new(vec![6, 3, 1, 3]);
        let certificate_set = CertificateSet::from(vec![certificate.clone()]);

        // Act
        let found_certificate = certificate_set.certificate(0);

        // Assert
        assert_eq!(found_certificate, Some(&certificate));
    }
}


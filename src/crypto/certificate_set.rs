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
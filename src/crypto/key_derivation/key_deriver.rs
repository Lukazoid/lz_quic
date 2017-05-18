use errors::*;
use crypto::key_derivation::DerivedKeys;

pub trait KeyDeriver {
    fn derive_keys(&self) -> Result<DerivedKeys>;
}


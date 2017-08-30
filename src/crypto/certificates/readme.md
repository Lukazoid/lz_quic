This is some data to be signed by the private key and the signature should subsequently be verified.

OpenSSL may be used to generate the signature of this file like so:
   
    openssl dgst -sha256 -sign example.pkcs8 -keyform der -binary -sigopt rsa_padding_mode:pss -sigopt rsa_pss_saltlen:-1 -out readme.signature.dat readme.md

Before the signature may be verified, we require the public key, which can be extracted from the certificate like so:

    openssl x509 -pubkey -in example.cer -inform der -out example.public.pem

The signature may then be verified like so:

    openssl dgst -sha256 -verify example.public.pem -sigopt rsa_padding_mode:pss -sigopt rsa_pss_saltlen:-1 -signature readme.signature.dat readme.md

The signature should be regenerated every time this file is changed.
use argon2::Argon2;
use crypto::aead::{ AeadDecryptor, AeadEncryptor };
use crypto::aes_gcm::AesGcm;
use std::error::Error;
use std::io::ErrorKind;

fn key_derivation(password: &[u8], salt: &[u8; 32]) -> Result<[u8; 32], argon2::Error> {
    let mut output_key: [u8; 32] = [0u8; 32];

    Argon2::default().hash_password_into(&password, salt, &mut output_key)?;

    Ok(output_key)
}

// https://stackoverflow.com/questions/43439771/how-do-i-create-an-empty-byte-array

/// Creates an initial vector (iv). This is also called a nonce
fn get_iv(size: usize) -> Vec<u8> {
    let mut iv: Vec<u8> = vec![];
    for _ in 0..size {
        let r: u8 = rand::random();
        iv.push(r);
    }

    iv
}

///encrypt "data" using "password" as the password
/// Output is [hexNonce]/[hexCipher]/[hexMac] (nonce and iv are the same thing)
pub fn encrypt(data: &[u8], password: &[u8], salt: &[u8; 32]) -> Result<String, argon2::Error> {
    let key_size = crypto::aes::KeySize::KeySize128;

    //pad or truncate the key if necessary
    let key: [u8; 32] = key_derivation(&password, &salt)?;
    let iv: Vec<u8> = get_iv(12); //initial vector (iv), also called a nonce
    let mut cipher: AesGcm<'_> = AesGcm::new(key_size, &key, &iv, &[]);

    //create a vec of data.len 0's. This is where the encrypted data will be saved.
    //the encryption is performed in-place, so this vector of 0's will be converted
    //to the encrypted data
    let mut encrypted: Vec<u8> = std::iter::repeat(0).take(data.len()).collect();

    //create a vec of 16 0's. This is for the mac. This library calls it a "tag", but it's really
    // the mac address. This vector will be modified in place, just like the "encrypted" vector
    // above
    let mut mac: Vec<u8> = std::iter::repeat(0).take(16).collect();

    //encrypt data, put it into "encrypted"
    cipher.encrypt(data, &mut encrypted, &mut mac[..]);

    //create the output string that contains the nonce, cipher text, and mac
    let hex_iv = hex::encode(iv);
    let hex_cipher = hex::encode(encrypted);
    let hex_mac = hex::encode(mac);
    let output = format!("{}/{}/{}", hex_iv, hex_cipher, hex_mac);

    Ok(output)
}

/// orig must be a string of the form [hexNonce]/[hexCipherText]/[hexMac]. This
/// is the data returned from encrypt(). This function splits the data, removes
/// the hex encoding, and returns each as a list of bytes.
fn split_iv_data_mac(orig: &str) -> Result<(Vec<u8>, Vec<u8>, Vec<u8>), Box<dyn Error>> {
    let split: Vec<&str> = orig.split('/').into_iter().collect();

    if split.len() != 3 {
        return Err(Box::new(std::io::Error::from(ErrorKind::Other)));
    }
    let iv_res = hex::decode(split[0]);
    if iv_res.is_err() {
        return Err(Box::new(std::io::Error::from(ErrorKind::Other)));
    }
    let iv = iv_res.unwrap();

    let data_res = hex::decode(split[1]);
    if data_res.is_err() {
        return Err(Box::new(std::io::Error::from(ErrorKind::Other)));
    }
    let data = data_res.unwrap();

    let mac_res = hex::decode(split[2]);
    if mac_res.is_err() {
        return Err(Box::new(std::io::Error::from(ErrorKind::Other)));
    }
    let mac = mac_res.unwrap();

    Ok((iv, data, mac))
}

///Decryption using AES-GCM 128
///iv_data_mac is a string that contains the iv/nonce, data, and mac values. All these values
/// must be hex encoded, and separated by "/" i.e. [hex(iv)/hex(data)/hex(mac)]. This function decodes
/// the values. key (or password) is the raw (not hex encoded) password
pub fn decrypt(
    iv_data_mac: &str,
    password: &[u8],
    salt: &[u8; 32]
) -> Result<String, Box<dyn Error>> {
    let (iv, data, mac) = split_iv_data_mac(iv_data_mac)?;
    let key: [u8; 32] = match key_derivation(&password, &salt) {
        Ok(k) => k,
        _ => {
            return Err(Box::new(std::io::Error::from(ErrorKind::Other)));
        }
    };

    let key_size: crypto::aes::KeySize = crypto::aes::KeySize::KeySize128;

    // I don't use the aad for verification. aad isn't encrypted anyway, so it's just specified
    // as &[].
    let mut decipher: AesGcm<'_> = AesGcm::new(key_size, &key, &iv, &[]);

    // create a list where the decoded data will be saved. dst is transformed in place. It must be exactly the same
    // size as the encrypted data
    let mut dst: Vec<u8> = std::iter::repeat(0).take(data.len()).collect();
    let result: bool = decipher.decrypt(&data, &mut dst, &mac);

    if !result {
        return Err(Box::new(std::io::Error::from(ErrorKind::PermissionDenied)));
    }

    Ok(String::from_utf8(dst)?)
}

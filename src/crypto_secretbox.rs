use crypto::{curve25519, salsa20, poly1305};

use crypto::symmetriccipher::SynchronousStreamCipher;
use crypto::mac::Mac;

use super::{PublicX25519, Secret, Error}; 



use ::{XSALSA20_NONCE_BYTES, PRIVATE_KEY_BYTES, POLY1305_MAC_BYTES};

pub const SECRET_BYTES: usize =32;



fn poly1305_(key: &[u8], msg: &[u8], mac: &mut [u8]){
    
    let mut poly = poly1305::Poly1305::new(key);
    poly.input(msg);
    poly.raw_result(mac);
    
}


pub fn seal(msg: &[u8], nonce: &[u8;XSALSA20_NONCE_BYTES], key:&[u8;SECRET_BYTES] ) -> Result< Vec<u8>, Error>
{

    let mut xs = salsa20::Salsa20::new_xsalsa20(key, nonce);

    //key for Poly1305 generated by encrypting 32 bytes of zeros
    let pzero =[0u8;32];
    let mut poly_key = [0u8; 32];
    
    xs.process(&pzero, &mut poly_key);
    
    let mut out = vec![0u8; msg.len() + POLY1305_MAC_BYTES];

    //encrypt stream
    xs.process( msg, &mut out[POLY1305_MAC_BYTES..]);

    {
        let (left, right) = out.split_at_mut(POLY1305_MAC_BYTES);
        poly1305_(&poly_key,right, left);
    }
   

    Ok(out)

}




pub fn open(cipher: &[u8], nonce: &[u8;XSALSA20_NONCE_BYTES], key: &[u8;SECRET_BYTES])->Result< Vec<u8> , Error>
{

    if cipher.len() < POLY1305_MAC_BYTES{
        return Err(Error::Custom(String::from("Message is invalid")));
    }
    let mut xs = salsa20::Salsa20::new_xsalsa20(key, nonce);
    
    let mut poly_key = [0u8; 32];
    let pzero =[0u8;32];
    xs.process(&pzero, &mut poly_key);


    //Verify Mac
    let mac = &cipher[..POLY1305_MAC_BYTES];
    let mut tmp= [0u8; POLY1305_MAC_BYTES];
    poly1305_( &poly_key, &cipher[POLY1305_MAC_BYTES..],&mut tmp );
    
    if !crypto::util::fixed_time_eq(mac,&tmp ){
        return Err(Error::Custom(String::from("Message Authentication Failed!")));
    }

    //Decrypt 
    let mut out = vec![0u8; cipher.len() - POLY1305_MAC_BYTES];
    xs.process(&cipher[POLY1305_MAC_BYTES..], &mut out);
    
    Ok(out)

}



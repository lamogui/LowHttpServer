use std::env;
use std::io::BufReader;
use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::fs::File;
use std::mem::transmute;
use std::collections::HashMap;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    let packFilename: String;
    if args.len() > 1 {
        packFilename = String::from( &args[ 1 ] );
    } else {
        packFilename = String::from( "website.pack" );
    }

	let mut fileEntries: HashMap< String, Vec<u8> > = HashMap::new();
    {
        let packFile: File = match File::open( packFilename.as_str() ) {
            Ok(f) => {
                // L'ouverture du fichier s'est bien déroulée, on renvoie l'objet
                f
            }
            Err(e) => {
                println!("Erreur impossible d'ouvrir {} : {:?}", packFilename.as_str(), e);
                return;
            }
        };

        let mut reader = BufReader::new( packFile );
        'readFile: loop {
			let mut keySizeBuf : [u8;4] = [0,0,0,0];
            match reader.read_exact(&mut keySizeBuf) {
				Ok ( size ) => {
					//assert_eq!( size, 4 );
				}
				Err( e ) => {
					break 'readFile;					
				}
			}
			let keySize : u32 = u2::from_le_bytes( keySizeBuf );
			let mut keyVec: Vec< u8 > = vec![ 0; keysize ];
			match reader.read_exact(&mut keyVec) {
				Ok ( size ) => {
					//assert_eq!( size, keySize as usize );
				}
				Err( e ) => {
					println!("Erreur impossible de lire la clef {:?}", e);
					break 'readFile;					
				}
			}
			let keyString: String = match String::from_utf8( keyVec ) {
				Ok( s ) => { s }
				Err( e ) => { 
					println!("Erreur impossible de decoder l'utf8 {:?}", e);
					break 'readFile;	
				}
			};
			let mut answerSizeBuf : [u8;4] = [0,0,0,0];
            match reader.read_exact(&mut answerSizeBuf) {
				Ok ( size ) => {
					//assert_eq!( size, 4 );
				}
				Err( e ) => {
					println!("Erreur impossible de lire la taille des datas de {} {:?}", keyString.as_str(), e);
					break 'readFile;					
				}
			}
			let mut answerSize : u32 = u32::from_le_bytes( answerSizeBuf );
			let mut dataVec: Vec< u8 > = vec![ 0; answerSize ];
			match reader.read_exact(&mut dataVec) {
				Ok ( size ) => {
					//assert_eq!( size, answerSize as usize );
				}
				Err( e ) => {
					println!("Erreur durant la lecture des datas de {} {:?}", keyString.as_str(), e);
					break 'readFile;					
				}
			}
			println!( "Readed {} {} bytes", keyString.as_str(), answerSize );
			if ( !fileEntries.insert( keyString.clone(), dataVec ).is_none() ) {
				println!( "Warning, double insertion for key {}", keyString.as_str() );
			}
        }
    }

    let listener: TcpListener = match TcpListener::bind("0.0.0.0:80") {
        Ok( l ) => { l }
        Err(e) => {
            println!("Erreur de creer le server : {:?}", e);
            return;
        }
    };

    'serverLoop: for stream in listener.incoming() {
        match stream {
            Ok( mut stream) => {
				let mut requestBuffer: Vec< u8 > = Vec::new();
				
				let socketStr: String= match stream.peer_addr() {
					Ok( a ) => {
						a.to_string()
					} Err( e ) => {
						println!( "Erreur impossible de recuperer l'addresse {:?}", e );
						"<erreur>".to_string()
					}
				};
				'singleRequestLoop: loop {
					let mut tempBuffer: [u8; 1024] = [0; 1024]; 
					match stream.read( &mut tempBuffer ) {
						Ok( size ) => {
							if (size > 0) {
								requestBuffer.extend_from_slice( &tempBuffer );
								let requestString: String = match String::from_utf8( requestBuffer.clone() ) {
									Ok( s ) => { 
										s 
									} Err( e ) => { 
										println!( "Utf8 error {} {:?}", socketStr.as_str(), e );
										break 'singleRequestLoop;
									}
								};

								if ( requestString.len() > 9000 ) {
									println!( "Huge input request {} {}", socketStr.as_str(), requestString.as_str() );
									break 'singleRequestLoop; // break connection if the request is too long
								}

								// detect empty line that mean request is finnished 
								if ( requestString.find( "\r\n\r\n" ).is_some() || requestString.find( "\n\n" ).is_some() ) {
									let mut split = requestString.split_whitespace();
									split.next(); // skip GET/POST etc
									match ( split.next() ) {
										Some( uri ) => {
											//let uriOwned = uri.to_string();
											//let uriSlash = uri.to_string() + "/";
											let possibleUris: [ &str; 1] = [ uri ];
											for uri in possibleUris.into_iter() {
												match ( fileEntries.get_key_value( uri ) ) {
													Some( (key, value) ) => {
														match stream.write_all( &value ) {
															Ok(_) => {
																println!( "Send page {} success for {}", uri, socketStr.as_str() );
																break 'singleRequestLoop;
															}
															Err( e ) => {
																println!( "Failed to send page {} for {}", uri, socketStr.as_str() );
																break 'singleRequestLoop;
															}
														}
													} None => {
														println!( "No 404 page for request {} from {}", uri, socketStr.as_str() );
														break 'singleRequestLoop; // todo 
													}
												} 
											}
										} None => {
											// continue until having more characters
										}
									}
								}
							}
						} Err( e ) => {
							println!( "Erreur de lecture de socket {} {:?}", socketStr.as_str(), e );
							break 'singleRequestLoop;
						}
					}
				}
            }
            Err(e) => { println!("Erreur connection : {:?}", e); }
        }
    }
}

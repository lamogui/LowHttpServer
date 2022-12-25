
#![allow(unused_parens)]
#![allow(unused_labels)]
#![allow(non_snake_case)]

extern crate low;
use low::Log;
use low::LogWarning;
use low::LogError;


use std::env;

use std::io::BufReader;
use std::io::Read;
use std::io::Write;

use std::net::TcpListener;
use std::net::TcpStream;

use std::fs::File;

use std::collections::HashMap;

use std::time::Instant;

use std::sync::Arc;

static ERROR_501_STR : &str = "HTTP/1.0 501 Not Implemented\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: 73\r\n\r\n<h1>501 Not Implemented</h1><p>This server only support GET requests.</p>";
static ERROR_404_STR : &str = "HTTP/1.0 404 Not Found\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: 59\r\n\r\n<h1>404 Not Found</h1><p>File not found on this server.</p>";

fn HandleConnection( mut _stream : TcpStream, _fileEntries: &HashMap< String, Vec<u8> > ) {
	let mut requestBuffer: Vec< u8 > = Vec::new();
	let socketStr: String= match _stream.peer_addr() {
		Ok( a ) => {
			a.to_string()
		} Err( e ) => {
			low::Error!( "Impossible de recuperer l'addresse {:?}", e );
			"<erreur>".to_string()
		}
	};
	let startedConnectionTime = Instant::now();
	'singleRequestLoop: loop {
		if ( startedConnectionTime.elapsed().as_secs() > 5 ) {
			low::Printf!( "Timeout for request from {} after {} ms\n", socketStr.as_str(), startedConnectionTime.elapsed().as_millis() );
			break 'singleRequestLoop; // break connection if the request is too long
		}
		let mut tempBuffer: [u8; 1024] = [0; 1024]; 
		match _stream.read( &mut tempBuffer ) {
			Ok( size ) => {
				if (size > 0) {
					requestBuffer.extend_from_slice( &tempBuffer );
					let requestString: String = match String::from_utf8( requestBuffer.clone() ) {
						Ok( s ) => { 
							s 
						} Err( e ) => { 
							low::Error!( "Utf8 error {} {:?}", socketStr.as_str(), e );
							break 'singleRequestLoop;
						}
					};

					if ( requestString.len() > 9000 ) {
						low::Error!( "Huge input request {} {}", socketStr.as_str(), requestString.as_str() );
						break 'singleRequestLoop; // break connection if the request is too long
					}

					// detect empty line that mean request is finnished 
					if ( requestString.find( "\r\n\r\n" ).is_some() || requestString.find( "\n\n" ).is_some() ) {
						let mut split = requestString.split_whitespace();
						let keyword = split.next();
						if (keyword.is_none()) {
							continue 'singleRequestLoop;
						}
						let keyword: &str = keyword.unwrap();
						match ( keyword ) { // GET/POST etc 
							"GET" => {

							} _ => {
								match _stream.write_all( ERROR_501_STR.as_bytes() ) {
									Ok(_) => {
										low::Printf!( "Send 501 page success for {} [{} ms]\n", socketStr.as_str(), startedConnectionTime.elapsed().as_millis() );
										break 'singleRequestLoop;
									}
									Err( e ) => {
										low::Error!( "Failed to send 501 page for {} {}", socketStr.as_str(), e );
										break 'singleRequestLoop;
									}
								}
								break 'singleRequestLoop;
							}
						}
						match ( split.next() ) {
							Some( uri ) => {
								let possibleUris: [ &str; 1] = [ uri ]; 
								for uri in possibleUris.into_iter() {
									match ( _fileEntries.get_key_value( uri ) ) {
										Some( (key, value) ) => {
											match _stream.write_all( &value ) {
												Ok(_) => {
													low::Printf!( "Send page {} success for {} [{} ms]\n", key.as_str(), socketStr.as_str(), startedConnectionTime.elapsed().as_millis() );
													break 'singleRequestLoop;
												}
												Err( e ) => {
													low::Error!( "Failed to send page {} for {} {}", key.as_str(), socketStr.as_str(), e );
													break 'singleRequestLoop;
												}
											}
										} None => {
											match _stream.write_all( ERROR_404_STR.as_bytes() ) {
												Ok(_) => {
													low::Printf!( "Send 404 page {} success for {} [{} ms]\n", uri, socketStr.as_str(), startedConnectionTime.elapsed().as_millis() );
													break 'singleRequestLoop;
												}
												Err( e ) => {
													low::Error!( "Failed to send 404 page {} for {} {}", uri, socketStr.as_str(), e );
													break 'singleRequestLoop;
												}
											}
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
				low::Error!( "Erreur de lecture de socket {} {:?}", socketStr.as_str(), e );
				break 'singleRequestLoop;
			}
		}
	}
}



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
                low::Error!("Erreur impossible d'ouvrir {} : {:?}", packFilename.as_str(), e);
                return;
            }
        };

        let mut reader = BufReader::new( packFile );
        'readFile: loop {
			let mut keySizeBuf : [u8;4] = [0,0,0,0];
            match reader.read_exact(&mut keySizeBuf) {
				Ok ( _ ) => {
				}
				Err( _ ) => {
					break 'readFile;					
				}
			}
			let keySize : u32 = u32::from_le_bytes( keySizeBuf );
			let mut keyVec: Vec< u8 > = Vec::new();
			keyVec.resize( keySize as usize, 0 );
			match reader.read_exact(&mut keyVec) {
				Ok ( _ ) => {
				}
				Err( e ) => {
					low::Error!("Impossible de lire la clef {:?}", e);
					break 'readFile;					
				}
			}
			let keyString: String = match String::from_utf8( keyVec ) {
				Ok( s ) => { s }
				Err( e ) => { 
					low::Error!("Impossible de decoder l'utf8 {:?}", e);
					break 'readFile;	
				}
			};
			let mut answerSizeBuf : [u8;4] = [0,0,0,0];
            match reader.read_exact(&mut answerSizeBuf) {
				Ok ( _ ) => {
				}
				Err( e ) => {
					low::Error!("Impossible de lire la taille des datas de {} {:?}", keyString.as_str(), e);
					break 'readFile;					
				}
			}
			let answerSize : u32 = u32::from_le_bytes( answerSizeBuf );
			let mut dataVec: Vec< u8 > = Vec::new();
			dataVec.resize( answerSize as usize, 0 );
			match reader.read_exact(&mut dataVec) {
				Ok ( _ ) => {
				}
				Err( e ) => {
					low::Error!("Impossible de lire les datas de {} {:?}", keyString.as_str(), e);
					break 'readFile;					
				}
			}
			low::Printf!( "Readed {} ({} bytes)\n", keyString.as_str(), answerSize );
			if ( !fileEntries.insert( keyString.clone(), dataVec ).is_none() ) {
				low::Warning!( "Double insertion for key {}", keyString.as_str() );
			}
        }
    }

	let fileEntriesShared = Arc::new( fileEntries );

    let listener: TcpListener = match TcpListener::bind("0.0.0.0:80") {
        Ok( l ) => { l }
        Err(e) => {
            low::Error!("Impossible de binder la socket : {:?}", e);
            return;
        }
    };

    'serverLoop: for stream in listener.incoming() {
        match stream {
            Ok( stream) => {
				let fileEntryClone = fileEntriesShared.clone();
				std::thread::spawn( move || { HandleConnection( stream, &fileEntryClone ); } );
			}	
            Err(e) => { low::Error!("Connection : {:?}", e); }
		}
    }
}

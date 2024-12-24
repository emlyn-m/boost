/*
    Set of functions to manage saving and loading credentials for bridge bots
*/

#![allow(unused)] 

use std::fs;
use regex::Regex;
use bcrypt;

const SUPPORTED_PLATFORMS: &[&str] = &["discord", "instagram", "fb_messenger"];

pub struct HomeserverCredentials {
    pub address: String,
    pub username: String,
    pub password: String,
}

pub fn load_homeserver_creds(credfile_path: &'static str) -> Result<HomeserverCredentials, String> {
    // todo: add some point support multiple homeservers
    let contents = match fs::read_to_string(credfile_path) {
        Ok(contents) => contents,
        Err(_) => return Err("Unable to open credential file".to_string())
    };

    let credline_seperate_re = Regex::new("[\r\n]+").unwrap();
    let header_line_re = Regex::new("\\[.+\\]").unwrap();
    let kv_split_re = Regex::new("=").unwrap();

    let mut url = "";
    let mut username = "";
    let mut password = "";

    let lines = credline_seperate_re.split(&contents);
    for line in lines {
        if line.trim().is_empty() || header_line_re.is_match(line) {
            continue;
        }

        let credpair_kv: Vec::<_> = kv_split_re.split(line.trim()).collect();
        if credpair_kv.len() != 2 {
            return Err(format!("Invalid line in credential file: \"{}\"", line));
        }

        let cred_key = credpair_kv[0];
        let cred_value = credpair_kv[1];

        match cred_key {
            "url" => url = cred_value,
            "username" => username = cred_value,
            "password" => password = cred_value,
            &_ => panic!("Unknown line in credfile: {}", cred_key)
        };
    }

    Ok(HomeserverCredentials {
        address: url.to_string(),
        username: username.to_string(),
        password: password.to_string()
    })

}


pub struct BridgeBotCredentials {
    pub bot_address: String, // address of the puppeting bot on our homeserver
    pub service_name: String, // name of the external service, used to handle username conflicts between platforms
    pub username: String,  // Specifically the username used for boost client -> boost server authentication, no relation to the platform username or bot address
    password: String, // See BridgeBotCredentials::username, note this is hashing using bcrypt with a cost of 12 (the default in the bcrypt crate)
}

impl BridgeBotCredentials {

    pub fn new(bot_address: String, service_name: String, username: String, password: String) -> BridgeBotCredentials {

        BridgeBotCredentials{
            bot_address,
            service_name,
            username,
            password,
        }

    }

    pub fn validate_credentials(&self, username: &str, password: &[u8]) -> Result<bool, bcrypt::BcryptError> {
        // technically double-checking as username is used to find the correct BridgeBotCredentials, but stil worth doing
        if username != self.username {
            return Ok(false);
        }

        match bcrypt::verify(password, &self.password) {
            Ok(res) => return Ok(res),
            Err(why) => return Err(why), 
        };
    }
}

fn set_credential(store: &mut String, key: &str, val: String) -> Result<(), String> {
    if store == "" {
        *store = val.to_string();
        Ok(())
    } else {
        Err(format!("Attempt to double-set property \"{}\"", key))
    }
}

pub fn load_credential_file(credfile_path: &'static str) -> Result<Vec::<BridgeBotCredentials>, String> {
    let mut current_credentials: Vec::<BridgeBotCredentials> = vec![];
    let contents = match fs::read_to_string(credfile_path) {
        Ok(cont) => cont,
        Err(_) => return Err("Unable to open credential file".to_string()),
    };

    let allcred_split_re = Regex::new("\\[.+?\\]").unwrap(); // would like to have ^$ but isn't working, might be a CRFL issue
    let credline_seperate_re = Regex::new("[\r\n]+").unwrap();
    let kv_split_re = Regex::new("=").unwrap();
    
    
    let all_credentials = allcred_split_re.split(&contents);


    for botcred_details in all_credentials {

        if botcred_details.trim().is_empty() {
            // artifact of using regex - just empty, skip it
            continue;
        }

        let credential_pairs = credline_seperate_re.split(&botcred_details);

        let mut ccred_bot_address: String = "".to_string();
        let mut ccred_service_name: String = "".to_string();
        let mut ccred_username: String = "".to_string();
        let mut ccred_password: String = "".to_string();


        for credpair in credential_pairs {

            if credpair.trim().is_empty() {
                // yet another regex artifact :(
                continue;
            }

            let credpair_kv: Vec::<_> = kv_split_re.split(credpair.trim()).collect();
            if credpair_kv.len() != 2 {
                return Err(format!("Invalid line in credential file: \"{}\"", credpair));
            }

            let cred_key = credpair_kv[0];
            let cred_value = credpair_kv[1];
            if cred_value.trim().is_empty() {
                return Err(format!("No value set for key \"{}\"", cred_key));
            }

            match cred_key {
                "bot_address" => set_credential(&mut ccred_bot_address, cred_key, cred_value.to_lowercase())?,
                "service_name" => {
                    if !SUPPORTED_PLATFORMS.contains(&cred_value) {
                        return Err(format!("Unsupported platform: {}", cred_value));
                    }
                    set_credential(&mut ccred_service_name, cred_key, cred_value.to_string())?;
                }
                "username" => set_credential(&mut ccred_username, cred_key, cred_value.to_lowercase())?,
                "password" => set_credential(&mut ccred_password, cred_key, cred_value.to_string())?,

                _ => return Err(format!("Unknown key \"{}\" in credential file", cred_key)),
            };

        }

        // Check for a duplicated bot address
        for existing_botcred in &current_credentials {
            if existing_botcred.bot_address == ccred_bot_address {
                return Err(format!("Duplicated bot address \"{}\"", ccred_bot_address));
            }
        } 

        if ccred_bot_address != "" && ccred_service_name != "" && ccred_username != "" && ccred_password != "" {
            current_credentials.push(BridgeBotCredentials::new(ccred_bot_address, ccred_service_name, ccred_username, ccred_password));
        } else {
            return Err("Missing values for a bot's credentials".to_string());
        }

    }

    return Ok(current_credentials);
}
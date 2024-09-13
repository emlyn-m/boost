use bitvec::prelude::*;

pub const COMMAND_BITLENGTH: usize = 8;
pub type CommandInt = u8;


#[repr(u8)]
#[derive(Debug)]
pub enum CommandValue {
    // cryptography
    DhkeInit = 1, // 1 of these each way...
    DhkeValidate = 2, // then 1 of these each way (likely with some known value (<4222.2209>))
    Unencrypted = 3, // Reply when an instruction requiring encryption is received and the user has not yet established a secure connection

    // account related
    AuthenticateToAccount = 4, // authenticate matrix bridge for a new user@domain type account (e.g. linking a discord account / fb messenger account)
    AuthenticationResult = 12, // Response to AuthenticateToAccount, contains NULL if unsuccessful, and 0x01 followed by the domain_idx if successful
    RequestKnownUsers = 7, // Request last N users on a given domain
    SignOut = 14, // Deauthenticate the sender from a given bot (send bot_idx)
    RevokeAllClients = 13, // Revoke _all_ boost clients authentication with a given bot (requires authentication with the bot obv) (send bot address)
    
    // message related
    UnknownDomain = 5, // (used as a response to DAT [ user_idx@domain_idx payload ])
    TargetUserNotFound = 6,
    

    // general
    Error = 8,
    InvalidCommand = 9,
    DuplicateBlock = 10, // not strictly for blocks but single-part messages dont have idempotency tokens
    BlockAck = 11,
}



pub struct Command {}
impl Command {
    
    pub fn get_matching_command(payload: &BitVec::<u8,Lsb0>) -> CommandValue {
        let command_value: CommandInt = payload.get(0..COMMAND_BITLENGTH).unwrap().load::<CommandInt>();
        
        {
            if command_value == CommandValue::DhkeInit as CommandInt { CommandValue::DhkeInit }
            else if command_value == CommandValue::DhkeValidate as CommandInt { CommandValue::DhkeValidate }
            // else if command_value == CommandValue::Unencrypted as CommandInt { CommandValue::Unencrypted }
            else if command_value == CommandValue::AuthenticateToAccount as CommandInt { CommandValue::AuthenticateToAccount }
            else if command_value == CommandValue::SignOut as CommandInt { CommandValue::SignOut }
            else if command_value == CommandValue::RevokeAllClients as CommandInt { CommandValue::RevokeAllClients } // for some reason this is flagged as unreachable
            // else if command_value == CommandValue::UnknownDomain as CommandInt { CommandValue::UnknownDomain }
            // else if command_value == CommandValue::TargetUserNotFound as CommandInt { CommandValue::TargetUserNotFound }
            else if command_value == CommandValue::RequestKnownUsers as CommandInt { CommandValue::RequestKnownUsers }
            else if command_value == CommandValue::Error as CommandInt { CommandValue::Error }
            else if command_value == CommandValue::BlockAck as CommandInt { CommandValue::BlockAck }
            else { panic!("Unknown command attempted") }
        }
    }


}
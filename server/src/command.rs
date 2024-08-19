use bitvec::prelude::*;

pub const COMMAND_BITLENGTH: usize = 8;
pub type CommandInt = u8;


#[repr(u8)]
#[derive(Debug)]
pub enum CommandValue {
    // cryptography
    DhkeInit = 1, // 1 of these each way...
    DhkeValidate = 2, // then 1 of these each way (likely with some known value (<4222.2209>))
    Unencrypted = 3, // Used to respond to a message which requires encryption

    // account related
    AuthenticateNewAccount = 4, // authenticate matrix bridge for a new user@domain type account (e.g. linking a discord account / fb messenger account)
    RequestKnownUsers = 7, // Request last N users on a given domain
    
    // message related
    DomainUnlinked = 5, // sender does not have the corrosponding domain linked (used as a response to DAT [ user@unlinked_domain payload ])
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
        
        return {
            if command_value == CommandValue::DhkeInit as CommandInt { CommandValue::DhkeInit }
            else if command_value == CommandValue::DhkeValidate as CommandInt { CommandValue::DhkeValidate }
            else if command_value == CommandValue::Unencrypted as CommandInt { CommandValue::Unencrypted }
            else if command_value == CommandValue::AuthenticateNewAccount as CommandInt { CommandValue::AuthenticateNewAccount }
            else if command_value == CommandValue::DomainUnlinked as CommandInt { CommandValue::DomainUnlinked }
            else if command_value == CommandValue::TargetUserNotFound as CommandInt { CommandValue::TargetUserNotFound }
            else if command_value == CommandValue::RequestKnownUsers as CommandInt { CommandValue::RequestKnownUsers }
            else if command_value == CommandValue::Error as CommandInt { CommandValue::Error }
            else if command_value == CommandValue::BlockAck as CommandInt { CommandValue::BlockAck }
            else { panic!("Unknown command attempted") }
        };
    }


}
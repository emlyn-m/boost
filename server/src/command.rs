use bitvec::prelude::*;

pub const COMMAND_BITLENGTH: usize = 8;
pub type CommandInt = u8;


#[repr(u8)]
#[derive(Debug)]
pub enum CommandValue {
    // cryptography
    DhkeInit = 1, // 1 of these each way...
    Unencrypted = 3, // Reply when an instruction requiring encryption is received and the user has not yet established a secure connection

    // account related
    AuthenticateToAccount = 4, // authenticate matrix bridge for a new user@domain type account (e.g. linking a discord account / fb messenger account)
    AuthenticationResult = 12, // Response to AuthenticateToAccount, contains NULL if unsuccessful, and 0x01 followed by the domain_idx if successful
    RequestKnownUsers = 7, // Request last N users on a given domain
    SignOut = 14, // Deauthenticate the sender from a given bot (send bot_idx)
    SignOutSuccess = 17,
    RevokeAllClients = 13, // Revoke _all_ boost clients authentication with a given bot (requires authentication with the bot obv) (send bot address)
    RequestDomains = 15,
    DomainUpdate = 18,
    ChannelUpdate = 16, // response to RequestKnownUsers

    // message related
    UnknownDomain = 5, // (used as a response to DAT [ user_idx@domain_idx payload ])
    TargetUserNotFound = 6,
    FindUser = 19, // Find a user by their remote id, add to known_users
    UserFound = 20, // CMD_19 successful! Client should send RequestKnownUsers
    
    // general
    Error = 8,
    InvalidCommand = 9,
    BlockAck = 11,

    Data = 255,
}

impl std::convert::TryFrom<u8> for CommandValue {
    type Error = &'static str;
    fn try_from(command_value: u8) -> Result<Self, <CommandValue as TryFrom<u8>>::Error> {
        if command_value == CommandValue::DhkeInit as CommandInt {  Ok(CommandValue::DhkeInit) }
        // else if command_value == CommandValue::Unencrypted as CommandInt { CommandValue::Unencrypted }
        else if command_value == CommandValue::AuthenticateToAccount as CommandInt {  Ok(CommandValue::AuthenticateToAccount) }
        else if command_value == CommandValue::SignOut as CommandInt {  Ok(CommandValue::SignOut) }
        else if command_value == CommandValue::RevokeAllClients as CommandInt {  Ok(CommandValue::RevokeAllClients) } // for some reason this is flagged as unreachable
        else if command_value == CommandValue::RequestDomains as CommandInt {  Ok(CommandValue::RequestDomains) }
        else if command_value == CommandValue::UnknownDomain as CommandInt {  Ok(CommandValue::UnknownDomain) }
        else if command_value == CommandValue::TargetUserNotFound as CommandInt {  Ok(CommandValue::TargetUserNotFound) }
        else if command_value == CommandValue::RequestKnownUsers as CommandInt {  Ok(CommandValue::RequestKnownUsers) }
        else if command_value == CommandValue::Error as CommandInt {  Ok(CommandValue::Error) }
        else if command_value == CommandValue::BlockAck as CommandInt { Ok(CommandValue::BlockAck) }
        else { Err("Unknown command") }
    }
}

pub struct Command {}
impl Command {

    pub fn get_matching_command(payload: &BitVec::<u8,Lsb0>) -> Result<CommandValue, ()> {
        let command_value: CommandInt = match payload.get(0..COMMAND_BITLENGTH) {
            Ok(x) => x.load::<CommandInt>(),
            Err(_) => return Err()
        }
        return command_value.try_into()
    }


}
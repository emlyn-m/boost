# boost-sms

<div align="center"><img align="center" src="assets/logo.png" /></div>

**<p align="center">Proxy layer for communication with Matrix bots over SMS</p>**

*<p align="center">in which our hero fulfills her childhood dream of writing a datasheet</p>*

<!-- ci badge -->

**<p align="center"><a href="#features">features</a></p>**
**<p align="center"><a href="#installation--configuration">installation + configuration</a></p>**
**<p align="center"><a href="#process">process + specs</a></p>**

## Features
|  | Supported? | Notes
| -- |:--:| -- |
| Multiple accounts | ✅ | Up to 256 |
| Multiple numbers | ✅ ||
| Arbitrary matrix bots | ✅ |Currently only [mautrix-discord](https://github.com/mautrix/discord) tested|
| Non-text messages | ❌ | planned |
| Encryption | ❌ | Key exchange supported, encryption in the works! | |
| Sending messages | ✅ ||
| Receiving messages | ✅ ||
| Refreshing user list | ✅ ||
| Messaging unknown external user | ❌ | planned | 

## Installation / Configuration

### Building

Ensure that `server/src/sms.rs` is modified to work with your chosen SMS integration method. By default it will operate on files at `sharedmem/server_input` and `sharedmem/server_output`.

**Requirements**
- `server`: rust, cargo
- `testing`: python3

**Build from source**
```
git clone https://github.com/emlyn-m/boost && \
    cd boost && \
    cargo build --release
    
// executable is in ./boost/target/release
```

### Server Configuration

**`credfile.cfg`**
```
[credential_block_nickname]
bot_address= //bot_name@homeserver.example.com
service_name= //typically name of external platform
username= //used only for boost
password= //used only for boost - bcrypt hash, cost 12
dm_space_id=
admin_room_id=

[credential_block_two_nickname]
...
```

**`homeserver_creds.cfg`**
```
[homeserver_nickname]
url=https://homeserver.example.com
username=@admin:homeserver.example.com
password=plaintext

[homeserver_two_nickname]
...
```

## Process

```
            client  |   boost_server  |  matrix_homeserver
```
**Initialization**
```
                      --------------> | register bots
                      --------------> | request bot0 channels
                        bot0 channels | <--------------
                           (update)   |
                      --------------> | request bot1 channels
                        bot1 channels | <--------------
                           (update)   |
                                     ...
```


**Setup/Auth**
```
    --------------> | auth_to_account
        unencrypted | <--------------
```
```
    --------------> | dhke_init
          block_ack | <--------------
          dhke_init | <--------------
    --------------> | block_ack
```
```
    --------------> | auth_to_account
          block_ack | <--------------
        auth_result | <--------------
    --------------> | block_ack
```
```
    --------------> | signout
          block_ack | <--------------
    signout_success | <--------------
    --------------> | block_ack
```

**Configuring users**
```
    --------------> | reqdomains      |
          block_ack | <-------------- |
                    | --------------> | fetch users
                    |  (update list)  |
      domain_update | <-------------- |
    --------------> | block_ack       |
```
```                  
    --------------> |     requsers   
          block_ack | <--------------
                    |   (checks did) 
     unknown_domain | <--------------
```
```    
    --------------> |     requsers   
          block_ack | <--------------
                    |   (checks did) 
     channel_update | <--------------
```
```
                    |  new user list  | <--------------
                    |  (update list)  |
      domain_update | <-------------- |
    --------------> | block_ack       |
```

**Sending and Receiving messages**
```
    --------------> | msg:data        |
          block_ack | <-------------- |
                    |    (process)    |
                    | --------------> | payload_data
                    |  delivery info  | <--------------
    <-------------? | error
```


```
                    |    payload_data | <--------------
                    |    (process)    |
           msg:data | <-------------- |
    --------------> | block_ack       |
```

## Protocol Specification

### `msg`
| Name | Start (hex) | End (hex) | Size (bits) | Guaranteed | Notes |
|--|--|--|--|--|--|
|`block_id`|0x00|0x08|8|No|Only present if `head_universal.flag_is_multipart` set. If present, all other ranges shifted by 8 bits. If `head_universal.flag_mp_first` set, this instead is the number of blocks in the full message, and this block has `block_id=0`|
|`head_universal`|0x00-0x08|0x08-0x16|8|Yes||
|`command_id`|0x08-0x16|0x16-0x24|8|No|One of `command_id` or `head_data` guaranteed|
|`command_data`|0x16-0x24|varies|varies|No||
|`head_data`|0x08-0x16|varies|varies|No|One of `command_id` or `head_data` guaranteed|
|`payload_data`|0x08-varies|varies|varies|with `head_data`|General utf8 text|


### `head_universal`
| Name | Start (hex) | End (hex) | Size (bits) | Guaranteed | Notes |
|--|--|--|--|--|--|
| `flag_is_command` | 0x00 | 0x01 | 1 | Yes ||
| `flag_is_multipart` | 0x01 | 0x02 | 1 | Yes ||
| `flag_mp_first` | 0x02 | 0x03 | 1 | Yes ||
| `msg_id` | 0x03 | 0x08 | 5 | Yes ||

### `command_id`
| Name | Value (Hex) | Requires Ack | `command_data` | Notes
|--|--|--|--|--|
|`block_ack`| `0x0b` | No | `[0x00-0x08] block id` |  |
|`dhke_init`| `0x01` | No | `[0x00-0xff] x25519 client public` |  |
|`auth_to_account`| `0x04` | Yes | `[0x00-varies] credfile.cfg:service_name` `[] 0x00` `[varies-varies] credfile.cfg:username` `[] 0x00` `[varies-varies] credfile.cfg:password` `[] 0x00`|  |
|`auth_result`| `0x0c` | No | `[0x00-0x08] status_res (normal=1)` `[0x08-0x16] original msg_id` `[0x16-0x24] new domain_id` | response to `auth_to_account` |
|`req_domains`| `0x0f` | No |  |  |
|`domain_update`| `0x12` | Yes | `[0x00-varies] name for domain_id=0` `[] 0x00` `[varies-varies] name for domain_id=1` `[] 0x00` `...` | response to `req_domains` |
|`req_known_users`| `0x07` | No | `[0x00-0x08] domain_id` |  |
|`channel_update`| `0x10` | Yes | `[0x00-0x08] domain_id` `[varies-varies] name for user_id=0 on domain_id` `[] 0x00` `[varies-varies] name for user_id=1 on domain_id` `[] 0x00` `...`| response to `req_known_users` |
|`find_user`| `0x13`|No|`[⚠️unimpl]`|
|`user_found`|`0x14`|No|`[⚠️unimpl]`| response to `find_user`|
|`revoke_all_clients`| `0x0d` | Yes | `[⚠️unimpl]` |  |
|`sign_out`| `0x0e` | Yes | `[0x00-0x08] domain_id to sign out of` |  |
|`signout_success`| `0x11` | Yes | `[0x00-0x08] domain_id signed out of`  | require client ACK as this may change the mapping of `domain_id`s |
|||||
|`error`| `0x08` | No | `[0x00-0x08] msg_id of cause` `[0x08-varies] error message (utf8)` |  |
|`invalid_command`| `0x09` | No | `[0x00-0x08] msg_id of cause` `[0x08-varies] error message (utf8)` |  |
|`duplicate_block`| `0x0a` | No | `[⚠️unimpl]` | client has sent this block before, generally equivalent to `block_ack` |
|`unencrypted`| `0x03` | No |  | client MUST encrypt with `dhke_init` before taking any other actions |
|`unknown_domain`| `0x05` | No | `[⚠️unimpl]` | response to `req_known_users` |
|`target_user_not_found`| `0x06` | No | `[0x00-0x08] msg_id of cause` `[0x08-varies] error message (utf8)` | response to `auth_to_account`, `find_user` |


### `head_data`
| Name | Start (hex) | End (hex) | Size (bits) | Guaranteed | Notes |
|--|--|--|--|--|--|
|`user_id`| 0x00 | 0x08 | 8 | Yes | |
|`domain_id` | 0x08 | 0x16 | 8 | Yes ||


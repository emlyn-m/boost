# Boost

Note that this is currently a very bad readme, and is bascially just a list of notes for what should be in the final version

- Setup
    - This project requires a self-hosted matrix homeserver, with all necessary bridge bots set up and authenticated before this project can be used
    - Credentials.cfg
        - This project requires a credential file to authenticate this server with a client, for each bot they wish to access
        - The structure of this file is any number of the blocks defined below

        ```
        [bot-identifier]
        bot_address=value
        service_name=value
        username=value
        password=value
        ```

        Outline of the different parameters in a block:
            - bot-identifier: The value of this is completely ignored by the code, however it must have some value (i.e. bot-identifier != "") for structural reasons
            - bot_address: The full matrix address of the relevant puppeting bot
            - service_name: The name of the service. This should be all lowercase, and one of the following supported values:
                - TODO: Write list of supported platforms
            - username: The username a user of the boost client uses to authenticate, no relation to the actual username on the platform
            - password: See username, must be in the form of a bcrypt hash (compliant with the rust crate (bcrypt)[https://docs.rs/bcrypt/latest/bcrypt/]), and a cost value of 12
                - TODO: Change the cost value if we decide to use a different one


RESET = "\x1b[0m"
BOLD = "\x1b[1m"

INFO_MSG = "        \n\x1b[1m\x1b[38;5;089m         _                     _       _______  _     _______          \n        | |                   | |     / /  __ \\| |   |_   _\\ \\         \n        | |__   ___   ___  ___| |_   | || /  \\/| |     | |  | |        \n        | '_ \\ / _ \\ / _ \\/ __| __|  | || |    | |     | |  | |        \n        | |_) | (_) | (_) \\__ \\ |_   | || \\__/\\| |_____| |_ | |        \n        |_.__/ \\___/ \\___/|___/\\__|  | | \\____/\\_____/\\___/ | |        \n                                      \\_\\                  /_/         \n        CTRL+C for input, .help for help                               \n                                                                       \x1b[0m"
HELP_MSG = "\
\x1b[1m                                     \x1b[38;5;203m.help\x1b[0m  Show this message\n\
\x1b[1m                                     \x1b[38;5;203m.quit\x1b[0m  Quit\n\
\x1b[1m                    \x1b[38;5;203m.loglevel [debug|prod]\x1b[0m  Set the level used for logging\n\
\x1b[1m                        \x1b[38;5;203m.ph [phone number]\x1b[0m  Switch the testing phone number\n\
\x1b[1m                                     \x1b[38;5;203m.init\x1b[0m  Setup a communication channel\n\
\x1b[1m\x1b[38;5;203m.auth [service name] [username] [password]\x1b[0m  Authenticate a given account\n\
\x1b[1m      \x1b[38;5;203m.send [user_idx@bridgebot_idx] [msg]\x1b[0m  Send a message to target@t_domain\n\
\x1b[1m                                \x1b[38;5;203m.lsdomains\x1b[0m  List all authenticated domains and indices\n\
\x1b[1m                         \x1b[38;5;203m.lsusers [domain]\x1b[0m  List all users and their indices on [domain]\n\
\x1b[1m                               \x1b[38;5;203m.reqdomains\x1b[0m  Request an updated list of domains from the server\n\
\x1b[1m                        \x1b[38;5;203m.requsers [domain]\x1b[0m  Request an updated list of users and indices on [domain]\n\
\x1b[1m                    \x1b[38;5;203m.logout [domain index]\x1b[0m  Sign out of platform [domain index]\n\
\x1b[1m                 \x1b[38;5;203m.revokeall [domain index]\x1b[0m  Sign out of platform [domain index] on all clients\n\
"
 
PH_INPUT = "\x1b[1m\x1b[38;5;218m ph# \x1b[0m "
PH_INVALID = "\x1b[1m\x1b[38;5;124mInvalid phone number\x1b[0m"
COMMAND_INPUT = "\n\x1b[1m\x1b[38;5;213m inp \x1b[0m "
LOG_COLORS = { "debug": "\x1b[38;5;162m\x1b[1m", "warn": "\x1b[38;5;202m\x1b[1m", "prod": "\x1b[38;5;66m\x1b[1m", "err": "\x1b[38;5;52m\x1b[1m" }

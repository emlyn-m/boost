RESET = "\x1b[0m"
BOLD = "\x1b[1m"

INFO_MSG = "\n\x1b[1;m\x1b[38;5;213m\t _                     _       _______  _     _______  \n\t| |                   | |     / /  __ \\| |   |_   _\\ \\ \n\t| |__   ___   ___  ___| |_   | || /  \\/| |     | |  | |\n\t| '_ \\ / _ \\ / _ \\/ __| __|  | || |    | |     | |  | |\n\t| |_) | (_) | (_) \\__ \\ |_   | || \\__/\\| |_____| |_ | |\n\t|_.__/ \\___/ \\___/|___/\\__|  | | \\____/\\_____/\\___/ | |\n\t                              \\_\\                  /_/ \x1b[0;m\n\n\n\t\x1b[38;5;213mCTRL+C for input, .help for help\x1b[0m\n\n"
HELP_MSG = "\x1b[1m\
\x1b[38;5;141m.help\x1b[0m                                         Show this message\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.quit\x1b[0m                                         Quit\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.loglevel [debug|prod]\x1b[0m                        Set the level used for logging\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.ph [phone number]\x1b[0m                            Switch the testing phone number\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.init\x1b[0m                                         Setup a communication channel\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.auth [service name] [username] [password]\x1b[0m    Authenticate a given account\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.send [user_idx@bridgebot_idx] [msg]\x1b[0m          Send a message to target@t_domain\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.lsdomains\x1b[0m                                    List all authenticated domains and indices\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.lsusers [domain]\x1b[0m                             List all users and their indices on [domain]\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.reqdomains\x1b[0m                                   Request an updated list of domains from the server\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.requsers [domain]\x1b[0m                            Request an updated list of users and indices on [domain]\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.logout [domain index]\x1b[0m                        Sign out of platform [domain index]\n\
\x1b[38;5;132m--------------------------------------------\x1b[0m\x1b[1m\n\
\x1b[38;5;141m.revokeall [domain index]\x1b[0m                     Sign out of platform [domain index] on all clients\n\
"

PH_INPUT = "\x1b[38;5;141mPhone number: \x1b[0m"
PH_INVALID = "\x1b[38;5;124mInvalid phone number!!\x1b[0m"
COMMAND_INPUT = "\n\x1b[38;5;212m❯❯❯\x1b[0m "

# lan server
Do you ever need to share root acces to all your files over LAN?  
I love security risks, so i made this.  
Atleast it has auth so only people you want can acces your files .

## Installation
- grab a release
- unzip It
- run the file
    - lan_server.exe on windows
    - lan_server on linux
- make sure to set your preffered username and password in the `.env` file

## Using the server
- Run the .exe
- Get your local ip adress
- on a browser, enter the adress in this configuration: `~your_local_ip:8080/login/~your_username/~your_password`
- you will be redirected back to the directory list

## Building locally:
- `git clone` the repo
- create a `.env`, otherwise login is `admin:password` by default
- `cargo run`

source "$HOME/.cargo/env"
cd ~
git clone https://github.com/AlistairKeiller/discordBot
cd ~/discordBot
git pull
DISCORD_TOKEN= cargo run --release

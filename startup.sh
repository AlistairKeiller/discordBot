sudo dnf install git -y
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
source "$HOME/.cargo/env"
cd ~
git clone https://github.com/AlistairKeiller/discordBot
cd ~/discordBot
git pull
DISCORD_TOKEN= cargo run --release

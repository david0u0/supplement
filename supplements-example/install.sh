set -e

cd $(dirname $0)

if [[ ! -v USR_PATH ]]; then
    read -p "Please provide the path to usr (or through env variable \$USR_PATH): " USR_PATH
fi

USR_PATH=$(eval realpath $USR_PATH)

echo Building binary
cargo build

BIN_FILE=$USR_PATH/bin/qit
echo Installing binary to $BIN_FILE...
cp ../target/debug/supplements-example $BIN_FILE

COMP_FILE=$USR_PATH/share/fish/completions/qit.fish
echo Installing binary to $COMP_FILE...
cp shell/qit.fish $COMP_FILE
BIN_FILE_ESC=$(echo $BIN_FILE | sed -e "s/\//\\\\\//g")
sed -i -e "s/PLACEHOLDER_FOR_BIN_PATH/$BIN_FILE_ESC/g" $COMP_FILE


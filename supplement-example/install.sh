#!/usr/bin/env bash

set -e

cd $(dirname $0)

if [[ ! -v USR_PATH ]]; then
    read -p "Please provide the path to usr (or through env variable \$USR_PATH): " USR_PATH
fi

USR_PATH=$(eval realpath $USR_PATH)
BIN=../target/debug/supplement-example

echo Building binary
set +e
cargo build
if [ ! "$?" = "0" ]; then
    if [[ -f $BIN ]]; then
        read -n1 -p "Cargo build failed, but the binary exists. Do you want to proceed? [Y/N] " RESP
        echo
        if [[ "$RESP" =~ ([yY]) ]]; then
            echo "Continue installing..."
        else
            exit 1
        fi
    else
        exit 1
    fi
fi
set -e

BIN_FILE=$USR_PATH/bin/qit
echo Installing binary to $BIN_FILE...
cp $BIN $BIN_FILE

COMP_FILE=$USR_PATH/share/fish/completions/qit.fish
echo Installing completion file to $COMP_FILE...
cp shell/qit.fish $COMP_FILE
BIN_FILE_ESC=$(echo $BIN_FILE | sed -e "s/\//\\\\\//g")
sed -i -e "s/PLACEHOLDER_FOR_BIN_PATH/$BIN_FILE_ESC/g" $COMP_FILE

COMP_FILE=$USR_PATH/share/bash-completion/completions/qit
echo Installing completion file to $COMP_FILE...
cp shell/qit.bash $COMP_FILE
BIN_FILE_ESC=$(echo $BIN_FILE | sed -e "s/\//\\\\\//g")
sed -i -e "s/PLACEHOLDER_FOR_BIN_PATH/$BIN_FILE_ESC/g" $COMP_FILE

COMP_FILE=$USR_PATH/local/share/zsh/site-functions/_qit
echo Installing completion file to $COMP_FILE...
cp shell/qit.zsh $COMP_FILE
BIN_FILE_ESC=$(echo $BIN_FILE | sed -e "s/\//\\\\\//g")
sed -i -e "s/PLACEHOLDER_FOR_BIN_PATH/$BIN_FILE_ESC/g" $COMP_FILE

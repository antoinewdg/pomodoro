install:
	cargo build --release
	sudo ln -sf ${ROOT_DIR}/systemd/pomodoro.service /etc/systemd/system/pomodoro.service
	sudo ln -sf ${ROOT_DIR}/target/release/pomodoro /usr/local/bin/pomodoro
	sudo systemctl daemon-reload
	sudo systemctl enable pomodoro.service
	sudo systemctl restart pomodoro.service

.PHONY: install

ROOT_DIR:=$(shell dirname $(realpath $(firstword $(MAKEFILE_LIST))))
